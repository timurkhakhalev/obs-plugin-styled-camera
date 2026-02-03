use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc, Condvar, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use obs_sys as obs;

use crate::constants::MODEL_FILE;
use crate::perf::SegPerf;
use crate::util::cstr;

pub(crate) struct SegInput {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub temporal_smoothing: f32,
    pub capture_time: Instant,
}

pub(crate) struct SegOutput {
    pub mask: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub capture_time: Instant,
}

pub(crate) struct SegInbox {
    state: Mutex<SegInboxState>,
    cv: Condvar,
}

struct SegInboxState {
    latest: Option<SegInput>,
    shutdown: bool,
}

impl SegInbox {
    fn new() -> Self {
        Self {
            state: Mutex::new(SegInboxState {
                latest: None,
                shutdown: false,
            }),
            cv: Condvar::new(),
        }
    }

    pub(crate) fn push_latest(&self, input: SegInput) {
        let Ok(mut guard) = self.state.lock() else {
            return;
        };
        if guard.shutdown {
            return;
        }
        guard.latest = Some(input);
        self.cv.notify_one();
    }

    pub(crate) fn shutdown(&self) {
        let Ok(mut guard) = self.state.lock() else {
            return;
        };
        guard.shutdown = true;
        guard.latest = None;
        self.cv.notify_all();
    }

    fn pop_latest_blocking(&self) -> Option<SegInput> {
        let mut guard = self.state.lock().ok()?;
        loop {
            if guard.shutdown {
                return None;
            }
            if let Some(msg) = guard.latest.take() {
                return Some(msg);
            }
            guard = self.cv.wait(guard).ok()?;
        }
    }
}

pub(crate) struct SegmentationState {
    pub inbox: Option<Arc<SegInbox>>,
    pub rx: Option<Receiver<SegOutput>>,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Default for SegmentationState {
    fn default() -> Self {
        Self {
            inbox: None,
            rx: None,
            thread: None,
        }
    }
}

impl SegmentationState {
    pub(crate) unsafe fn ensure_running(&mut self) {
        if self.inbox.is_some() {
            return;
        }

        let dylib_path = match resolve_onnxruntime_dylib_path() {
            Some(p) => p,
            None => {
                obs::blog(
                    obs::LOG_WARNING as i32,
                    cstr(b"StyledCamera: ONNX Runtime dylib not found; segmentation disabled\n\0"),
                );
                return;
            }
        };

        let model_path = match resolve_model_path() {
            Some(p) => p,
            None => {
                obs::blog(
                    obs::LOG_WARNING as i32,
                    cstr(b"StyledCamera: segmentation model not found; segmentation disabled\n\0"),
                );
                return;
            }
        };

        let (out_tx, out_rx) = mpsc::sync_channel::<SegOutput>(1);

        let inbox = Arc::new(SegInbox::new());
        let inbox_for_thread = inbox.clone();
        let handle = thread::spawn(move || {
            segmentation_thread_main(inbox_for_thread, out_tx, dylib_path, model_path);
        });

        self.inbox = Some(inbox);
        self.rx = Some(out_rx);
        self.thread = Some(handle);
    }

