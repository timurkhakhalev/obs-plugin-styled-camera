#![allow(clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use obs_sys as obs;

static MODULE_NAME: &[u8] = b"StyledCamera\0";
static MODULE_DESCRIPTION: &[u8] = b"StyledCamera (Rust) - styled camera filter (skeleton)\0";
static MODULE_AUTHOR: &[u8] = b"StyledCamera contributors\0";

static FILTER_ID: &[u8] = b"styled_camera_filter\0";
static FILTER_DISPLAY_NAME: &[u8] = b"Styled Camera Filter\0";

static SETTING_BLUR_INTENSITY: &[u8] = b"blur_intensity\0";
static SETTING_DEBUG_SHOW_MASK: &[u8] = b"debug_show_mask\0";
static SETTING_MASK_FPS: &[u8] = b"mask_fps\0";
static SETTING_MASK_TEMPORAL: &[u8] = b"mask_temporal_smoothing\0";
static SETTING_MASK_THRESHOLD: &[u8] = b"mask_threshold\0";
static SETTING_MASK_SOFTNESS: &[u8] = b"mask_softness\0";
static SETTING_MASK_INVERT: &[u8] = b"mask_invert\0";
static SETTING_BG_DIM: &[u8] = b"bg_dim\0";
static SETTING_BG_DESAT: &[u8] = b"bg_desat\0";
static SETTING_SHAPE_TYPE: &[u8] = b"shape_type\0";
static SETTING_CORNER_RADIUS: &[u8] = b"corner_radius\0";
static SETTING_FEATHER: &[u8] = b"feather\0";
static SETTING_BORDER_THICKNESS: &[u8] = b"border_thickness\0";
static SETTING_BORDER_COLOR: &[u8] = b"border_color\0";
static SETTING_PADDING: &[u8] = b"padding\0";
static SETTING_SHADOW_OPACITY: &[u8] = b"shadow_opacity\0";
static SETTING_SHADOW_BLUR: &[u8] = b"shadow_blur\0";
static SETTING_SHADOW_OFFSET_X: &[u8] = b"shadow_offset_x\0";
static SETTING_SHADOW_OFFSET_Y: &[u8] = b"shadow_offset_y\0";
static SETTING_SHADOW_COLOR: &[u8] = b"shadow_color\0";

static PROP_BLUR_INTENSITY: &[u8] = b"Blur intensity\0";
static PROP_DEBUG_SHOW_MASK: &[u8] = b"Debug: show mask\0";
static PROP_MASK_FPS: &[u8] = b"Mask FPS\0";
static PROP_MASK_TEMPORAL: &[u8] = b"Mask temporal smoothing\0";
static PROP_MASK_THRESHOLD: &[u8] = b"Mask threshold\0";
static PROP_MASK_SOFTNESS: &[u8] = b"Mask softness\0";
static PROP_MASK_INVERT: &[u8] = b"Invert mask\0";
static PROP_BG_DIM: &[u8] = b"Background dim\0";
static PROP_BG_DESAT: &[u8] = b"Background desaturate\0";
static PROP_SHAPE_TYPE: &[u8] = b"Shape\0";
static PROP_CORNER_RADIUS: &[u8] = b"Corner radius\0";
static PROP_FEATHER: &[u8] = b"Feather\0";
static PROP_BORDER_THICKNESS: &[u8] = b"Border thickness\0";
static PROP_BORDER_COLOR: &[u8] = b"Border color\0";
static PROP_PADDING: &[u8] = b"Padding\0";
static PROP_SHADOW_OPACITY: &[u8] = b"Shadow opacity\0";
static PROP_SHADOW_BLUR: &[u8] = b"Shadow blur\0";
static PROP_SHADOW_OFFSET_X: &[u8] = b"Shadow offset X\0";
static PROP_SHADOW_OFFSET_Y: &[u8] = b"Shadow offset Y\0";
static PROP_SHADOW_COLOR: &[u8] = b"Shadow color\0";

static GROUP_SEGMENTATION: &[u8] = b"group_segmentation\0";
static GROUP_BACKGROUND: &[u8] = b"group_background\0";
static GROUP_SHAPE: &[u8] = b"group_shape\0";
static GROUP_BORDER: &[u8] = b"group_border\0";
static GROUP_SHADOW: &[u8] = b"group_shadow\0";
static GROUP_DEBUG: &[u8] = b"group_debug\0";

static GROUP_LABEL_SEGMENTATION: &[u8] = b"Segmentation\0";
static GROUP_LABEL_BACKGROUND: &[u8] = b"Background\0";
static GROUP_LABEL_SHAPE: &[u8] = b"Shape\0";
static GROUP_LABEL_BORDER: &[u8] = b"Border\0";
static GROUP_LABEL_SHADOW: &[u8] = b"Shadow\0";
static GROUP_LABEL_DEBUG: &[u8] = b"Debug\0";

static EFFECT_BLUR_DOWNSAMPLE: &[u8] = b"blur_downsample.effect\0";
static EFFECT_BLUR_PASS: &[u8] = b"blur_pass.effect\0";
static EFFECT_COMPOSITE: &[u8] = b"styled_composite.effect\0";
static EFFECT_SHAPE_STYLE: &[u8] = b"shape_style.effect\0";

static TECH_DOWNSAMPLE: &[u8] = b"Downsample\0";
static TECH_BLUR_H: &[u8] = b"BlurH\0";
static TECH_BLUR_V: &[u8] = b"BlurV\0";
static TECH_COMPOSITE: &[u8] = b"Composite\0";
static TECH_SHAPE_STYLE: &[u8] = b"ShapeStyle\0";

#[repr(C)]
struct StyledCameraFilter {
    source: *mut obs::obs_source_t,
    blur_intensity: f32,
    debug_show_mask: bool,

    mask_fps: u32,
    mask_temporal_smoothing: f32,
    mask_threshold: f32,
    mask_softness: f32,
    mask_invert: bool,
    bg_dim: f32,
    bg_desat: f32,

    shape_type: i32,
    corner_radius: f32,
    feather: f32,
    border_thickness: f32,
    border_color_argb: u32,
    padding: f32,

    shadow_opacity: f32,
    shadow_blur: f32,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    shadow_color_argb: u32,

