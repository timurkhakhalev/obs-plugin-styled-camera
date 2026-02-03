use std::os::raw::c_char;

use obs_sys as obs;

use crate::constants::*;
use crate::frame_history::FrameHistory;
use crate::settings::FilterSettings;
use crate::util::cstr;
use styledcamera_core::color::obs_abgr_to_rgba_vec4;

pub(crate) struct GraphicsState {
    pub effect_downsample: *mut obs::gs_effect_t,
    pub effect_blur: *mut obs::gs_effect_t,
    pub effect_composite: *mut obs::gs_effect_t,
    pub effect_shape: *mut obs::gs_effect_t,

    pub downsample_image: *mut obs::gs_eparam_t,
    pub downsample_texel_size: *mut obs::gs_eparam_t,

    pub blur_image: *mut obs::gs_eparam_t,
    pub blur_texel_size: *mut obs::gs_eparam_t,
    pub blur_radius: *mut obs::gs_eparam_t,

    pub composite_image: *mut obs::gs_eparam_t,
    pub composite_blur_image: *mut obs::gs_eparam_t,
    pub composite_mask_image: *mut obs::gs_eparam_t,
    pub composite_mask_threshold: *mut obs::gs_eparam_t,
    pub composite_mask_softness: *mut obs::gs_eparam_t,
    pub composite_mask_invert: *mut obs::gs_eparam_t,
    pub composite_bg_dim: *mut obs::gs_eparam_t,
    pub composite_bg_desat: *mut obs::gs_eparam_t,

    pub shape_image: *mut obs::gs_eparam_t,
    pub shape_size: *mut obs::gs_eparam_t,
    pub shape_type_param: *mut obs::gs_eparam_t,
    pub shape_corner_radius: *mut obs::gs_eparam_t,
    pub shape_feather: *mut obs::gs_eparam_t,
    pub shape_border_thickness: *mut obs::gs_eparam_t,
    pub shape_border_color: *mut obs::gs_eparam_t,
    pub shape_inset: *mut obs::gs_eparam_t,
    pub shape_frame_scale: *mut obs::gs_eparam_t,
    pub shape_shadow_offset: *mut obs::gs_eparam_t,
    pub shape_shadow_blur: *mut obs::gs_eparam_t,
    pub shape_shadow_color: *mut obs::gs_eparam_t,

    pub tex_down: *mut obs::gs_texrender_t,
    pub tex_ping: *mut obs::gs_texrender_t,
    pub tex_pong: *mut obs::gs_texrender_t,
    pub tex_comp: *mut obs::gs_texrender_t,
    pub tex_seg: *mut obs::gs_texrender_t,

    pub mask_tex: *mut obs::gs_texture_t,
    pub mask_w: u32,
    pub mask_h: u32,
    pub stage_seg: *mut obs::gs_stagesurf_t,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
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
            shape_frame_scale: std::ptr::null_mut(),
            shape_shadow_offset: std::ptr::null_mut(),
            shape_shadow_blur: std::ptr::null_mut(),
            shape_shadow_color: std::ptr::null_mut(),

            tex_down: std::ptr::null_mut(),
            tex_ping: std::ptr::null_mut(),
            tex_pong: std::ptr::null_mut(),
            tex_comp: std::ptr::null_mut(),
            tex_seg: std::ptr::null_mut(),

            mask_tex: std::ptr::null_mut(),
            mask_w: 0,
            mask_h: 0,
            stage_seg: std::ptr::null_mut(),
        }
    }
}

impl GraphicsState {
    pub(crate) unsafe fn ensure(&mut self) -> bool {
        if !self.effect_downsample.is_null()
            && !self.effect_blur.is_null()
            && !self.effect_composite.is_null()
            && !self.effect_shape.is_null()
            && !self.tex_down.is_null()
            && !self.tex_ping.is_null()
            && !self.tex_pong.is_null()
            && !self.tex_comp.is_null()
            && !self.tex_seg.is_null()
            && !self.mask_tex.is_null()
            && !self.stage_seg.is_null()
        {
            return true;
        }

        self.init();
        !self.effect_downsample.is_null()
            && !self.effect_blur.is_null()
            && !self.effect_composite.is_null()
            && !self.effect_shape.is_null()
            && !self.tex_down.is_null()
            && !self.tex_ping.is_null()
            && !self.tex_pong.is_null()
            && !self.tex_comp.is_null()
            && !self.tex_seg.is_null()
            && !self.mask_tex.is_null()
            && !self.stage_seg.is_null()
    }

