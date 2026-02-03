pub fn update_mask_latency_ema_ms(ema_ms: &mut f32, measured_ms: f32) {
    if !measured_ms.is_finite() || measured_ms < 0.0 || measured_ms > 2000.0 {
        return;
    }

    if *ema_ms <= 0.0 {
        *ema_ms = measured_ms;
    } else {
        *ema_ms = (*ema_ms * 0.8) + (measured_ms * 0.2);
    }
}

#[cfg(test)]
mod tests {
    use super::update_mask_latency_ema_ms;

    #[test]
    fn mask_latency_ema_initializes() {
        let mut ema = 0.0;
        update_mask_latency_ema_ms(&mut ema, 100.0);
        assert_eq!(ema, 100.0);
    }

    #[test]
    fn mask_latency_ema_updates_with_weights() {
        let mut ema = 100.0;
        update_mask_latency_ema_ms(&mut ema, 200.0);
        assert!((ema - 120.0).abs() < 1e-6);
    }

    #[test]
    fn mask_latency_ema_ignores_bad_values() {
        let mut ema = 50.0;
        update_mask_latency_ema_ms(&mut ema, f32::NAN);
        update_mask_latency_ema_ms(&mut ema, -1.0);
        update_mask_latency_ema_ms(&mut ema, 5000.0);
        assert_eq!(ema, 50.0);
    }
}

