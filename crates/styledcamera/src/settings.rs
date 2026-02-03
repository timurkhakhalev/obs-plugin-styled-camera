use obs_sys as obs;

use crate::constants::*;
use crate::util::cstr;

unsafe extern "C" fn on_shape_type_modified(
    props: *mut obs::obs_properties_t,
    _property: *mut obs::obs_property_t,
    settings: *mut obs::obs_data_t,
) -> bool {
    if props.is_null() || settings.is_null() {
        return false;
    }

    let shape_type = obs::obs_data_get_int(settings, cstr(SETTING_SHAPE_TYPE));
    // Only Rectangle and Square expose frame bounds controls.
    let show_frame = shape_type == 1 || shape_type == 3;

    let p_l = obs::obs_properties_get(props, cstr(SETTING_FRAME_INSET_LEFT));
    let p_t = obs::obs_properties_get(props, cstr(SETTING_FRAME_INSET_TOP));
    let p_r = obs::obs_properties_get(props, cstr(SETTING_FRAME_INSET_RIGHT));
    let p_b = obs::obs_properties_get(props, cstr(SETTING_FRAME_INSET_BOTTOM));
    if !p_l.is_null() { obs::obs_property_set_visible(p_l, show_frame); }
    if !p_t.is_null() { obs::obs_property_set_visible(p_t, show_frame); }
    if !p_r.is_null() { obs::obs_property_set_visible(p_r, show_frame); }
    if !p_b.is_null() { obs::obs_property_set_visible(p_b, show_frame); }

    // Visibility changes require a refresh.
    true
}

#[derive(Clone, Copy)]
pub(crate) struct FilterSettings {
    pub blur_intensity: f32,
    pub debug_show_mask: bool,

    pub mask_fps: u32,
    pub mask_temporal_smoothing: f32,
    pub mask_threshold: f32,
    pub mask_softness: f32,
    pub mask_invert: bool,
    pub sync_video_extra_delay_ms: f32,
    pub bg_dim: f32,
    pub bg_desat: f32,

    pub shape_type: i32,
    pub corner_radius: f32,
    pub feather: f32,
    pub border_thickness: f32,
    pub border_color_argb: u32,
    pub padding: f32,
    pub frame_inset_left: f32,
    pub frame_inset_top: f32,
    pub frame_inset_right: f32,
    pub frame_inset_bottom: f32,

    pub shadow_opacity: f32,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub shadow_color_argb: u32,
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            blur_intensity: 0.0,
            debug_show_mask: false,

            mask_fps: 15,
            mask_temporal_smoothing: 0.4,
            mask_threshold: 0.5,
            mask_softness: 0.1,
            mask_invert: false,
            sync_video_extra_delay_ms: 0.0,
            bg_dim: 0.0,
            bg_desat: 0.0,

            shape_type: 0,
            corner_radius: 24.0,
            feather: 1.5,
            border_thickness: 0.0,
            border_color_argb: 0xFFFFFFFF,
            padding: 0.0,
            frame_inset_left: 0.0,
            frame_inset_top: 0.0,
            frame_inset_right: 0.0,
            frame_inset_bottom: 0.0,

            shadow_opacity: 0.0,
            shadow_blur: 18.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: -4.0,
            shadow_color_argb: 0xFF000000,
        }
    }
}

impl FilterSettings {
    pub(crate) fn needs_segmentation(&self) -> bool {
        self.debug_show_mask
            || self.blur_intensity > 0.0001
            || self.bg_dim > 0.0001
            || self.bg_desat > 0.0001
    }

    pub(crate) fn needs_background_composite(&self) -> bool {
        self.blur_intensity > 0.0001 || self.bg_dim > 0.0001 || self.bg_desat > 0.0001
    }

