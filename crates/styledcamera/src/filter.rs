use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use obs_sys as obs;
use styledcamera_core::timing::update_mask_latency_ema_ms;

use crate::constants::*;
use crate::frame_history::FrameHistory;
use crate::graphics::{
    draw_shape_to_screen, render_effect_to_texrender, render_source_to_texrender, set_float_param,
    set_vec2_param, GraphicsState,
};
use crate::perf::RenderPerf;
use crate::segmentation::{SegInput, SegmentationState, SegOutput};
use crate::settings::{self, FilterSettings};
use crate::util::cstr;

const SEG_SIZE: u32 = 256;
const DOWNSCALE_DIV: u32 = 2;

#[repr(C)]
struct StyledCameraFilter {
    source: *mut obs::obs_source_t,
    settings: FilterSettings,

    mask_latency_ema_ms: f32,
    last_mask_request: Option<Instant>,

    perf: RenderPerf,
    graphics: GraphicsState,
    segmentation: SegmentationState,
    frame_history: FrameHistory,
}

impl StyledCameraFilter {
    fn new(source: *mut obs::obs_source_t) -> Self {
        Self {
            source,
            settings: FilterSettings::default(),

            mask_latency_ema_ms: 0.0,
            last_mask_request: None,

            perf: RenderPerf::new(),
            graphics: GraphicsState::default(),
            segmentation: SegmentationState::default(),
            frame_history: FrameHistory::default(),
        }
    }
}

// Must be called while in graphics context.
unsafe fn apply_segmentation_output(
    gfx: &mut GraphicsState,
    mask_latency_ema_ms: &mut f32,
    out: SegOutput,
) {
    if out.width == 0 || out.height == 0 {
        return;
    }

    if gfx.mask_tex.is_null() || gfx.mask_w != out.width || gfx.mask_h != out.height {
        if !gfx.mask_tex.is_null() {
            obs::gs_texture_destroy(gfx.mask_tex);
            gfx.mask_tex = std::ptr::null_mut();
        }
        // Default to 0 (no person) until we receive real mask data.
        let init = vec![0u8; (out.width * out.height) as usize];
        let mut data_ptrs = [init.as_ptr()];
        gfx.mask_tex = obs::gs_texture_create(
            out.width,
            out.height,
            obs::gs_color_format_GS_R8,
            1,
            data_ptrs.as_mut_ptr(),
            obs::GS_DYNAMIC,
        );
        gfx.mask_w = out.width;
        gfx.mask_h = out.height;
    }

    if !gfx.mask_tex.is_null() && out.mask.len() == (out.width * out.height) as usize {
        obs::gs_texture_set_image(gfx.mask_tex, out.mask.as_ptr(), out.width, false);
    }

    let measured_ms = out.capture_time.elapsed().as_secs_f32() * 1000.0;
    update_mask_latency_ema_ms(mask_latency_ema_ms, measured_ms);
}

