use std::time::{Duration, Instant};

use obs_sys as obs;

pub(crate) struct FrameHistoryEntry {
    pub tex: *mut obs::gs_texrender_t,
    pub time: Option<Instant>,
}

pub(crate) struct FrameHistory {
    entries: Vec<FrameHistoryEntry>,
    next: usize,
}

impl Default for FrameHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next: 0,
        }
    }
}

impl FrameHistory {
    pub(crate) fn next_slot_mut(&mut self) -> Option<&mut FrameHistoryEntry> {
        if self.entries.is_empty() {
            return None;
        }

        let i = self.next % self.entries.len();
        self.next = (i + 1) % self.entries.len();
        Some(&mut self.entries[i])
    }

    // Must be called while in graphics context.
    pub(crate) unsafe fn ensure_allocated(&mut self) {
        if !self.entries.is_empty() {
            return;
        }

        // Keep this small to avoid large VRAM usage; enough for typical inference latency (tens of ms).
        const N: usize = 8;
        self.entries.reserve(N);
        for _ in 0..N {
            let tr = obs::gs_texrender_create(
                obs::gs_color_format_GS_RGBA,
                obs::gs_zstencil_format_GS_ZS_NONE,
            );
            self.entries.push(FrameHistoryEntry { tex: tr, time: None });
        }
        self.next = 0;
    }

    // Must be called while in graphics context.
    pub(crate) unsafe fn destroy(&mut self) {
        for entry in self.entries.drain(..) {
            if !entry.tex.is_null() {
                obs::gs_texrender_destroy(entry.tex);
            }
        }
        self.next = 0;
    }

    pub(crate) unsafe fn select_delayed_texture(
        &self,
        fallback: *mut obs::gs_texture_t,
        delay_ms: f32,
    ) -> *mut obs::gs_texture_t {
        if self.entries.is_empty() {
            return fallback;
        }

        let delay_ms = delay_ms.clamp(0.0, 500.0);
        if delay_ms <= 0.5 {
            return fallback;
        }

        let desired = Instant::now() - Duration::from_secs_f32(delay_ms / 1000.0);
        let mut best_tex: *mut obs::gs_texture_t = std::ptr::null_mut();
        let mut best_dt: Option<Duration> = None;

        for entry in self.entries.iter() {
            let Some(t) = entry.time else {
                continue;
            };
            let dt = if t <= desired {
                desired.duration_since(t)
            } else {
                t.duration_since(desired)
            };
            if best_dt.map(|b| dt < b).unwrap_or(true) {
                let tex = obs::gs_texrender_get_texture(entry.tex);
                if !tex.is_null() {
                    best_dt = Some(dt);
                    best_tex = tex;
                }
            }
        }

        if best_tex.is_null() { fallback } else { best_tex }
    }
}