    pub(crate) unsafe fn init(&mut self) {
        obs::obs_enter_graphics();

        if self.effect_downsample.is_null() {
            self.effect_downsample = load_effect(EFFECT_BLUR_DOWNSAMPLE);
            if !self.effect_downsample.is_null() {
                self.downsample_image =
                    obs::gs_effect_get_param_by_name(self.effect_downsample, cstr(b"image\0"));
                self.downsample_texel_size = obs::gs_effect_get_param_by_name(
                    self.effect_downsample,
                    cstr(b"texel_size\0"),
                );
            }
        }

        if self.effect_blur.is_null() {
            self.effect_blur = load_effect(EFFECT_BLUR_PASS);
            if !self.effect_blur.is_null() {
                self.blur_image =
                    obs::gs_effect_get_param_by_name(self.effect_blur, cstr(b"image\0"));
                self.blur_texel_size =
                    obs::gs_effect_get_param_by_name(self.effect_blur, cstr(b"texel_size\0"));
                self.blur_radius =
                    obs::gs_effect_get_param_by_name(self.effect_blur, cstr(b"blur_radius\0"));
            }
        }

        if self.effect_composite.is_null() {
            self.effect_composite = load_effect(EFFECT_COMPOSITE);
            if !self.effect_composite.is_null() {
                self.composite_image =
                    obs::gs_effect_get_param_by_name(self.effect_composite, cstr(b"image\0"));
                self.composite_blur_image = obs::gs_effect_get_param_by_name(
                    self.effect_composite,
                    cstr(b"blur_image\0"),
                );
                self.composite_mask_image = obs::gs_effect_get_param_by_name(
                    self.effect_composite,
                    cstr(b"mask_image\0"),
                );
                self.composite_mask_threshold = obs::gs_effect_get_param_by_name(
                    self.effect_composite,
                    cstr(b"mask_threshold\0"),
                );
                self.composite_mask_softness =
                    obs::gs_effect_get_param_by_name(self.effect_composite, cstr(b"mask_softness\0"));
                self.composite_mask_invert =
                    obs::gs_effect_get_param_by_name(self.effect_composite, cstr(b"mask_invert\0"));
                self.composite_bg_dim =
                    obs::gs_effect_get_param_by_name(self.effect_composite, cstr(b"bg_dim\0"));
                self.composite_bg_desat =
                    obs::gs_effect_get_param_by_name(self.effect_composite, cstr(b"bg_desat\0"));
            }
        }

        if self.effect_shape.is_null() {
            self.effect_shape = load_effect(EFFECT_SHAPE_STYLE);
            if !self.effect_shape.is_null() {
                self.shape_image =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"image\0"));
                self.shape_size = obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"size\0"));
                self.shape_type_param =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"shape_type\0"));
                self.shape_corner_radius =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"corner_radius\0"));
                self.shape_feather =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"feather\0"));
                self.shape_border_thickness =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"border_thickness\0"));
                self.shape_border_color =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"border_color\0"));
                self.shape_inset =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"inset\0"));
                self.shape_frame_scale =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"frame_scale\0"));
                self.shape_shadow_offset =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"shadow_offset\0"));
                self.shape_shadow_blur =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"shadow_blur\0"));
                self.shape_shadow_color =
                    obs::gs_effect_get_param_by_name(self.effect_shape, cstr(b"shadow_color\0"));
            }
        }

        if self.tex_down.is_null() {
            self.tex_down = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
        }
        if self.tex_ping.is_null() {
            self.tex_ping = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
        }
        if self.tex_pong.is_null() {
            self.tex_pong = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
        }
        if self.tex_comp.is_null() {
            self.tex_comp = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
        }
        if self.tex_seg.is_null() {
            self.tex_seg = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
        }

        if self.mask_tex.is_null() {
            let mask_w = 256u32;
            let mask_h = 256u32;
            // Default to 0 (no person) until we receive real mask data.
            let init = vec![0u8; (mask_w * mask_h) as usize];
            let mut data_ptrs = [init.as_ptr()];
            self.mask_tex = obs::gs_texture_create(
                mask_w,
                mask_h,
                obs::gs_color_format_GS_R8,
                1,
                data_ptrs.as_mut_ptr(),
                obs::GS_DYNAMIC,
            );
            self.mask_w = mask_w;
            self.mask_h = mask_h;
        }

        if self.stage_seg.is_null() {
            self.stage_seg = obs::gs_stagesurface_create(256, 256, obs::gs_color_format_GS_RGBA);
        }

        obs::obs_leave_graphics();
    }

    pub(crate) unsafe fn destroy(&mut self, frame_history: &mut FrameHistory) {
        obs::obs_enter_graphics();

        if !self.mask_tex.is_null() {
            obs::gs_texture_destroy(self.mask_tex);
            self.mask_tex = std::ptr::null_mut();
        }
        self.mask_w = 0;
        self.mask_h = 0;

        if !self.stage_seg.is_null() {
            obs::gs_stagesurface_destroy(self.stage_seg);
            self.stage_seg = std::ptr::null_mut();
        }

        frame_history.destroy();

        if !self.tex_down.is_null() {
            obs::gs_texrender_destroy(self.tex_down);
            self.tex_down = std::ptr::null_mut();
        }
        if !self.tex_ping.is_null() {
            obs::gs_texrender_destroy(self.tex_ping);
            self.tex_ping = std::ptr::null_mut();
        }
        if !self.tex_pong.is_null() {
            obs::gs_texrender_destroy(self.tex_pong);
            self.tex_pong = std::ptr::null_mut();
        }
        if !self.tex_comp.is_null() {
            obs::gs_texrender_destroy(self.tex_comp);
            self.tex_comp = std::ptr::null_mut();
        }
        if !self.tex_seg.is_null() {
            obs::gs_texrender_destroy(self.tex_seg);
            self.tex_seg = std::ptr::null_mut();
        }

        if !self.effect_downsample.is_null() {
            obs::gs_effect_destroy(self.effect_downsample);
            self.effect_downsample = std::ptr::null_mut();
            self.downsample_image = std::ptr::null_mut();
            self.downsample_texel_size = std::ptr::null_mut();
        }
        if !self.effect_blur.is_null() {
            obs::gs_effect_destroy(self.effect_blur);
            self.effect_blur = std::ptr::null_mut();
            self.blur_image = std::ptr::null_mut();
            self.blur_texel_size = std::ptr::null_mut();
            self.blur_radius = std::ptr::null_mut();
        }
        if !self.effect_composite.is_null() {
            obs::gs_effect_destroy(self.effect_composite);
            self.effect_composite = std::ptr::null_mut();
            self.composite_image = std::ptr::null_mut();
            self.composite_blur_image = std::ptr::null_mut();
            self.composite_mask_image = std::ptr::null_mut();
            self.composite_mask_threshold = std::ptr::null_mut();
            self.composite_mask_softness = std::ptr::null_mut();
            self.composite_mask_invert = std::ptr::null_mut();
            self.composite_bg_dim = std::ptr::null_mut();
            self.composite_bg_desat = std::ptr::null_mut();
        }
        if !self.effect_shape.is_null() {
            obs::gs_effect_destroy(self.effect_shape);
            self.effect_shape = std::ptr::null_mut();
            self.shape_image = std::ptr::null_mut();
            self.shape_size = std::ptr::null_mut();
            self.shape_type_param = std::ptr::null_mut();
            self.shape_corner_radius = std::ptr::null_mut();
            self.shape_feather = std::ptr::null_mut();
            self.shape_border_thickness = std::ptr::null_mut();
            self.shape_border_color = std::ptr::null_mut();
            self.shape_inset = std::ptr::null_mut();
            self.shape_frame_scale = std::ptr::null_mut();
            self.shape_shadow_offset = std::ptr::null_mut();
            self.shape_shadow_blur = std::ptr::null_mut();
            self.shape_shadow_color = std::ptr::null_mut();
        }

        obs::obs_leave_graphics();
    }
}