    effect_downsample: *mut obs::gs_effect_t,
    effect_blur: *mut obs::gs_effect_t,
    effect_composite: *mut obs::gs_effect_t,
    effect_shape: *mut obs::gs_effect_t,

    downsample_image: *mut obs::gs_eparam_t,
    downsample_texel_size: *mut obs::gs_eparam_t,

    blur_image: *mut obs::gs_eparam_t,
    blur_texel_size: *mut obs::gs_eparam_t,
    blur_radius: *mut obs::gs_eparam_t,

    composite_image: *mut obs::gs_eparam_t,
    composite_blur_image: *mut obs::gs_eparam_t,
    composite_mask_image: *mut obs::gs_eparam_t,
    composite_mask_threshold: *mut obs::gs_eparam_t,
    composite_mask_softness: *mut obs::gs_eparam_t,
    composite_mask_invert: *mut obs::gs_eparam_t,
    composite_bg_dim: *mut obs::gs_eparam_t,
    composite_bg_desat: *mut obs::gs_eparam_t,

    shape_image: *mut obs::gs_eparam_t,
    shape_size: *mut obs::gs_eparam_t,
    shape_type_param: *mut obs::gs_eparam_t,
    shape_corner_radius: *mut obs::gs_eparam_t,
    shape_feather: *mut obs::gs_eparam_t,
    shape_border_thickness: *mut obs::gs_eparam_t,
    shape_border_color: *mut obs::gs_eparam_t,
    shape_inset: *mut obs::gs_eparam_t,
    shape_shadow_offset: *mut obs::gs_eparam_t,
    shape_shadow_blur: *mut obs::gs_eparam_t,
    shape_shadow_color: *mut obs::gs_eparam_t,

    tex_full: *mut obs::gs_texrender_t,
    tex_down: *mut obs::gs_texrender_t,
    tex_ping: *mut obs::gs_texrender_t,
    tex_pong: *mut obs::gs_texrender_t,
    tex_comp: *mut obs::gs_texrender_t,
    tex_seg: *mut obs::gs_texrender_t,

    mask_tex: *mut obs::gs_texture_t,
    mask_w: u32,
    mask_h: u32,
    stage_seg: *mut obs::gs_stagesurf_t,
    last_mask_request: Option<Instant>,

    seg_tx: Option<SyncSender<Option<SegInput>>>,
    seg_rx: Option<Receiver<SegOutput>>,
    seg_thread: Option<thread::JoinHandle<()>>,
}

unsafe fn read_settings(filter: &mut StyledCameraFilter, settings: *mut obs::obs_data_t) {
    if settings.is_null() {
        return;
    }

    filter.blur_intensity = obs::obs_data_get_double(settings, cstr(SETTING_BLUR_INTENSITY)) as f32;
    filter.debug_show_mask = obs::obs_data_get_bool(settings, cstr(SETTING_DEBUG_SHOW_MASK));

    filter.mask_fps = obs::obs_data_get_int(settings, cstr(SETTING_MASK_FPS)).max(1) as u32;
    filter.mask_temporal_smoothing =
        obs::obs_data_get_double(settings, cstr(SETTING_MASK_TEMPORAL)) as f32;
    filter.mask_threshold = obs::obs_data_get_double(settings, cstr(SETTING_MASK_THRESHOLD)) as f32;
    filter.mask_softness = obs::obs_data_get_double(settings, cstr(SETTING_MASK_SOFTNESS)) as f32;
    filter.mask_invert = obs::obs_data_get_bool(settings, cstr(SETTING_MASK_INVERT));
    filter.bg_dim = obs::obs_data_get_double(settings, cstr(SETTING_BG_DIM)) as f32;
    filter.bg_desat = obs::obs_data_get_double(settings, cstr(SETTING_BG_DESAT)) as f32;

    filter.shape_type = obs::obs_data_get_int(settings, cstr(SETTING_SHAPE_TYPE)) as i32;
    filter.corner_radius = obs::obs_data_get_double(settings, cstr(SETTING_CORNER_RADIUS)) as f32;
    filter.feather = obs::obs_data_get_double(settings, cstr(SETTING_FEATHER)) as f32;
    filter.border_thickness = obs::obs_data_get_double(settings, cstr(SETTING_BORDER_THICKNESS)) as f32;
    filter.border_color_argb = obs::obs_data_get_int(settings, cstr(SETTING_BORDER_COLOR)) as u32;
    filter.padding = obs::obs_data_get_double(settings, cstr(SETTING_PADDING)) as f32;

    filter.shadow_opacity = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OPACITY)) as f32;
    filter.shadow_blur = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_BLUR)) as f32;
    filter.shadow_offset_x = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OFFSET_X)) as f32;
    filter.shadow_offset_y = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OFFSET_Y)) as f32;
    filter.shadow_color_argb = obs::obs_data_get_int(settings, cstr(SETTING_SHADOW_COLOR)) as u32;
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_ver() -> u32 {
    (obs::LIBOBS_API_MAJOR_VER << 24) | (obs::LIBOBS_API_MINOR_VER << 16) | obs::LIBOBS_API_PATCH_VER
}

static mut OBS_MODULE_PTR: *mut obs::obs_module_t = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn obs_module_set_pointer(module: *mut obs::obs_module_t) {
    OBS_MODULE_PTR = module;
}