    pub(crate) unsafe fn stop(&mut self) {
        if let Some(inbox) = self.inbox.take() {
            inbox.shutdown();
        }
        self.rx.take();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn segmentation_thread_main(
    inbox: Arc<SegInbox>,
    tx: SyncSender<SegOutput>,
    dylib_path: PathBuf,
    model_path: PathBuf,
) {
    let mut perf = SegPerf::new();

    unsafe {
        if let Ok(p) = CString::new(dylib_path.to_string_lossy().as_bytes()) {
            obs::blog(
                obs::LOG_INFO as i32,
                cstr(b"StyledCamera: ONNX Runtime dylib: %s\n\0"),
                p.as_ptr(),
            );
        }
        if let Ok(p) = CString::new(model_path.to_string_lossy().as_bytes()) {
            obs::blog(
                obs::LOG_INFO as i32,
                cstr(b"StyledCamera: segmentation model: %s\n\0"),
                p.as_ptr(),
            );
        }
    }

    if ort::init_from(&dylib_path)
        .and_then(|b| Ok(b.commit()))
        .is_err()
    {
        unsafe {
            obs::blog(
                obs::LOG_WARNING as i32,
                cstr(b"StyledCamera: failed to init ONNX Runtime; segmentation disabled\n\0"),
            );
        }
        return;
    }

    let mut session =
        match ort::session::Session::builder().and_then(|b| b.commit_from_file(model_path)) {
            Ok(s) => s,
            Err(_) => {
                unsafe {
                    obs::blog(
                        obs::LOG_WARNING as i32,
                        cstr(b"StyledCamera: failed to load segmentation model into ORT session\n\0"),
                    );
                }
                return;
            }
        };

    let mut prev_mask: Vec<f32> = Vec::new();
    let mut input_is_nchw: Option<bool> = None;
    let mut last_infer_error_log: Option<Instant> = None;

    while let Some(input) = inbox.pop_latest_blocking() {
        let t_total = perf.start();

        let w = input.width as usize;
        let h = input.height as usize;
        if w == 0 || h == 0 || input.rgba.len() != w * h * 4 {
            continue;
        }

        let expected = (input.width * input.height) as usize;
        let wh = w * h;
        let inv_255 = 1.0f32 / 255.0;

        let mut run_and_pick_output =
            |shape: Vec<i64>, data: Box<[f32]>| -> Option<(Vec<i64>, Vec<f32>)> {
                let tensor = ort::value::Tensor::<f32>::from_array((shape, data)).ok()?;
                let outputs = session.run(ort::inputs![tensor]).ok()?;
                for (_, v) in outputs.iter() {
                    if let Ok((shape, data)) = v.try_extract_tensor::<f32>() {
                        if data.len() == expected
                            || data.len() == expected * 2
                            || data.len() >= expected
                        {
                            let shape_vec: Vec<i64> = shape.iter().copied().collect();
                            return Some((shape_vec, data.to_vec()));
                        }
                    }
                }
                None
            };

        let picked = match input_is_nchw {
            Some(true) => {
                // Build NCHW directly.
                let t_pre = perf.start();
                let mut rgb = vec![0f32; wh * 3];
                for (i, px) in input.rgba.chunks_exact(4).enumerate() {
                    let r = px[0] as f32 * inv_255;
                    let g = px[1] as f32 * inv_255;
                    let b = px[2] as f32 * inv_255;
                    rgb[i] = r;
                    rgb[wh + i] = g;
                    rgb[wh * 2 + i] = b;
                }
                perf.record_preprocess(t_pre);

                let t_infer = perf.start();
                let res = run_and_pick_output(vec![1i64, 3, h as i64, w as i64], rgb.into_boxed_slice());
                perf.record_infer(t_infer);
                res
            }
            Some(false) => {
                // Build NHWC directly.
                let t_pre = perf.start();
                let mut rgb = Vec::<f32>::with_capacity(wh * 3);
                for px in input.rgba.chunks_exact(4) {
                    rgb.push(px[0] as f32 * inv_255);
                    rgb.push(px[1] as f32 * inv_255);
                    rgb.push(px[2] as f32 * inv_255);
                }
                perf.record_preprocess(t_pre);

                let t_infer = perf.start();
                let res = run_and_pick_output(vec![1i64, h as i64, w as i64, 3], rgb.into_boxed_slice());
                perf.record_infer(t_infer);
                res
            }
            None => {
                // Auto-detect layout once using NHWC input first (cheap to assemble from RGBA).
                let t_pre = perf.start();
                let mut rgb_nhwc = Vec::<f32>::with_capacity(wh * 3);
                for px in input.rgba.chunks_exact(4) {
                    rgb_nhwc.push(px[0] as f32 * inv_255);
                    rgb_nhwc.push(px[1] as f32 * inv_255);
                    rgb_nhwc.push(px[2] as f32 * inv_255);
                }
                perf.record_preprocess(t_pre);

                let t_infer = perf.start();
                let nhwc = run_and_pick_output(
                    vec![1i64, h as i64, w as i64, 3],
                    rgb_nhwc.clone().into_boxed_slice(),
                );
                perf.record_infer(t_infer);
                if nhwc.is_some() {
                    input_is_nchw = Some(false);
                    nhwc
                } else {
                    let t_pre = perf.start();
                    let mut rgb_nchw = vec![0f32; wh * 3];
                    for i in 0..wh {
                        rgb_nchw[i] = rgb_nhwc[i * 3];
                        rgb_nchw[wh + i] = rgb_nhwc[i * 3 + 1];
                        rgb_nchw[wh * 2 + i] = rgb_nhwc[i * 3 + 2];
                    }
                    perf.record_preprocess(t_pre);

                    let t_infer = perf.start();
                    let nchw = run_and_pick_output(
                        vec![1i64, 3, h as i64, w as i64],
                        rgb_nchw.into_boxed_slice(),
                    );
                    perf.record_infer(t_infer);
                    if nchw.is_some() {
                        input_is_nchw = Some(true);
                    }
                    nchw
                }
            }
        };

        let Some((out_shape, out_data)) = picked else {
            let now = Instant::now();
            let due = last_infer_error_log
                .map(|t| now.duration_since(t) >= Duration::from_secs(2))
                .unwrap_or(true);
            if due {
                unsafe {
                    obs::blog(
                        obs::LOG_WARNING as i32,
                        cstr(b"StyledCamera: segmentation inference failed (model/input mismatch?)\n\0"),
                    );
                }
                last_infer_error_log = Some(now);
            }
            continue;
        };

        let t_post = perf.start();
        let Some(mask_u8) = styledcamera_core::segmentation::postprocess_mask_u8(
            expected,
            &out_shape,
            &out_data,
            &mut prev_mask,
            input.temporal_smoothing,
        ) else {
            continue;
        };
        perf.record_postprocess(t_post);

        let _ = tx.try_send(SegOutput {
            mask: mask_u8,
            width: input.width,
            height: input.height,
            capture_time: input.capture_time,
        });

        perf.record_total(t_total);
    }
}

unsafe fn resolve_onnxruntime_dylib_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ONNXRUNTIME_DYLIB") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    let module = obs::obs_current_module();
    if module.is_null() {
        return None;
    }

    let bin_path = obs::obs_get_module_binary_path(module);
    if bin_path.is_null() {
        return None;
    }
    let mut bin = PathBuf::from(CStr::from_ptr(bin_path).to_string_lossy().to_string());
    // bin_path is typically a directory; if it's a file path, use its parent.
    if bin.is_file() {
        if let Some(parent) = bin.parent() {
            bin = parent.to_path_buf();
        }
    }

    let frameworks = bin.join("../Frameworks");
    let candidates = [
        frameworks.join("libonnxruntime.dylib"),
        frameworks.join("libonnxruntime.1.dylib"),
    ];
    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }

    // Fall back to any versioned name, e.g. libonnxruntime.1.23.0.dylib
    if let Ok(rd) = std::fs::read_dir(&frameworks) {
        for ent in rd.flatten() {
            let p = ent.path();
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("libonnxruntime") && name.ends_with(".dylib") && p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    None
}

unsafe fn resolve_model_path() -> Option<PathBuf> {
    let module = obs::obs_current_module();
    if module.is_null() {
        return None;
    }

    let p = obs::obs_find_module_file(module, cstr(MODEL_FILE));
    if p.is_null() {
        return None;
    }
    let s = CStr::from_ptr(p).to_string_lossy().to_string();
    obs::bfree(p.cast());
    let pb = PathBuf::from(s);
    if pb.is_file() { Some(pb) } else { None }
}