    pub(crate) unsafe fn load(settings: *mut obs::obs_data_t) -> Self {
        if settings.is_null() {
            return Self::default();
        }

        let mut s = Self::default();

        s.blur_intensity = obs::obs_data_get_double(settings, cstr(SETTING_BLUR_INTENSITY)) as f32;
        s.debug_show_mask = obs::obs_data_get_bool(settings, cstr(SETTING_DEBUG_SHOW_MASK));

        s.mask_fps = obs::obs_data_get_int(settings, cstr(SETTING_MASK_FPS)).max(1) as u32;
        s.mask_temporal_smoothing =
            obs::obs_data_get_double(settings, cstr(SETTING_MASK_TEMPORAL)) as f32;
        s.mask_threshold = obs::obs_data_get_double(settings, cstr(SETTING_MASK_THRESHOLD)) as f32;
        s.mask_softness = obs::obs_data_get_double(settings, cstr(SETTING_MASK_SOFTNESS)) as f32;
        s.mask_invert = obs::obs_data_get_bool(settings, cstr(SETTING_MASK_INVERT));
        s.sync_video_extra_delay_ms =
            obs::obs_data_get_int(settings, cstr(SETTING_SYNC_VIDEO_EXTRA_DELAY_MS)) as f32;
        s.bg_dim = obs::obs_data_get_double(settings, cstr(SETTING_BG_DIM)) as f32;
        s.bg_desat = obs::obs_data_get_double(settings, cstr(SETTING_BG_DESAT)) as f32;

        s.shape_type = obs::obs_data_get_int(settings, cstr(SETTING_SHAPE_TYPE)) as i32;
        s.corner_radius = obs::obs_data_get_double(settings, cstr(SETTING_CORNER_RADIUS)) as f32;
        s.feather = obs::obs_data_get_double(settings, cstr(SETTING_FEATHER)) as f32;
        s.border_thickness =
            obs::obs_data_get_double(settings, cstr(SETTING_BORDER_THICKNESS)) as f32;
        s.border_color_argb = obs::obs_data_get_int(settings, cstr(SETTING_BORDER_COLOR)) as u32;
        s.padding = obs::obs_data_get_double(settings, cstr(SETTING_PADDING)) as f32;
        s.frame_inset_left = obs::obs_data_get_double(settings, cstr(SETTING_FRAME_INSET_LEFT)) as f32;
        s.frame_inset_top = obs::obs_data_get_double(settings, cstr(SETTING_FRAME_INSET_TOP)) as f32;
        s.frame_inset_right = obs::obs_data_get_double(settings, cstr(SETTING_FRAME_INSET_RIGHT)) as f32;
        s.frame_inset_bottom = obs::obs_data_get_double(settings, cstr(SETTING_FRAME_INSET_BOTTOM)) as f32;

        s.shadow_opacity = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OPACITY)) as f32;
        s.shadow_blur = obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_BLUR)) as f32;
        s.shadow_offset_x =
            obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OFFSET_X)) as f32;
        s.shadow_offset_y =
            obs::obs_data_get_double(settings, cstr(SETTING_SHADOW_OFFSET_Y)) as f32;
        s.shadow_color_argb = obs::obs_data_get_int(settings, cstr(SETTING_SHADOW_COLOR)) as u32;

        s
    }
}

pub(crate) unsafe fn set_defaults(settings: *mut obs::obs_data_t) {
    if settings.is_null() {
        return;
    }

    obs::obs_data_set_default_double(settings, cstr(SETTING_BLUR_INTENSITY), 0.0);
    obs::obs_data_set_default_bool(settings, cstr(SETTING_DEBUG_SHOW_MASK), false);

    obs::obs_data_set_default_int(settings, cstr(SETTING_MASK_FPS), 15);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_TEMPORAL), 0.4);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_THRESHOLD), 0.5);
    obs::obs_data_set_default_double(settings, cstr(SETTING_MASK_SOFTNESS), 0.1);
    obs::obs_data_set_default_bool(settings, cstr(SETTING_MASK_INVERT), false);
    obs::obs_data_set_default_int(settings, cstr(SETTING_SYNC_VIDEO_EXTRA_DELAY_MS), 0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BG_DIM), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BG_DESAT), 0.0);

    obs::obs_data_set_default_int(settings, cstr(SETTING_SHAPE_TYPE), 0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_CORNER_RADIUS), 24.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FEATHER), 1.5);
    obs::obs_data_set_default_double(settings, cstr(SETTING_BORDER_THICKNESS), 0.0);
    obs::obs_data_set_default_int(settings, cstr(SETTING_BORDER_COLOR), 0xFFFFFFFFu32 as i64);
    obs::obs_data_set_default_double(settings, cstr(SETTING_PADDING), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FRAME_INSET_LEFT), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FRAME_INSET_TOP), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FRAME_INSET_RIGHT), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_FRAME_INSET_BOTTOM), 0.0);

    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OPACITY), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_BLUR), 18.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OFFSET_X), 0.0);
    obs::obs_data_set_default_double(settings, cstr(SETTING_SHADOW_OFFSET_Y), -4.0);
    obs::obs_data_set_default_int(settings, cstr(SETTING_SHADOW_COLOR), 0xFF000000u32 as i64);
}