#[no_mangle]
pub unsafe extern "C" fn obs_current_module() -> *mut obs::obs_module_t {
    OBS_MODULE_PTR
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_name() -> *const c_char {
    MODULE_NAME.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_description() -> *const c_char {
    MODULE_DESCRIPTION.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_author() -> *const c_char {
    MODULE_AUTHOR.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_load() -> bool {
    register_sources();
    true
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_unload() {}

unsafe fn register_sources() {
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
    settings: *mut obs::obs_data_t,
    source: *mut obs::obs_source_t,
) -> *mut c_void {
    let mut filter = Box::new(StyledCameraFilter {
        source,
        blur_intensity: 0.0,
        debug_show_mask: false,

        mask_fps: 15,
        mask_temporal_smoothing: 0.85,
        mask_threshold: 0.5,
        mask_softness: 0.1,
        mask_invert: false,
        bg_dim: 0.0,
        bg_desat: 0.0,

        shape_type: 0,
        corner_radius: 24.0,
        feather: 1.5,
        border_thickness: 0.0,
        border_color_argb: 0xFFFFFFFF,
        padding: 0.0,

        shadow_opacity: 0.0,
        shadow_blur: 18.0,
        shadow_offset_x: 0.0,
        shadow_offset_y: -4.0,
        shadow_color_argb: 0xFF000000,

        effect_downsample: std::ptr::null_mut(),
        effect_blur: std::ptr::null_mut(),
        effect_composite: std::ptr::null_mut(),
        effect_shape: std::ptr::null_mut(),

        downsample_image: std::ptr::null_mut(),
        downsample_texel_size: std::ptr::null_mut(),

        blur_image: std::ptr::null_mut(),
        blur_texel_size: std::ptr::null_mut(),
        blur_radius: std::ptr::null_mut(),

        composite_image: std::ptr::null_mut(),
        composite_blur_image: std::ptr::null_mut(),
        composite_mask_image: std::ptr::null_mut(),
        composite_mask_threshold: std::ptr::null_mut(),
        composite_mask_softness: std::ptr::null_mut(),
        composite_mask_invert: std::ptr::null_mut(),
        composite_bg_dim: std::ptr::null_mut(),
        composite_bg_desat: std::ptr::null_mut(),

        shape_image: std::ptr::null_mut(),
        shape_size: std::ptr::null_mut(),
        shape_type_param: std::ptr::null_mut(),
        shape_corner_radius: std::ptr::null_mut(),
        shape_feather: std::ptr::null_mut(),
        shape_border_thickness: std::ptr::null_mut(),
        shape_border_color: std::ptr::null_mut(),
        shape_inset: std::ptr::null_mut(),
        shape_shadow_offset: std::ptr::null_mut(),
        shape_shadow_blur: std::ptr::null_mut(),
        shape_shadow_color: std::ptr::null_mut(),

        tex_full: std::ptr::null_mut(),
        tex_down: std::ptr::null_mut(),
        tex_ping: std::ptr::null_mut(),
        tex_pong: std::ptr::null_mut(),
        tex_comp: std::ptr::null_mut(),
        tex_seg: std::ptr::null_mut(),

        mask_tex: std::ptr::null_mut(),
        mask_w: 0,
        mask_h: 0,
        stage_seg: std::ptr::null_mut(),
        last_mask_request: None,

        seg_tx: None,
        seg_rx: None,
        seg_thread: None,
    });
    read_settings(&mut filter, settings);

    init_graphics(&mut filter);
    ensure_segmentation_thread(&mut filter);
    Box::into_raw(filter).cast()
}

unsafe extern "C" fn styled_camera_filter_destroy(data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let mut filter = Box::from_raw(data.cast::<StyledCameraFilter>());
    stop_segmentation_thread(&mut filter);
    destroy_graphics(&mut filter);
}

unsafe extern "C" fn styled_camera_filter_update(
    data: *mut c_void,
    settings: *mut obs::obs_data_t,
) {
    if data.is_null() {
        return;
    }
    let filter = &mut *data.cast::<StyledCameraFilter>();
    read_settings(filter, settings);
}

unsafe extern "C" fn styled_camera_filter_get_defaults(settings: *mut obs::obs_data_t) {
    if settings.is_null() {
        return;
    }
    obs::obs_data_set_default_double(settings, cstr(SETTING_BLUR_INTENSITY), 0.0);
    obs::obs_data_set_default_bool(settings, cstr(SETTING_DEBUG_SHOW_MASK), false);

    obs::obs_data_set_default_int(settings, cstr(SETTING_MASK_FPS), 15);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_TEMPORAL), 0.85);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_THRESHOLD), 0.5);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_SOFTNESS), 0.1);
    obs::obs_data_set_default_bool(settings, cstr(SETTING_MASK_INVERT), false);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BG_DIM), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BG_DESAT), 0.0);

    obs::obs_data_set_default_int(settings, cstr(SETTING_SHAPE_TYPE), 0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_CORNER_RADIUS), 24.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FEATHER), 1.5);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BORDER_THICKNESS), 0.0);
    obs::obs_data_set_default_int(settings, cstr(SETTING_BORDER_COLOR), 0xFFFFFFFFu32 as i64);
    obs::obs_data_set_default_double(settings, cstr(SETTING_PADDING), 0.0);

    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OPACITY), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_BLUR), 18.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OFFSET_X), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OFFSET_Y), -4.0);
    obs::obs_data_set_default_int(settings, cstr(SETTING_SHADOW_COLOR), 0xFF000000u32 as i64);
}

