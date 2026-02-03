use std::time::Instant;

#[cfg(feature = "perf")]
use std::time::Duration;

#[cfg(feature = "perf")]
use std::sync::OnceLock;

#[cfg(feature = "perf")]
use obs_sys as obs;

#[cfg(feature = "perf")]
use crate::util::cstr;

#[cfg(feature = "perf")]
pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| match std::env::var("STYLEDCAMERA_PERF") {
        Ok(v) => {
            let v = v.trim();
            !v.is_empty() && v != "0" && v.to_ascii_lowercase() != "false"
        }
        Err(_) => false,
    })
}

#[cfg(not(feature = "perf"))]
#[allow(dead_code)]
pub(crate) fn enabled() -> bool {
    false
}

#[cfg(feature = "perf")]
#[derive(Clone, Copy, Default)]
struct StageStats {
    count: u64,
    total_ns: u64,
    max_ns: u64,
}

#[cfg(feature = "perf")]
impl StageStats {
    fn record(&mut self, d: Duration) {
        let ns = d.as_nanos().min(u128::from(u64::MAX)) as u64;
        self.count = self.count.saturating_add(1);
        self.total_ns = self.total_ns.saturating_add(ns);
        self.max_ns = self.max_ns.max(ns);
    }

    fn avg_ms(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        (self.total_ns as f32) / (self.count as f32) / 1_000_000.0
    }

    fn max_ms(&self) -> f32 {
        (self.max_ns as f32) / 1_000_000.0
    }
}

pub(crate) struct RenderPerf {
    #[cfg(feature = "perf")]
    enabled: bool,
    #[cfg(feature = "perf")]
    last_log: Instant,
    #[cfg(feature = "perf")]
    interval: Duration,
    #[cfg(feature = "perf")]
    frame: StageStats,
    #[cfg(feature = "perf")]
    consume_outputs: StageStats,
    #[cfg(feature = "perf")]
    render_source: StageStats,
    #[cfg(feature = "perf")]
    seg_request: StageStats,
    #[cfg(feature = "perf")]
    blur: StageStats,
    #[cfg(feature = "perf")]
    composite: StageStats,
    #[cfg(feature = "perf")]
    shape: StageStats,
}

impl RenderPerf {
    pub(crate) fn new() -> Self {
        #[cfg(feature = "perf")]
        {
            let enabled = enabled();
            Self {
                enabled,
                last_log: Instant::now(),
                interval: Duration::from_secs(1),
                frame: StageStats::default(),
                consume_outputs: StageStats::default(),
                render_source: StageStats::default(),
                seg_request: StageStats::default(),
                blur: StageStats::default(),
                composite: StageStats::default(),
                shape: StageStats::default(),
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            Self {}
        }
    }

    pub(crate) fn start(&self) -> Option<Instant> {
        #[cfg(feature = "perf")]
        {
            if self.enabled {
                Some(Instant::now())
            } else {
                None
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            None
        }
    }

    pub(crate) fn record_frame(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.frame.record(t0.elapsed());
            }
            self.maybe_log();
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_consume_outputs(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.consume_outputs.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_render_source(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.render_source.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_seg_request(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.seg_request.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_blur(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.blur.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_composite(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.composite.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_shape(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.shape.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    #[cfg(feature = "perf")]
    fn maybe_log(&mut self) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_log);
        if dt < self.interval {
            return;
        }
        self.last_log = now;

        let secs = dt.as_secs_f32().max(1e-3);
        let fps = (self.frame.count as f32) / secs;

        let msg = format!(
            "perf(render): fps={fps:.1} n={} frame={:.3}ms (max {:.3}) | src={:.3} | seg_req={:.3} | blur={:.3} | comp={:.3} | shape={:.3}",
            self.frame.count,
            self.frame.avg_ms(),
            self.frame.max_ms(),
            self.render_source.avg_ms(),
            self.seg_request.avg_ms(),
            self.blur.avg_ms(),
            self.composite.avg_ms(),
            self.shape.avg_ms(),
        );

        unsafe {
            if let Ok(s) = std::ffi::CString::new(msg) {
                obs::blog(obs::LOG_INFO as i32, cstr(b"StyledCamera: %s\n\0"), s.as_ptr());
            }
        }

        self.frame = StageStats::default();
        self.consume_outputs = StageStats::default();
        self.render_source = StageStats::default();
        self.seg_request = StageStats::default();
        self.blur = StageStats::default();
        self.composite = StageStats::default();
        self.shape = StageStats::default();
    }
}

pub(crate) struct SegPerf {
    #[cfg(feature = "perf")]
    enabled: bool,
    #[cfg(feature = "perf")]
    last_log: Instant,
    #[cfg(feature = "perf")]
    interval: Duration,
    #[cfg(feature = "perf")]
    total: StageStats,
    #[cfg(feature = "perf")]
    preprocess: StageStats,
    #[cfg(feature = "perf")]
    infer: StageStats,
    #[cfg(feature = "perf")]
    postprocess: StageStats,
}

impl SegPerf {
    pub(crate) fn new() -> Self {
        #[cfg(feature = "perf")]
        {
            let enabled = enabled();
            Self {
                enabled,
                last_log: Instant::now(),
                interval: Duration::from_secs(1),
                total: StageStats::default(),
                preprocess: StageStats::default(),
                infer: StageStats::default(),
                postprocess: StageStats::default(),
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            Self {}
        }
    }

    pub(crate) fn start(&self) -> Option<Instant> {
        #[cfg(feature = "perf")]
        {
            if self.enabled {
                Some(Instant::now())
            } else {
                None
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            None
        }
    }

    pub(crate) fn record_total(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.total.record(t0.elapsed());
            }
            self.maybe_log();
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_preprocess(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.preprocess.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_infer(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.infer.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    pub(crate) fn record_postprocess(&mut self, start: Option<Instant>) {
        #[cfg(feature = "perf")]
        {
            if let Some(t0) = start {
                self.postprocess.record(t0.elapsed());
            }
        }

        #[cfg(not(feature = "perf"))]
        {
            let _ = start;
        }
    }

    #[cfg(feature = "perf")]
    fn maybe_log(&mut self) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_log);
        if dt < self.interval {
            return;
        }
        self.last_log = now;

        let secs = dt.as_secs_f32().max(1e-3);
        let hz = (self.total.count as f32) / secs;

        let msg = format!(
            "perf(seg): hz={hz:.1} n={} total={:.3}ms (max {:.3}) | pre={:.3} | infer={:.3} | post={:.3}",
            self.total.count,
            self.total.avg_ms(),
            self.total.max_ms(),
            self.preprocess.avg_ms(),
            self.infer.avg_ms(),
            self.postprocess.avg_ms(),
        );

        unsafe {
            if let Ok(s) = std::ffi::CString::new(msg) {
                obs::blog(obs::LOG_INFO as i32, cstr(b"StyledCamera: %s\n\0"), s.as_ptr());
            }
        }

        self.total = StageStats::default();
        self.preprocess = StageStats::default();
        self.infer = StageStats::default();
        self.postprocess = StageStats::default();
    }
}
