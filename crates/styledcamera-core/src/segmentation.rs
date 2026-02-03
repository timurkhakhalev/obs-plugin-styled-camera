pub fn postprocess_mask_u8(
    expected: usize,
    out_shape: &[i64],
    out_data: &[f32],
    prev_mask: &mut Vec<f32>,
    temporal_smoothing: f32,
) -> Option<Vec<u8>> {
    let mut current = extract_mask_values(expected, out_shape, out_data)?;
    if looks_like_logits(&current) {
        sigmoid_in_place(&mut current);
    }

    update_temporal_smoothed(prev_mask, &current, temporal_smoothing);
    Some(to_u8_mask(prev_mask))
}

fn extract_mask_values(expected: usize, out_shape: &[i64], out_data: &[f32]) -> Option<Vec<f32>> {
    // Handle common MediaPipe-style outputs:
    // - [1, H, W, 1] or [1, 1, H, W] (single channel)
    // - [1, H, W, 2] or [1, 2, H, W] (2 classes: background/person)
    if out_data.len() == expected {
        return Some(out_data.to_vec());
    }

    if out_data.len() == expected * 2 {
        let is_nhwc_2 = out_shape.len() == 4 && out_shape[3] == 2;
        let is_nchw_2 = out_shape.len() == 4 && out_shape[1] == 2;

        let mut current = vec![0.0f32; expected];
        if is_nhwc_2 {
            // Interleaved channels per pixel: [..., 2]
            for i in 0..expected {
                current[i] = out_data[i * 2 + 1];
            }
        } else if is_nchw_2 {
            // Planar channels: [1, 2, H, W]
            current.copy_from_slice(&out_data[expected..expected * 2]);
        } else {
            // Fallback: assume interleaved and take channel 1.
            for i in 0..expected {
                current[i] = out_data[i * 2 + 1];
            }
        }
        return Some(current);
    }

    // Fallback: take the first plane worth of values.
    if out_data.len() >= expected {
        return Some(out_data[..expected].to_vec());
    }

    None
}

fn looks_like_logits(values: &[f32]) -> bool {
    let mut min_v = f32::INFINITY;
    let mut max_v = f32::NEG_INFINITY;
    for &v in values {
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }

    min_v < -0.01 || max_v > 1.01
}

fn sigmoid_in_place(values: &mut [f32]) {
    for v in values {
        let x = *v;
        *v = 1.0 / (1.0 + (-x).exp());
    }
}

fn update_temporal_smoothed(prev_mask: &mut Vec<f32>, current: &[f32], temporal_smoothing: f32) {
    if prev_mask.len() != current.len() {
        prev_mask.clear();
        prev_mask.extend_from_slice(current);
        return;
    }

    let a = temporal_smoothing.clamp(0.0, 0.99);
    let b = 1.0 - a;
    for (p, &c) in prev_mask.iter_mut().zip(current.iter()) {
        *p = (*p * a) + (c * b);
    }
}

fn to_u8_mask(values: &[f32]) -> Vec<u8> {
    let mut mask_u8 = vec![0u8; values.len()];
    for (dst, &v) in mask_u8.iter_mut().zip(values.iter()) {
        *dst = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    mask_u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_single_channel_output() {
        let expected = 4;
        let out_shape = [1, 2, 2, 1];
        let out_data = [0.0, 0.25, 0.5, 1.0];
        let mut prev = Vec::new();
        let mask = postprocess_mask_u8(expected, &out_shape, &out_data, &mut prev, 0.0).unwrap();
        assert_eq!(mask, vec![0, 64, 128, 255]);
    }

    #[test]
    fn extracts_two_channel_nhwc_person() {
        let expected = 2;
        let out_shape = [1, 1, 2, 2];
        // [bg0, person0, bg1, person1]
        let out_data = [0.1, 0.9, 0.2, 0.8];
        let mut prev = Vec::new();
        let mask = postprocess_mask_u8(expected, &out_shape, &out_data, &mut prev, 0.0).unwrap();
        assert_eq!(mask, vec![230, 204]);
    }

    #[test]
    fn extracts_two_channel_nchw_person_plane() {
        let expected = 3;
        let out_shape = [1, 2, 1, 3];
        // planar: bg[0..expected], person[expected..expected*2]
        let out_data = [0.1, 0.2, 0.3, 0.9, 0.8, 0.7];
        let mut prev = Vec::new();
        let mask = postprocess_mask_u8(expected, &out_shape, &out_data, &mut prev, 0.0).unwrap();
        assert_eq!(mask, vec![230, 204, 179]);
    }

    #[test]
    fn applies_sigmoid_for_logits() {
        let expected = 3;
        let out_shape = [1, 1, 3, 1];
        let out_data = [-1.0, 0.0, 1.0];
        let mut prev = Vec::new();
        let mask = postprocess_mask_u8(expected, &out_shape, &out_data, &mut prev, 0.0).unwrap();
        assert_eq!(mask, vec![69, 128, 186]);
    }

    #[test]
    fn applies_temporal_smoothing() {
        let expected = 2;
        let out_shape = [1, 1, 2, 1];
        let mut prev = Vec::new();

        let mask1 = postprocess_mask_u8(expected, &out_shape, &[0.0, 1.0], &mut prev, 0.5).unwrap();
        assert_eq!(mask1, vec![0, 255]);

        let mask2 = postprocess_mask_u8(expected, &out_shape, &[1.0, 0.0], &mut prev, 0.5).unwrap();
        assert_eq!(mask2, vec![128, 128]);
    }

    #[test]
    fn temporal_smoothing_is_clamped() {
        let expected = 1;
        let out_shape = [1, 1, 1, 1];
        let mut prev = Vec::new();

        let _ = postprocess_mask_u8(expected, &out_shape, &[0.0], &mut prev, 0.0).unwrap();
        let mask = postprocess_mask_u8(expected, &out_shape, &[1.0], &mut prev, 1.0).unwrap();
        assert_eq!(mask, vec![3]);
    }
}