unsafe extern "C" fn styled_camera_filter_get_properties(
    _data: *mut c_void,
) -> *mut obs::obs_properties_t {
    let props = obs::obs_properties_create();
    if props.is_null() {
        return props;
    }

    // Segmentation
    let seg_props = obs::obs_properties_create();
    if !seg_props.is_null() {
        obs::obs_properties_add_int_slider(
            seg_props,
            cstr(SETTING_MASK_FPS),
            cstr(PROP_MASK_FPS),
            1,
            30,
            1,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_TEMPORAL),
            cstr(PROP_MASK_TEMPORAL),
            0.0,
            0.95,
            0.01,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_THRESHOLD),
            cstr(PROP_MASK_THRESHOLD),
            0.0,
            1.0,
            0.01,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_SOFTNESS),
            cstr(PROP_MASK_SOFTNESS),
            0.0,
            0.5,
            0.01,
        );
        obs::obs_properties_add_bool(seg_props, cstr(SETTING_MASK_INVERT), cstr(PROP_MASK_INVERT));

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_SEGMENTATION),
            cstr(GROUP_LABEL_SEGMENTATION),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            seg_props,
        );
    }

    // Background (blur, dim, desat)
    let bg_props = obs::obs_properties_create();
    if !bg_props.is_null() {
        obs::obs_properties_add_float_slider(
            bg_props,
            cstr(SETTING_BLUR_INTENSITY),
            cstr(PROP_BLUR_INTENSITY),
            0.0,
            1.0,
            0.01,
        );
        obs::obs_properties_add_float_slider(
            bg_props,
            cstr(SETTING_BG_DIM),
            cstr(PROP_BG_DIM),
            0.0,
            1.0,
            0.01,
        );
        obs::obs_properties_add_float_slider(
            bg_props,
            cstr(SETTING_BG_DESAT),
            cstr(PROP_BG_DESAT),
            0.0,
            1.0,
            0.01,
        );

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_BACKGROUND),
            cstr(GROUP_LABEL_BACKGROUND),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            bg_props,
        );
    }

    // Shape (type + edge + padding)
    let shape_props = obs::obs_properties_create();
    if !shape_props.is_null() {
        let shape_list = obs::obs_properties_add_list(
            shape_props,
            cstr(SETTING_SHAPE_TYPE),
            cstr(PROP_SHAPE_TYPE),
            obs::obs_combo_type_OBS_COMBO_TYPE_LIST,
            obs::obs_combo_format_OBS_COMBO_FORMAT_INT,
        );
        if !shape_list.is_null() {
            obs::obs_property_list_add_int(shape_list, cstr(b"Circle\0"), 0);
            obs::obs_property_list_add_int(shape_list, cstr(b"Rectangle\0"), 1);
            obs::obs_property_list_add_int(shape_list, cstr(b"Rounded rectangle\0"), 2);
            obs::obs_property_list_add_int(shape_list, cstr(b"Square\0"), 3);
        }

        obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_CORNER_RADIUS),
            cstr(PROP_CORNER_RADIUS),
            0.0,
            2000.0,
            1.0,
        );
        obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FEATHER),
            cstr(PROP_FEATHER),
            0.0,
            32.0,
            0.1,
        );
        obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_PADDING),
            cstr(PROP_PADDING),
            0.0,
            500.0,
            1.0,
        );

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_SHAPE),
            cstr(GROUP_LABEL_SHAPE),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            shape_props,
        );
    }

    // Border
    let border_props = obs::obs_properties_create();
    if !border_props.is_null() {
        obs::obs_properties_add_float_slider(
            border_props,
            cstr(SETTING_BORDER_THICKNESS),
            cstr(PROP_BORDER_THICKNESS),
            0.0,
            32.0,
            0.5,
        );
        obs::obs_properties_add_color_alpha(
            border_props,
            cstr(SETTING_BORDER_COLOR),
            cstr(PROP_BORDER_COLOR),
        );

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_BORDER),
            cstr(GROUP_LABEL_BORDER),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            border_props,
        );
    }

    // Shadow
    let shadow_props = obs::obs_properties_create();
    if !shadow_props.is_null() {
        obs::obs_properties_add_float_slider(
            shadow_props,
            cstr(SETTING_SHADOW_OPACITY),
            cstr(PROP_SHADOW_OPACITY),
            0.0,
            1.0,
            0.01,
        );
        obs::obs_properties_add_float_slider(
            shadow_props,
            cstr(SETTING_SHADOW_BLUR),
            cstr(PROP_SHADOW_BLUR),
            0.0,
            64.0,
            0.5,
        );
        obs::obs_properties_add_float_slider(
            shadow_props,
            cstr(SETTING_SHADOW_OFFSET_X),
            cstr(PROP_SHADOW_OFFSET_X),
            -200.0,
            200.0,
            1.0,
        );
        obs::obs_properties_add_float_slider(
            shadow_props,
            cstr(SETTING_SHADOW_OFFSET_Y),
            cstr(PROP_SHADOW_OFFSET_Y),
            -200.0,
            200.0,
            1.0,
        );
        obs::obs_properties_add_color_alpha(
            shadow_props,
            cstr(SETTING_SHADOW_COLOR),
            cstr(PROP_SHADOW_COLOR),
        );

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_SHADOW),
            cstr(GROUP_LABEL_SHADOW),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            shadow_props,
        );
    }

    // Debug
    let debug_props = obs::obs_properties_create();
    if !debug_props.is_null() {
        obs::obs_properties_add_bool(
            debug_props,
            cstr(SETTING_DEBUG_SHOW_MASK),
            cstr(PROP_DEBUG_SHOW_MASK),
        );

        obs::obs_properties_add_group(
            props,
            cstr(GROUP_DEBUG),
            cstr(GROUP_LABEL_DEBUG),
            obs::obs_group_type_OBS_GROUP_NORMAL,
            debug_props,
        );
    }

    props
}

