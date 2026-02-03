pub(crate) static MODULE_NAME: &[u8] = b"StyledCamera\0";
pub(crate) static MODULE_DESCRIPTION: &[u8] =
    b"StyledCamera (Rust) - styled camera filter (skeleton)\0";
pub(crate) static MODULE_AUTHOR: &[u8] = b"StyledCamera contributors\0";
pub(crate) const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) static FILTER_ID: &[u8] = b"styled_camera_filter\0";
pub(crate) static FILTER_DISPLAY_NAME: &[u8] = b"Styled Camera Filter\0";

pub(crate) static SETTING_BLUR_INTENSITY: &[u8] = b"blur_intensity\0";
pub(crate) static SETTING_DEBUG_SHOW_MASK: &[u8] = b"debug_show_mask\0";
pub(crate) static SETTING_MASK_FPS: &[u8] = b"mask_fps\0";
pub(crate) static SETTING_MASK_TEMPORAL: &[u8] = b"mask_temporal_smoothing\0";
pub(crate) static SETTING_MASK_THRESHOLD: &[u8] = b"mask_threshold\0";
pub(crate) static SETTING_MASK_SOFTNESS: &[u8] = b"mask_softness\0";
pub(crate) static SETTING_MASK_INVERT: &[u8] = b"mask_invert\0";
pub(crate) static SETTING_SYNC_VIDEO_EXTRA_DELAY_MS: &[u8] = b"sync_video_extra_delay_ms\0";
pub(crate) static SETTING_BG_DIM: &[u8] = b"bg_dim\0";
pub(crate) static SETTING_BG_DESAT: &[u8] = b"bg_desat\0";
pub(crate) static SETTING_SHAPE_TYPE: &[u8] = b"shape_type\0";
pub(crate) static SETTING_CORNER_RADIUS: &[u8] = b"corner_radius\0";
pub(crate) static SETTING_FEATHER: &[u8] = b"feather\0";
pub(crate) static SETTING_BORDER_THICKNESS: &[u8] = b"border_thickness\0";
pub(crate) static SETTING_BORDER_COLOR: &[u8] = b"border_color\0";
pub(crate) static SETTING_PADDING: &[u8] = b"padding\0";
pub(crate) static SETTING_SHADOW_OPACITY: &[u8] = b"shadow_opacity\0";
pub(crate) static SETTING_SHADOW_BLUR: &[u8] = b"shadow_blur\0";
pub(crate) static SETTING_SHADOW_OFFSET_X: &[u8] = b"shadow_offset_x\0";
pub(crate) static SETTING_SHADOW_OFFSET_Y: &[u8] = b"shadow_offset_y\0";
pub(crate) static SETTING_SHADOW_COLOR: &[u8] = b"shadow_color\0";

pub(crate) static PROP_BLUR_INTENSITY: &[u8] = b"Blur intensity\0";
pub(crate) static PROP_DEBUG_SHOW_MASK: &[u8] = b"Debug: show mask\0";
pub(crate) static PROP_MASK_FPS: &[u8] = b"Mask FPS\0";
pub(crate) static PROP_MASK_TEMPORAL: &[u8] = b"Mask temporal smoothing\0";
pub(crate) static PROP_MASK_THRESHOLD: &[u8] = b"Mask threshold\0";
pub(crate) static PROP_MASK_SOFTNESS: &[u8] = b"Mask softness\0";
pub(crate) static PROP_MASK_INVERT: &[u8] = b"Invert mask\0";
pub(crate) static PROP_SYNC_VIDEO_EXTRA_DELAY_MS: &[u8] = b"Extra delay (ms)\0";
pub(crate) static PROP_BG_DIM: &[u8] = b"Background dim\0";
pub(crate) static PROP_BG_DESAT: &[u8] = b"Background desaturate\0";
pub(crate) static PROP_SHAPE_TYPE: &[u8] = b"Shape\0";
pub(crate) static PROP_CORNER_RADIUS: &[u8] = b"Corner radius\0";
pub(crate) static PROP_FEATHER: &[u8] = b"Feather\0";
pub(crate) static PROP_BORDER_THICKNESS: &[u8] = b"Border thickness\0";
pub(crate) static PROP_BORDER_COLOR: &[u8] = b"Border color\0";
pub(crate) static PROP_PADDING: &[u8] = b"Padding\0";
pub(crate) static PROP_SHADOW_OPACITY: &[u8] = b"Shadow opacity\0";
pub(crate) static PROP_SHADOW_BLUR: &[u8] = b"Shadow blur\0";
pub(crate) static PROP_SHADOW_OFFSET_X: &[u8] = b"Shadow offset X\0";
pub(crate) static PROP_SHADOW_OFFSET_Y: &[u8] = b"Shadow offset Y\0";
pub(crate) static PROP_SHADOW_COLOR: &[u8] = b"Shadow color\0";

pub(crate) static GROUP_SEGMENTATION: &[u8] = b"group_segmentation\0";
pub(crate) static GROUP_TIMING: &[u8] = b"group_timing\0";
pub(crate) static GROUP_BACKGROUND: &[u8] = b"group_background\0";
pub(crate) static GROUP_SHAPE: &[u8] = b"group_shape\0";
pub(crate) static GROUP_BORDER: &[u8] = b"group_border\0";
pub(crate) static GROUP_SHADOW: &[u8] = b"group_shadow\0";
pub(crate) static GROUP_DEBUG: &[u8] = b"group_debug\0";

pub(crate) static GROUP_LABEL_SEGMENTATION: &[u8] = b"Segmentation\0";
pub(crate) static GROUP_LABEL_TIMING: &[u8] = b"Timing\0";
pub(crate) static GROUP_LABEL_BACKGROUND: &[u8] = b"Background\0";
pub(crate) static GROUP_LABEL_SHAPE: &[u8] = b"Shape\0";
pub(crate) static GROUP_LABEL_BORDER: &[u8] = b"Border\0";
pub(crate) static GROUP_LABEL_SHADOW: &[u8] = b"Shadow\0";
pub(crate) static GROUP_LABEL_DEBUG: &[u8] = b"Debug\0";

pub(crate) static EFFECT_BLUR_DOWNSAMPLE: &[u8] = b"blur_downsample.effect\0";
pub(crate) static EFFECT_BLUR_PASS: &[u8] = b"blur_pass.effect\0";
pub(crate) static EFFECT_COMPOSITE: &[u8] = b"styled_composite.effect\0";
pub(crate) static EFFECT_SHAPE_STYLE: &[u8] = b"shape_style.effect\0";

pub(crate) static TECH_DOWNSAMPLE: &[u8] = b"Downsample\0";
pub(crate) static TECH_BLUR_H: &[u8] = b"BlurH\0";
pub(crate) static TECH_BLUR_V: &[u8] = b"BlurV\0";
pub(crate) static TECH_COMPOSITE: &[u8] = b"Composite\0";
pub(crate) static TECH_SHAPE_STYLE: &[u8] = b"ShapeStyle\0";

pub(crate) static MODEL_FILE: &[u8] = b"models/selfie_segmentation.onnx\0";