pub(crate) unsafe fn get_properties() -> *mut obs::obs_properties_t {
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
            60,
            1,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_TEMPORAL),
            cstr(PROP_MASK_TEMPORAL),
            0.0,
            0.95,
            0.005,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_THRESHOLD),
            cstr(PROP_MASK_THRESHOLD),
            0.0,
            1.0,
            0.005,
        );
        obs::obs_properties_add_float_slider(
            seg_props,
            cstr(SETTING_MASK_SOFTNESS),
            cstr(PROP_MASK_SOFTNESS),
            0.0,
            0.5,
            0.005,
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
            0.005,
        );
        obs::obs_properties_add_float_slider(
            bg_props,
            cstr(SETTING_BG_DIM),
            cstr(PROP_BG_DIM),
            0.0,
            1.0,
            0.005,
        );
        obs::obs_properties_add_float_slider(
            bg_props,
            cstr(SETTING_BG_DESAT),
            cstr(PROP_BG_DESAT),
            0.0,
            1.0,
            0.005,
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
            obs::obs_property_list_add_int(shape_list, cstr(b"Vertical rectangle\0"), 4);
        }
        obs::obs_property_set_modified_callback(shape_list, Some(on_shape_type_modified));

        obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_CORNER_RADIUS),
            cstr(PROP_CORNER_RADIUS),
            0.0,
            2000.0,
            1.0,
        );

        let p = obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FRAME_INSET_LEFT),
            cstr(PROP_FRAME_INSET_LEFT),
            0.0,
            2000.0,
            1.0,
        );
        obs::obs_property_set_visible(p, false);
        let p = obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FRAME_INSET_TOP),
            cstr(PROP_FRAME_INSET_TOP),
            0.0,
            2000.0,
            1.0,
        );
        obs::obs_property_set_visible(p, false);
        let p = obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FRAME_INSET_RIGHT),
            cstr(PROP_FRAME_INSET_RIGHT),
            0.0,
            2000.0,
            1.0,
        );
        obs::obs_property_set_visible(p, false);
        let p = obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FRAME_INSET_BOTTOM),
            cstr(PROP_FRAME_INSET_BOTTOM),
            0.0,
            2000.0,
            1.0,
        );
        obs::obs_property_set_visible(p, false);

        obs::obs_properties_add_float_slider(
            shape_props,
            cstr(SETTING_FEATHER),
            cstr(PROP_FEATHER),
            0.0,
            32.0,
            0.05,
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
        obs::obs_properties_add_color_alpha(border_props, cstr(SETTING_BORDER_COLOR), cstr(PROP_BORDER_COLOR));

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
            0.005,
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
        obs::obs_properties_add_color_alpha(shadow_props, cstr(SETTING_SHADOW_COLOR), cstr(PROP_SHADOW_COLOR));

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
        obs::obs_properties_add_int(
            debug_props,
            cstr(SETTING_SYNC_VIDEO_EXTRA_DELAY_MS),
            cstr(PROP_SYNC_VIDEO_EXTRA_DELAY_MS),
            0,
            250,
            1,
        );
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