unsafe extern "C" fn styled_camera_filter_video_render(
    data: *mut c_void,
    _effect: *mut obs::gs_effect_t,
) {
    if data.is_null() {
        return;
    }
    let filter = &mut *data.cast::<StyledCameraFilter>();

    if !ensure_graphics(filter) {
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

    ensure_segmentation_thread(filter);

    let blur_amount = filter.blur_intensity.clamp(0.0, 1.0);

    let downscale_div: u32 = 2;
    let down_cx = (cx / downscale_div).max(1);
    let down_cy = (cy / downscale_div).max(1);
    let seg_cx: u32 = 256;
    let seg_cy: u32 = 256;

    obs::obs_enter_graphics();

    // Consume latest segmentation output (if any) and update mask texture.
    if let Some(rx) = filter.seg_rx.as_ref() {
        while let Ok(out) = rx.try_recv() {
            if out.width == 0 || out.height == 0 {
                continue;
            }

            if filter.mask_tex.is_null() || filter.mask_w != out.width || filter.mask_h != out.height {
                if !filter.mask_tex.is_null() {
                    obs::gs_texture_destroy(filter.mask_tex);
                    filter.mask_tex = std::ptr::null_mut();
                }
                let init = vec![255u8; (out.width * out.height) as usize];
                let mut data_ptrs = [init.as_ptr()];
                filter.mask_tex = obs::gs_texture_create(
                    out.width,
                    out.height,
                    obs::gs_color_format_GS_R8,
                    1,
                    data_ptrs.as_mut_ptr(),
                    obs::GS_DYNAMIC,
                );
                filter.mask_w = out.width;
                filter.mask_h = out.height;
            }

            if !filter.mask_tex.is_null() && out.mask.len() == (out.width * out.height) as usize {
                obs::gs_texture_set_image(filter.mask_tex, out.mask.as_ptr(), out.width, false);
            }
        }
    }

    // Render source into full-size texture.
    if render_source_to_texrender(filter.tex_full, cx, cy, target) {
        let tex_full = obs::gs_texrender_get_texture(filter.tex_full);
        if !tex_full.is_null() {
            // Segmentation input (fixed 256x256) + async inference request.
            if render_effect_to_texrender(
                filter.tex_seg,
                seg_cx,
                seg_cy,
                filter.effect_downsample,
                TECH_DOWNSAMPLE,
                || {
                    set_vec2_param(filter.downsample_texel_size, 1.0 / (cx as f32), 1.0 / (cy as f32));
                    if !filter.downsample_image.is_null() {
                        obs::gs_effect_set_texture(filter.downsample_image, tex_full);
                    }
                },
                tex_full,
            ) {
                let now = Instant::now();
                let interval = Duration::from_secs_f32(1.0 / (filter.mask_fps.max(1) as f32));
                let due = filter
                    .last_mask_request
                    .map(|t| now.duration_since(t) >= interval)
                    .unwrap_or(true);

                if due {
                    if let Some(tx) = filter.seg_tx.as_ref() {
                        let tex_seg = obs::gs_texrender_get_texture(filter.tex_seg);
                        if !tex_seg.is_null() && !filter.stage_seg.is_null() {
                            obs::gs_stage_texture(filter.stage_seg, tex_seg);
                            let mut data: *mut u8 = std::ptr::null_mut();
                            let mut linesize: u32 = 0;
                            if obs::gs_stagesurface_map(filter.stage_seg, &mut data, &mut linesize) {
                                let mut rgba = vec![0u8; (seg_cx * seg_cy * 4) as usize];
                                for y in 0..seg_cy {
                                    let src_row = data.add((y * linesize) as usize);
                                    let dst_row = &mut rgba[(y * seg_cx * 4) as usize..][..(seg_cx * 4) as usize];
                                    std::ptr::copy_nonoverlapping(src_row, dst_row.as_mut_ptr(), dst_row.len());
                                }
                                obs::gs_stagesurface_unmap(filter.stage_seg);

                                let _ = tx.try_send(Some(SegInput {
                                    rgba,
                                    width: seg_cx,
                                    height: seg_cy,
                                    temporal_smoothing: filter.mask_temporal_smoothing,
                                }));
                                filter.last_mask_request = Some(now);
                            }
                        }
                    }
                }
            }

            // Blur (downsample -> blur passes). If blur_amount == 0, we keep blur_tex = sharp.
            let mut blur_tex = tex_full;
            if blur_amount > 0.0001
                && render_effect_to_texrender(
                    filter.tex_down,
                    down_cx,
                    down_cy,
                    filter.effect_downsample,
                    TECH_DOWNSAMPLE,
                    || {
                        set_vec2_param(filter.downsample_texel_size, 1.0 / (cx as f32), 1.0 / (cy as f32));
                        if !filter.downsample_image.is_null() {
                            obs::gs_effect_set_texture(filter.downsample_image, tex_full);
                        }
                    },
                    tex_full,
                )
            {
                let tex_down = obs::gs_texrender_get_texture(filter.tex_down);
                if !tex_down.is_null() {
                    let blur_radius = (blur_amount * 24.0).max(0.0);
                    let texel_x = 1.0 / (down_cx as f32);
                    let texel_y = 1.0 / (down_cy as f32);

                    let ok_h = render_effect_to_texrender(
                        filter.tex_ping,
                        down_cx,
                        down_cy,
                        filter.effect_blur,
                        TECH_BLUR_H,
                        || {
                            set_vec2_param(filter.blur_texel_size, texel_x, texel_y);
                            set_float_param(filter.blur_radius, blur_radius);
                            if !filter.blur_image.is_null() {
                                obs::gs_effect_set_texture(filter.blur_image, tex_down);
                            }
                        },
                        tex_down,
                    );

                    let tex_ping = obs::gs_texrender_get_texture(filter.tex_ping);
                    let ok_v = ok_h
                        && !tex_ping.is_null()
                        && render_effect_to_texrender(
                            filter.tex_pong,
                            down_cx,
                            down_cy,
                            filter.effect_blur,
                            TECH_BLUR_V,
                            || {
                                set_vec2_param(filter.blur_texel_size, texel_x, texel_y);
                                set_float_param(filter.blur_radius, blur_radius);
                                if !filter.blur_image.is_null() {
                                    obs::gs_effect_set_texture(filter.blur_image, tex_ping);
                                }
                            },
                            tex_ping,
                        );

                    if ok_v {
                        let t = obs::gs_texrender_get_texture(filter.tex_pong);
                        if !t.is_null() {
                            blur_tex = t;
                        }
                    }
                }
            }

            // Composite sharp + blurred background using mask texture.
            let ok_comp = render_effect_to_texrender(
                filter.tex_comp,
                cx,
                cy,
                filter.effect_composite,
                TECH_COMPOSITE,
                || {
                    if !filter.composite_image.is_null() {
                        obs::gs_effect_set_texture(filter.composite_image, tex_full);
                    }
                    if !filter.composite_blur_image.is_null() {
                        obs::gs_effect_set_texture(filter.composite_blur_image, blur_tex);
                    }
                    if !filter.composite_mask_image.is_null() {
                        obs::gs_effect_set_texture(filter.composite_mask_image, filter.mask_tex);
                    }

                    set_float_param(filter.composite_mask_threshold, filter.mask_threshold.clamp(0.0, 1.0));
                    set_float_param(filter.composite_mask_softness, filter.mask_softness.clamp(0.0, 1.0));
                    set_float_param(filter.composite_mask_invert, if filter.mask_invert { 1.0 } else { 0.0 });
                    set_float_param(filter.composite_bg_dim, filter.bg_dim.clamp(0.0, 1.0));
                    set_float_param(filter.composite_bg_desat, filter.bg_desat.clamp(0.0, 1.0));
                },
                tex_full,
            );

            let tex_comp = obs::gs_texrender_get_texture(filter.tex_comp);
            if ok_comp && !tex_comp.is_null() {
                if filter.debug_show_mask && !filter.mask_tex.is_null() {
                    let effect = obs::obs_get_base_effect(obs::obs_base_effect_OBS_EFFECT_DEFAULT);
                    let image_param = obs::gs_effect_get_param_by_name(effect, cstr(b"image\0"));
                    obs::gs_effect_set_texture(image_param, filter.mask_tex);
                    while obs::gs_effect_loop(effect, cstr(b"Draw\0")) {
                        obs::gs_draw_sprite(filter.mask_tex, 0, cx, cy);
                    }
                    obs::obs_leave_graphics();
                    return;
                }

                draw_shape_to_screen(filter, tex_comp, cx, cy);
                obs::obs_leave_graphics();
                return;
            }
        }
    }

    obs::obs_leave_graphics();
    obs::obs_source_skip_video_filter(filter.source);
}

fn cstr(bytes: &'static [u8]) -> *const c_char {
    debug_assert!(bytes.last() == Some(&0), "C string must be NUL-terminated");
    bytes.as_ptr().cast()
}

struct SegInput {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    temporal_smoothing: f32,
}

struct SegOutput {
    mask: Vec<u8>,
    width: u32,
    height: u32,
}

unsafe fn ensure_segmentation_thread(filter: &mut StyledCameraFilter) {
    if filter.seg_tx.is_some() {
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

    let (in_tx, in_rx) = mpsc::sync_channel::<Option<SegInput>>(1);
    let (out_tx, out_rx) = mpsc::sync_channel::<SegOutput>(1);

    let handle = thread::spawn(move || segmentation_thread_main(in_rx, out_tx, dylib_path, model_path));

    filter.seg_tx = Some(in_tx);
    filter.seg_rx = Some(out_rx);
    filter.seg_thread = Some(handle);
}

unsafe fn stop_segmentation_thread(filter: &mut StyledCameraFilter) {
    if let Some(tx) = filter.seg_tx.take() {
        let _ = tx.try_send(None);
    }
    filter.seg_rx.take();
    if let Some(handle) = filter.seg_thread.take() {
        let _ = handle.join();
    }
}

fn segmentation_thread_main(
    rx: Receiver<Option<SegInput>>,
    tx: SyncSender<SegOutput>,
    dylib_path: PathBuf,
    model_path: PathBuf,
) {
    if let Err(err) = ort::init_from(&dylib_path).and_then(|b| Ok(b.commit())) {
        let _ = err;
        return;
    }

    let mut session = match ort::session::Session::builder()
        .and_then(|b| b.commit_from_file(model_path))
    {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut prev_mask: Vec<f32> = Vec::new();

    while let Ok(msg) = rx.recv() {
        let Some(input) = msg else { break };

        let w = input.width as usize;
        let h = input.height as usize;
        if w == 0 || h == 0 || input.rgba.len() != w * h * 4 {
            continue;
        }

        let mut rgb = Vec::<f32>::with_capacity(w * h * 3);
        for px in input.rgba.chunks_exact(4) {
            rgb.push(px[0] as f32 / 255.0);
            rgb.push(px[1] as f32 / 255.0);
            rgb.push(px[2] as f32 / 255.0);
        }

        let shape = vec![1i64, input.height as i64, input.width as i64, 3];
        let tensor = match ort::value::Tensor::<f32>::from_array((shape, rgb.into_boxed_slice())) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let outputs = match session.run(ort::inputs![tensor]) {
            Ok(o) => o,
            Err(_) => continue,
        };

        let (out_shape, out_data) = match outputs[0].try_extract_tensor::<f32>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let expected = (input.width * input.height) as usize;
        let current: &[f32] = if out_data.len() == expected {
            out_data
        } else if out_data.len() == expected * 1 {
            &out_data[..expected]
        } else if out_data.len() >= expected {
            &out_data[..expected]
        } else {
            let _ = out_shape;
            continue;
        };

        if prev_mask.len() != expected {
            prev_mask.clear();
            prev_mask.extend_from_slice(current);
        } else {
            let a = input.temporal_smoothing.clamp(0.0, 0.99);
            let b = 1.0 - a;
            for (p, &c) in prev_mask.iter_mut().zip(current.iter()) {
                *p = (*p * a) + (c * b);
            }
        }

        let mut mask_u8 = vec![0u8; expected];
        for (dst, &v) in mask_u8.iter_mut().zip(prev_mask.iter()) {
            *dst = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        }

        let _ = tx.try_send(SegOutput {
            mask: mask_u8,
            width: input.width,
            height: input.height,
        });
    }
}

unsafe fn resolve_onnxruntime_dylib_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ONNXRUNTIME_DYLIB") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    let module = obs_current_module();
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
    let module = obs_current_module();
    if module.is_null() {
        return None;
    }

    let p = obs::obs_find_module_file(module, cstr(b"models/selfie_segmentation.onnx\0"));
    if p.is_null() {
        return None;
    }
    let s = CStr::from_ptr(p).to_string_lossy().to_string();
    obs::bfree(p.cast());
    let pb = PathBuf::from(s);
    if pb.is_file() {
        Some(pb)
    } else {
        None
    }
}

unsafe fn ensure_graphics(filter: &mut StyledCameraFilter) -> bool {
    if !filter.effect_downsample.is_null()
        && !filter.effect_blur.is_null()
        && !filter.effect_composite.is_null()
        && !filter.effect_shape.is_null()
        && !filter.tex_full.is_null()
        && !filter.tex_down.is_null()
        && !filter.tex_ping.is_null()
        && !filter.tex_pong.is_null()
        && !filter.tex_comp.is_null()
        && !filter.tex_seg.is_null()
        && !filter.mask_tex.is_null()
        && !filter.stage_seg.is_null()
    {
        return true;
    }

    init_graphics(filter);
    !filter.effect_downsample.is_null()
        && !filter.effect_blur.is_null()
        && !filter.effect_composite.is_null()
        && !filter.effect_shape.is_null()
        && !filter.tex_full.is_null()
        && !filter.tex_down.is_null()
        && !filter.tex_ping.is_null()
        && !filter.tex_pong.is_null()
        && !filter.tex_comp.is_null()
        && !filter.tex_seg.is_null()
        && !filter.mask_tex.is_null()
        && !filter.stage_seg.is_null()
}

unsafe fn init_graphics(filter: &mut StyledCameraFilter) {
    obs::obs_enter_graphics();

    if filter.effect_downsample.is_null() {
        filter.effect_downsample = load_effect(EFFECT_BLUR_DOWNSAMPLE);
        if !filter.effect_downsample.is_null() {
            filter.downsample_image =
                obs::gs_effect_get_param_by_name(filter.effect_downsample, cstr(b"image\0"));
            filter.downsample_texel_size = obs::gs_effect_get_param_by_name(
                filter.effect_downsample,
                cstr(b"texel_size\0"),
            );
        }
    }

    if filter.effect_blur.is_null() {
        filter.effect_blur = load_effect(EFFECT_BLUR_PASS);
        if !filter.effect_blur.is_null() {
            filter.blur_image = obs::gs_effect_get_param_by_name(filter.effect_blur, cstr(b"image\0"));
            filter.blur_texel_size =
                obs::gs_effect_get_param_by_name(filter.effect_blur, cstr(b"texel_size\0"));
            filter.blur_radius =
                obs::gs_effect_get_param_by_name(filter.effect_blur, cstr(b"blur_radius\0"));
        }
    }

    if filter.effect_composite.is_null() {
        filter.effect_composite = load_effect(EFFECT_COMPOSITE);
        if !filter.effect_composite.is_null() {
            filter.composite_image =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"image\0"));
            filter.composite_blur_image =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"blur_image\0"));
            filter.composite_mask_image =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"mask_image\0"));
            filter.composite_mask_threshold =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"mask_threshold\0"));
            filter.composite_mask_softness =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"mask_softness\0"));
            filter.composite_mask_invert =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"mask_invert\0"));
            filter.composite_bg_dim =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"bg_dim\0"));
            filter.composite_bg_desat =
                obs::gs_effect_get_param_by_name(filter.effect_composite, cstr(b"bg_desat\0"));
        }
    }

    if filter.effect_shape.is_null() {
        filter.effect_shape = load_effect(EFFECT_SHAPE_STYLE);
        if !filter.effect_shape.is_null() {
            filter.shape_image =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"image\0"));
            filter.shape_size = obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"size\0"));
            filter.shape_type_param =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"shape_type\0"));
            filter.shape_corner_radius =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"corner_radius\0"));
            filter.shape_feather =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"feather\0"));
            filter.shape_border_thickness =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"border_thickness\0"));
            filter.shape_border_color =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"border_color\0"));
            filter.shape_inset = obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"inset\0"));
            filter.shape_shadow_offset =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"shadow_offset\0"));
            filter.shape_shadow_blur =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"shadow_blur\0"));
            filter.shape_shadow_color =
                obs::gs_effect_get_param_by_name(filter.effect_shape, cstr(b"shadow_color\0"));
        }
    }

    if filter.tex_full.is_null() {
        filter.tex_full = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }
    if filter.tex_down.is_null() {
        filter.tex_down = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }
    if filter.tex_ping.is_null() {
        filter.tex_ping = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }
    if filter.tex_pong.is_null() {
        filter.tex_pong = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }
    if filter.tex_comp.is_null() {
        filter.tex_comp = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }
    if filter.tex_seg.is_null() {
        filter.tex_seg = obs::gs_texrender_create(obs::gs_color_format_GS_RGBA, obs::gs_zstencil_format_GS_ZS_NONE);
    }

    if filter.mask_tex.is_null() {
        let mask_w = 256u32;
        let mask_h = 256u32;
        let init = vec![255u8; (mask_w * mask_h) as usize];
        let mut data_ptrs = [init.as_ptr()];
        filter.mask_tex = obs::gs_texture_create(
            mask_w,
            mask_h,
            obs::gs_color_format_GS_R8,
            1,
            data_ptrs.as_mut_ptr(),
            obs::GS_DYNAMIC,
        );
        filter.mask_w = mask_w;
        filter.mask_h = mask_h;
    }

    if filter.stage_seg.is_null() {
        filter.stage_seg = obs::gs_stagesurface_create(256, 256, obs::gs_color_format_GS_RGBA);
    }

    obs::obs_leave_graphics();
}