unsafe fn load_effect(file: &'static [u8]) -> *mut obs::gs_effect_t {
    // obs_module_file is a macro; use the underlying exported function.
    let module = obs::obs_current_module();
    if module.is_null() {
        obs::blog(
            obs::LOG_WARNING as i32,
            cstr(b"StyledCamera: obs_current_module returned NULL\n\0"),
        );
        return std::ptr::null_mut();
    }

    let path = obs::obs_find_module_file(module, cstr(file));
    if path.is_null() {
        obs::blog(
            obs::LOG_WARNING as i32,
            cstr(b"StyledCamera: obs_find_module_file returned NULL\n\0"),
        );
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

pub(crate) unsafe fn render_source_to_texrender(
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

pub(crate) unsafe fn render_effect_to_texrender(
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

pub(crate) unsafe fn draw_shape_to_screen(
    gfx: &GraphicsState,
    settings: &FilterSettings,
    tex: *mut obs::gs_texture_t,
    cx: u32,
    cy: u32,
) {
    if gfx.effect_shape.is_null() || tex.is_null() {
        return;
    }

    if !gfx.shape_image.is_null() {
        obs::gs_effect_set_texture(gfx.shape_image, tex);
    }
    if !gfx.shape_size.is_null() {
        set_vec2_param(gfx.shape_size, cx as f32, cy as f32);
    }
    if !gfx.shape_type_param.is_null() {
        obs::gs_effect_set_float(gfx.shape_type_param, settings.shape_type as f32);
    }
    set_float_param(gfx.shape_corner_radius, settings.corner_radius);
    set_float_param(gfx.shape_feather, settings.feather);
    set_float_param(gfx.shape_border_thickness, settings.border_thickness);

    if !gfx.shape_border_color.is_null() {
        let border = obs_abgr_to_rgba_vec4(settings.border_color_argb);
        set_vec4_param(gfx.shape_border_color, border);
    }

    if !gfx.shape_inset.is_null() {
        let p = settings.padding.max(0.0);
        set_vec4_param(gfx.shape_inset, [p, p, p, p]);
    }

    if !gfx.shape_frame_scale.is_null() {
        let sx = settings.frame_width.clamp(0.05, 1.0);
        let sy = settings.frame_height.clamp(0.05, 1.0);
        // Only apply scaling for box-based shapes; circles remain fixed.
        let (sx, sy) = if settings.shape_type == 1
            || settings.shape_type == 2
            || settings.shape_type == 3
            || settings.shape_type == 4
        {
            (sx, sy)
        } else {
            (1.0, 1.0)
        };
        set_vec2_param(gfx.shape_frame_scale, sx, sy);
    }

    if !gfx.shape_shadow_offset.is_null() {
        set_vec2_param(
            gfx.shape_shadow_offset,
            settings.shadow_offset_x,
            settings.shadow_offset_y,
        );
    }
    set_float_param(gfx.shape_shadow_blur, settings.shadow_blur);
    if !gfx.shape_shadow_color.is_null() {
        let mut shadow = obs_abgr_to_rgba_vec4(settings.shadow_color_argb);
        shadow[3] *= settings.shadow_opacity.clamp(0.0, 1.0);
        set_vec4_param(gfx.shape_shadow_color, shadow);
    }

    obs::gs_blend_state_push();
    obs::gs_blend_function(
        obs::gs_blend_type_GS_BLEND_ONE,
        obs::gs_blend_type_GS_BLEND_INVSRCALPHA,
    );

    while obs::gs_effect_loop(gfx.effect_shape, cstr(TECH_SHAPE_STYLE)) {
        obs::gs_draw_sprite(tex, 0, cx, cy);
    }

    obs::gs_blend_state_pop();
}

pub(crate) unsafe fn set_vec2_param(param: *mut obs::gs_eparam_t, x: f32, y: f32) {
    if param.is_null() {
        return;
    }
    let v = [x, y];
    obs::gs_effect_set_val(param, v.as_ptr().cast(), std::mem::size_of_val(&v) as obs::size_t);
}

pub(crate) unsafe fn set_float_param(param: *mut obs::gs_eparam_t, v: f32) {
    if param.is_null() {
        return;
    }
    obs::gs_effect_set_float(param, v);
}

pub(crate) unsafe fn set_vec4_param(param: *mut obs::gs_eparam_t, v: [f32; 4]) {
    if param.is_null() {
        return;
    }
    obs::gs_effect_set_val(param, v.as_ptr().cast(), std::mem::size_of_val(&v) as obs::size_t);
}