// Must be called while in graphics context.
unsafe fn consume_segmentation_outputs(filter: &mut StyledCameraFilter) {
    let mut disconnected = false;

    let gfx = &mut filter.graphics;
    let mask_latency_ema_ms = &mut filter.mask_latency_ema_ms;

    if let Some(rx) = filter.segmentation.rx.as_ref() {
        loop {
            match rx.try_recv() {
                Ok(out) => apply_segmentation_output(gfx, mask_latency_ema_ms, out),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }
    }

    if disconnected {
        filter.segmentation.stop();
    }
}

// Must be called while in graphics context.
unsafe fn maybe_request_segmentation(
    filter: &mut StyledCameraFilter,
    settings: FilterSettings,
    tex_current: *mut obs::gs_texture_t,
    cx: u32,
    cy: u32,
    frame_time: Instant,
) {
    if filter.segmentation.inbox.is_none() {
        return;
    }

    if !render_effect_to_texrender(
        filter.graphics.tex_seg,
        SEG_SIZE,
        SEG_SIZE,
        filter.graphics.effect_downsample,
        TECH_DOWNSAMPLE,
        || {
            set_vec2_param(
                filter.graphics.downsample_texel_size,
                1.0 / (cx as f32),
                1.0 / (cy as f32),
            );
            if !filter.graphics.downsample_image.is_null() {
                obs::gs_effect_set_texture(filter.graphics.downsample_image, tex_current);
            }
        },
        tex_current,
    ) {
        return;
    }

    let now = Instant::now();
    let interval = Duration::from_secs_f32(1.0 / (settings.mask_fps.max(1) as f32));
    let due = filter
        .last_mask_request
        .map(|t| now.duration_since(t) >= interval)
        .unwrap_or(true);

    if !due {
        return;
    }

    let Some(inbox) = filter.segmentation.inbox.as_ref() else {
        return;
    };
    let tex_seg = obs::gs_texrender_get_texture(filter.graphics.tex_seg);
    if tex_seg.is_null() || filter.graphics.stage_seg.is_null() {
        return;
    }

    obs::gs_stage_texture(filter.graphics.stage_seg, tex_seg);

    let mut data: *mut u8 = std::ptr::null_mut();
    let mut linesize: u32 = 0;
    if !obs::gs_stagesurface_map(filter.graphics.stage_seg, &mut data, &mut linesize) {
        return;
    }

    let mut rgba = vec![0u8; (SEG_SIZE * SEG_SIZE * 4) as usize];
    for y in 0..SEG_SIZE {
        let src_row = data.add((y * linesize) as usize);
        let dst_row = &mut rgba[(y * SEG_SIZE * 4) as usize..][..(SEG_SIZE * 4) as usize];
        std::ptr::copy_nonoverlapping(src_row, dst_row.as_mut_ptr(), dst_row.len());
    }
    obs::gs_stagesurface_unmap(filter.graphics.stage_seg);

    inbox.push_latest(SegInput {
        rgba,
        width: SEG_SIZE,
        height: SEG_SIZE,
        temporal_smoothing: settings.mask_temporal_smoothing,
        capture_time: frame_time,
    });
    filter.last_mask_request = Some(now);
}

// Must be called while in graphics context.
unsafe fn render_blur(
    gfx: &mut GraphicsState,
    tex_for_comp: *mut obs::gs_texture_t,
    cx: u32,
    cy: u32,
    blur_amount: f32,
) -> *mut obs::gs_texture_t {
    let mut blur_tex = tex_for_comp;
    if blur_amount <= 0.0001 {
        return blur_tex;
    }

    let down_cx = (cx / DOWNSCALE_DIV).max(1);
    let down_cy = (cy / DOWNSCALE_DIV).max(1);

    if !render_effect_to_texrender(
        gfx.tex_down,
        down_cx,
        down_cy,
        gfx.effect_downsample,
        TECH_DOWNSAMPLE,
        || {
            set_vec2_param(gfx.downsample_texel_size, 1.0 / (cx as f32), 1.0 / (cy as f32));
            if !gfx.downsample_image.is_null() {
                obs::gs_effect_set_texture(gfx.downsample_image, tex_for_comp);
            }
        },
        tex_for_comp,
    ) {
        return blur_tex;
    }

    let tex_down = obs::gs_texrender_get_texture(gfx.tex_down);
    if tex_down.is_null() {
        return blur_tex;
    }

    let blur_radius = (blur_amount * 24.0).max(0.0);
    let texel_x = 1.0 / (down_cx as f32);
    let texel_y = 1.0 / (down_cy as f32);

    let ok_h = render_effect_to_texrender(
        gfx.tex_ping,
        down_cx,
        down_cy,
        gfx.effect_blur,
        TECH_BLUR_H,
        || {
            set_vec2_param(gfx.blur_texel_size, texel_x, texel_y);
            set_float_param(gfx.blur_radius, blur_radius);
            if !gfx.blur_image.is_null() {
                obs::gs_effect_set_texture(gfx.blur_image, tex_down);
            }
        },
        tex_down,
    );

    let tex_ping = obs::gs_texrender_get_texture(gfx.tex_ping);
    if !ok_h || tex_ping.is_null() {
        return blur_tex;
    }

    let ok_v = render_effect_to_texrender(
        gfx.tex_pong,
        down_cx,
        down_cy,
        gfx.effect_blur,
        TECH_BLUR_V,
        || {
            set_vec2_param(gfx.blur_texel_size, texel_x, texel_y);
            set_float_param(gfx.blur_radius, blur_radius);
            if !gfx.blur_image.is_null() {
                obs::gs_effect_set_texture(gfx.blur_image, tex_ping);
            }
        },
        tex_ping,
    );

    if ok_v {
        let t = obs::gs_texrender_get_texture(gfx.tex_pong);
        if !t.is_null() {
            blur_tex = t;
        }
    }

    blur_tex
}

// Must be called while in graphics context.
unsafe fn render_composite(
    gfx: &mut GraphicsState,
    settings: FilterSettings,
    tex_for_comp: *mut obs::gs_texture_t,
    blur_tex: *mut obs::gs_texture_t,
    cx: u32,
    cy: u32,
) -> Option<*mut obs::gs_texture_t> {
    let ok = render_effect_to_texrender(
        gfx.tex_comp,
        cx,
        cy,
        gfx.effect_composite,
        TECH_COMPOSITE,
        || {
            if !gfx.composite_image.is_null() {
                obs::gs_effect_set_texture(gfx.composite_image, tex_for_comp);
            }
            if !gfx.composite_blur_image.is_null() {
                obs::gs_effect_set_texture(gfx.composite_blur_image, blur_tex);
            }
            if !gfx.composite_mask_image.is_null() {
                obs::gs_effect_set_texture(gfx.composite_mask_image, gfx.mask_tex);
            }

            set_float_param(
                gfx.composite_mask_threshold,
                settings.mask_threshold.clamp(0.0, 1.0),
            );
            set_float_param(gfx.composite_mask_softness, settings.mask_softness.clamp(0.0, 1.0));
            set_float_param(gfx.composite_mask_invert, if settings.mask_invert { 1.0 } else { 0.0 });
            set_float_param(gfx.composite_bg_dim, settings.bg_dim.clamp(0.0, 1.0));
            set_float_param(gfx.composite_bg_desat, settings.bg_desat.clamp(0.0, 1.0));
        },
        tex_for_comp,
    );

    let tex_comp = obs::gs_texrender_get_texture(gfx.tex_comp);
    if ok && !tex_comp.is_null() { Some(tex_comp) } else { None }
}

pub(crate) unsafe fn register_sources() {
    let mut info: obs::obs_source_info = std::mem::zeroed();
    info.id = cstr(FILTER_ID);
    info.type_ = obs::obs_source_type_OBS_SOURCE_TYPE_FILTER;
    info.output_flags = obs::OBS_SOURCE_VIDEO;

    info.get_name = Some(styled_camera_filter_get_name);
    info.create = Some(styled_camera_filter_create);
    info.destroy = Some(styled_camera_filter_destroy);
    info.update = Some(styled_camera_filter_update);
    info.get_defaults = Some(styled_camera_filter_get_defaults);
    info.get_properties = Some(styled_camera_filter_get_properties);
    info.video_render = Some(styled_camera_filter_video_render);

    obs::obs_register_source_s(
        &info as *const obs::obs_source_info,
        std::mem::size_of::<obs::obs_source_info>() as obs::size_t,
    );
}

unsafe extern "C" fn styled_camera_filter_get_name(_type_data: *mut c_void) -> *const c_char {
    FILTER_DISPLAY_NAME.as_ptr().cast()
}

unsafe extern "C" fn styled_camera_filter_create(
    settings_data: *mut obs::obs_data_t,
    source: *mut obs::obs_source_t,
) -> *mut c_void {
    let mut filter = Box::new(StyledCameraFilter::new(source));
    filter.settings = FilterSettings::load(settings_data);

    filter.graphics.init();
    if filter.settings.needs_segmentation() {
        filter.segmentation.ensure_running();
    }

    Box::into_raw(filter).cast()
}

unsafe extern "C" fn styled_camera_filter_destroy(data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let mut filter = Box::from_raw(data.cast::<StyledCameraFilter>());
    filter.segmentation.stop();
    filter.graphics.destroy(&mut filter.frame_history);
}

unsafe extern "C" fn styled_camera_filter_update(data: *mut c_void, settings_data: *mut obs::obs_data_t) {
    if data.is_null() {
        return;
    }
    let filter = &mut *data.cast::<StyledCameraFilter>();
    let old_needs_segmentation = filter.settings.needs_segmentation();
    filter.settings = FilterSettings::load(settings_data);
    let new_needs_segmentation = filter.settings.needs_segmentation();

    if new_needs_segmentation && filter.segmentation.inbox.is_none() {
        filter.segmentation.ensure_running();
    } else if old_needs_segmentation && !new_needs_segmentation {
        filter.segmentation.stop();
        filter.mask_latency_ema_ms = 0.0;
        filter.last_mask_request = None;

        obs::obs_enter_graphics();
        filter.frame_history.destroy();
        obs::obs_leave_graphics();
    }
}

unsafe extern "C" fn styled_camera_filter_get_defaults(settings_data: *mut obs::obs_data_t) {
    settings::set_defaults(settings_data);
}

unsafe extern "C" fn styled_camera_filter_get_properties(_data: *mut c_void) -> *mut obs::obs_properties_t {
    settings::get_properties()
}

unsafe extern "C" fn styled_camera_filter_video_render(data: *mut c_void, _effect: *mut obs::gs_effect_t) {
    if data.is_null() {
        return;
    }
    let filter = &mut *data.cast::<StyledCameraFilter>();

    if !filter.graphics.ensure() {
        obs::obs_source_skip_video_filter(filter.source);
        return;
    }

    let target = obs::obs_filter_get_target(filter.source);
    if target.is_null() {
        obs::obs_source_skip_video_filter(filter.source);
        return;
    }

    let cx = obs::obs_source_get_base_width(target);
    let cy = obs::obs_source_get_base_height(target);
    if cx == 0 || cy == 0 {
        obs::obs_source_skip_video_filter(filter.source);
        return;
    }

    let settings = filter.settings;
    let needs_segmentation = settings.needs_segmentation();
    let needs_background_composite = settings.needs_background_composite();
    let blur_amount = settings.blur_intensity.clamp(0.0, 1.0);

    if needs_segmentation {
        filter.segmentation.ensure_running();
    }

    let t_frame = filter.perf.start();

    obs::obs_enter_graphics();

    if needs_segmentation {
        let t = filter.perf.start();
        consume_segmentation_outputs(filter);
        filter.perf.record_consume_outputs(t);

        // Render source into current history slot (used both for segmentation capture and as delay buffer).
        filter.frame_history.ensure_allocated();
        let frame_time = Instant::now();
        let mut tex_current: *mut obs::gs_texture_t = std::ptr::null_mut();
        if let Some(entry) = filter.frame_history.next_slot_mut() {
            let t = filter.perf.start();
            let ok = !entry.tex.is_null() && render_source_to_texrender(entry.tex, cx, cy, target);
            filter.perf.record_render_source(t);
            if ok {
                entry.time = Some(frame_time);
                tex_current = obs::gs_texrender_get_texture(entry.tex);
            }
        }

        if !tex_current.is_null() {
            let delay_ms =
                (filter.mask_latency_ema_ms.max(0.0) + settings.sync_video_extra_delay_ms.max(0.0))
                    .clamp(0.0, 500.0);
            let tex_for_comp = filter.frame_history.select_delayed_texture(tex_current, delay_ms);

            let t = filter.perf.start();
            maybe_request_segmentation(filter, settings, tex_current, cx, cy, frame_time);
            filter.perf.record_seg_request(t);

            if settings.debug_show_mask && !filter.graphics.mask_tex.is_null() {
                let effect = obs::obs_get_base_effect(obs::obs_base_effect_OBS_EFFECT_DEFAULT);
                let image_param = obs::gs_effect_get_param_by_name(effect, cstr(b"image\0"));
                obs::gs_effect_set_texture(image_param, filter.graphics.mask_tex);
                while obs::gs_effect_loop(effect, cstr(b"Draw\0")) {
                    obs::gs_draw_sprite(filter.graphics.mask_tex, 0, cx, cy);
                }
                obs::obs_leave_graphics();
                filter.perf.record_frame(t_frame);
                return;
            }

            let tex_out = if needs_background_composite {
                let t = filter.perf.start();
                let blur_tex = render_blur(&mut filter.graphics, tex_for_comp, cx, cy, blur_amount);
                filter.perf.record_blur(t);

                let t = filter.perf.start();
                let res = render_composite(&mut filter.graphics, settings, tex_for_comp, blur_tex, cx, cy);
                filter.perf.record_composite(t);
                let Some(tex_comp) = res else {
                    obs::obs_leave_graphics();
                    filter.perf.record_frame(t_frame);
                    obs::obs_source_skip_video_filter(filter.source);
                    return;
                };
                tex_comp
            } else {
                tex_for_comp
            };

            if !tex_out.is_null() {
                let t = filter.perf.start();
                draw_shape_to_screen(&filter.graphics, &settings, tex_out, cx, cy);
                filter.perf.record_shape(t);
                obs::obs_leave_graphics();
                filter.perf.record_frame(t_frame);
                return;
            }
        }

        obs::obs_leave_graphics();
        filter.perf.record_frame(t_frame);
        obs::obs_source_skip_video_filter(filter.source);
        return;
    }

    // Fast path: no segmentation or background composite needed.
    let t = filter.perf.start();
    let ok = render_source_to_texrender(filter.graphics.tex_comp, cx, cy, target);
    filter.perf.record_render_source(t);
    if ok {
        let tex = obs::gs_texrender_get_texture(filter.graphics.tex_comp);
        if !tex.is_null() {
            let t = filter.perf.start();
            draw_shape_to_screen(&filter.graphics, &settings, tex, cx, cy);
            filter.perf.record_shape(t);
            obs::obs_leave_graphics();
            filter.perf.record_frame(t_frame);
            return;
        }
    }

    obs::obs_leave_graphics();
    filter.perf.record_frame(t_frame);
    obs::obs_source_skip_video_filter(filter.source);
}