unsafe fn destroy_graphics(filter: &mut StyledCameraFilter) {
    obs::obs_enter_graphics();

    if !filter.mask_tex.is_null() {
        obs::gs_texture_destroy(filter.mask_tex);
        filter.mask_tex = std::ptr::null_mut();
    }
    filter.mask_w = 0;
    filter.mask_h = 0;

    if !filter.stage_seg.is_null() {
        obs::gs_stagesurface_destroy(filter.stage_seg);
        filter.stage_seg = std::ptr::null_mut();
    }

    if !filter.tex_full.is_null() {
        obs::gs_texrender_destroy(filter.tex_full);
        filter.tex_full = std::ptr::null_mut();
    }
    if !filter.tex_down.is_null() {
        obs::gs_texrender_destroy(filter.tex_down);
        filter.tex_down = std::ptr::null_mut();
    }
    if !filter.tex_ping.is_null() {
        obs::gs_texrender_destroy(filter.tex_ping);
        filter.tex_ping = std::ptr::null_mut();
    }
    if !filter.tex_pong.is_null() {
        obs::gs_texrender_destroy(filter.tex_pong);
        filter.tex_pong = std::ptr::null_mut();
    }
    if !filter.tex_comp.is_null() {
        obs::gs_texrender_destroy(filter.tex_comp);
        filter.tex_comp = std::ptr::null_mut();
    }
    if !filter.tex_seg.is_null() {
        obs::gs_texrender_destroy(filter.tex_seg);
        filter.tex_seg = std::ptr::null_mut();
    }

    if !filter.effect_downsample.is_null() {
        obs::gs_effect_destroy(filter.effect_downsample);
        filter.effect_downsample = std::ptr::null_mut();
    }
    if !filter.effect_blur.is_null() {
        obs::gs_effect_destroy(filter.effect_blur);
        filter.effect_blur = std::ptr::null_mut();
    }
    if !filter.effect_composite.is_null() {
        obs::gs_effect_destroy(filter.effect_composite);
        filter.effect_composite = std::ptr::null_mut();
    }
    if !filter.effect_shape.is_null() {
        obs::gs_effect_destroy(filter.effect_shape);
        filter.effect_shape = std::ptr::null_mut();
    }

    obs::obs_leave_graphics();
}

