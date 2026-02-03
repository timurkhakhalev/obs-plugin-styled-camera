pub fn obs_abgr_to_rgba_vec4(abgr: u32) -> [f32; 4] {
    // OBS color properties are stored as 0xAABBGGRR (ABGR).
    let a = ((abgr >> 24) & 0xFF) as f32 / 255.0;
    let b = ((abgr >> 16) & 0xFF) as f32 / 255.0;
    let g = ((abgr >> 8) & 0xFF) as f32 / 255.0;
    let r = (abgr & 0xFF) as f32 / 255.0;

    [r, g, b, a]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abgr_to_rgba_opaque_white() {
        assert_eq!(obs_abgr_to_rgba_vec4(0xFFFFFFFF), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn abgr_to_rgba_matches_channels() {
        // A=0x80, B=0x40, G=0x20, R=0x10 (ABGR).
        let rgba = obs_abgr_to_rgba_vec4(0x80402010);
        assert_eq!(rgba[0], 0x10 as f32 / 255.0);
        assert_eq!(rgba[1], 0x20 as f32 / 255.0);
        assert_eq!(rgba[2], 0x40 as f32 / 255.0);
        assert_eq!(rgba[3], 0x80 as f32 / 255.0);
    }
}