unsafe fn load_effect(file: &'static [u8]) -> *mut obs::gs_effect_t {
    // obs_module_file is a macro; use the underlying exported function.
    let module = obs::obs_current_module();
    if module.is_null() {
        obs::blog(obs::LOG_WARNING as i32, cstr(b"StyledCamera: obs_current_module returned NULL\n\0"));
        return std::ptr::null_mut();
    }

    let path = obs::obs_find_module_file(module, cstr(file));
    if path.is_null() {
        obs::blog(obs::LOG_WARNING as i32, cstr(b"StyledCamera: obs_find_module_file returned NULL\n\0"));
        return std::ptr::null_mut();
    }

    let mut error: *mut c_char = std::ptr::null_mut();
    let effect = obs::gs_effect_create_from_file(path, &mut error);
    if !error.is_null() {
        // blog is variadic; this logs the real compiler error text into the OBS log.
        obs::blog(
            obs::LOG_WARNING as i32,
            cstr(b"StyledCamera: effect compile error (%s): %s\n\0"),
            path,
            error,
        );
        obs::bfree(error.cast());
    }

    obs::bfree(path.cast());
    effect
}

unsafe fn render_source_to_texrender(
    texrender: *mut obs::gs_texrender_t,
    cx: u32,
    cy: u32,
    target: *mut obs::obs_source_t,
) -> bool {
    if texrender.is_null() {
        return false;
    }

    obs::gs_texrender_reset(texrender);

    let mut clear_color: obs::vec4 = std::mem::zeroed();

    obs::gs_blend_state_push();
    obs::gs_blend_function(obs::gs_blend_type_GS_BLEND_ONE, obs::gs_blend_type_GS_BLEND_ZERO);

    let ok = if obs::gs_texrender_begin(texrender, cx, cy) {
        obs::gs_clear(obs::GS_CLEAR_COLOR as u32, &mut clear_color, 0.0, 0);
        obs::gs_ortho(0.0, cx as f32, 0.0, cy as f32, -100.0, 100.0);
        obs::obs_source_video_render(target);
        obs::gs_texrender_end(texrender);
        true
    } else {
        false
    };

    obs::gs_blend_state_pop();
    ok
}

unsafe fn render_effect_to_texrender(
    texrender: *mut obs::gs_texrender_t,
    cx: u32,
    cy: u32,
    effect: *mut obs::gs_effect_t,
    technique: &'static [u8],
    set_params: impl FnOnce(),
    draw_tex: *mut obs::gs_texture_t,
) -> bool {
    if texrender.is_null() || effect.is_null() {
        return false;
    }

    obs::gs_texrender_reset(texrender);

    let mut clear_color: obs::vec4 = std::mem::zeroed();

    obs::gs_blend_state_push();
    obs::gs_blend_function(obs::gs_blend_type_GS_BLEND_ONE, obs::gs_blend_type_GS_BLEND_ZERO);

    let ok = if obs::gs_texrender_begin(texrender, cx, cy) {
        obs::gs_clear(obs::GS_CLEAR_COLOR as u32, &mut clear_color, 0.0, 0);
        obs::gs_ortho(0.0, cx as f32, 0.0, cy as f32, -100.0, 100.0);

        // Set effect params.
        set_params();

        while obs::gs_effect_loop(effect, cstr(technique)) {
            obs::gs_draw_sprite(draw_tex, 0, cx, cy);
        }

        obs::gs_texrender_end(texrender);
        true
    } else {
        false
    };

    obs::gs_blend_state_pop();
    ok
}

unsafe fn draw_shape_to_screen(filter: &StyledCameraFilter, tex: *mut obs::gs_texture_t, cx: u32, cy: u32) {
    if filter.effect_shape.is_null() || tex.is_null() {
        return;
    }

    if !filter.shape_image.is_null() {
        obs::gs_effect_set_texture(filter.shape_image, tex);
    }
    if !filter.shape_size.is_null() {
        set_vec2_param(filter.shape_size, cx as f32, cy as f32);
    }
    if !filter.shape_type_param.is_null() {
        obs::gs_effect_set_float(filter.shape_type_param, filter.shape_type as f32);
    }
    set_float_param(filter.shape_corner_radius, filter.corner_radius);
    set_float_param(filter.shape_feather, filter.feather);
    set_float_param(filter.shape_border_thickness, filter.border_thickness);

    if !filter.shape_border_color.is_null() {
        let border = obs_abgr_to_rgba_vec4(filter.border_color_argb);
        set_vec4_param(filter.shape_border_color, border);
    }

    if !filter.shape_inset.is_null() {
        let p = filter.padding.max(0.0);
        set_vec4_param(filter.shape_inset, [p, p, p, p]);
    }

    if !filter.shape_shadow_offset.is_null() {
        set_vec2_param(filter.shape_shadow_offset, filter.shadow_offset_x, filter.shadow_offset_y);
    }
    set_float_param(filter.shape_shadow_blur, filter.shadow_blur);
    if !filter.shape_shadow_color.is_null() {
        let mut shadow = obs_abgr_to_rgba_vec4(filter.shadow_color_argb);
        shadow[3] *= filter.shadow_opacity.clamp(0.0, 1.0);
        set_vec4_param(filter.shape_shadow_color, shadow);
    }

    obs::gs_blend_state_push();
    obs::gs_blend_function(
        obs::gs_blend_type_GS_BLEND_ONE,
        obs::gs_blend_type_GS_BLEND_INVSRCALPHA,
    );

    while obs::gs_effect_loop(filter.effect_shape, cstr(TECH_SHAPE_STYLE)) {
        obs::gs_draw_sprite(tex, 0, cx, cy);
    }

    obs::gs_blend_state_pop();
}

unsafe fn set_vec2_param(param: *mut obs::gs_eparam_t, x: f32, y: f32) {
    if param.is_null() {
        return;
    }
    let v = [x, y];
    obs::gs_effect_set_val(param, v.as_ptr().cast(), std::mem::size_of_val(&v) as obs::size_t);
}

unsafe fn set_float_param(param: *mut obs::gs_eparam_t, v: f32) {
    if param.is_null() {
        return;
    }
    obs::gs_effect_set_float(param, v);
}

unsafe fn set_vec4_param(param: *mut obs::gs_eparam_t, v: [f32; 4]) {
    if param.is_null() {
        return;
    }
    obs::gs_effect_set_val(param, v.as_ptr().cast(), std::mem::size_of_val(&v) as obs::size_t);
}

unsafe fn obs_abgr_to_rgba_vec4(abgr: u32) -> [f32; 4] {
    // OBS color properties are stored as 0xAABBGGRR (ABGR).
    let a = ((abgr >> 24) & 0xFF) as f32 / 255.0;
    let b = ((abgr >> 16) & 0xFF) as f32 / 255.0;
    let g = ((abgr >> 8) & 0xFF) as f32 / 255.0;
    let r = (abgr & 0xFF) as f32 / 255.0;

    [r, g, b, a]
}
