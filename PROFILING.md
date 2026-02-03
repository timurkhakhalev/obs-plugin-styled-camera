# Profiling StyledCamera (macOS)

This repo is primarily targeting OBS Studio on macOS (arm64), so the notes below focus on macOS tools.

## What to measure

1) **OBS FPS + render time** (OBS UI): `View → Stats`
2) **CPU / Memory / GPU** (system): Activity Monitor
3) **Hotspots** (sampling profiler): `sample` (built-in) or Xcode Instruments (optional)

To get meaningful numbers, keep the scene stable and measure for at least ~30–60 seconds per condition.

## Baseline: OBS with vs without the filter

Recommended setup:

- Create a scene with a single camera source (or a looping media source so it is perfectly repeatable).
- Duplicate the scene:
  - `Scene A`: no StyledCamera filter
  - `Scene B`: StyledCamera filter enabled
- Keep output settings identical (canvas/output resolution, FPS, renderer).

Then:

- Open `View → Stats`
- Observe `CPU Usage`, `FPS`, `Frame Render Time`, and dropped/skipped frames
- In Activity Monitor:
  - Find `OBS`
  - Watch `% CPU`, `Memory`, and `GPU` (Window → GPU History)

## Disable segmentation/background work (for A/B testing)

StyledCamera only runs segmentation/background compositing when one of these is enabled:

- Blur intensity > 0
- Background dim/desaturate > 0
- Debug: show mask

So if you set **blur/dim/desat = 0** (and keep Debug: show mask off), the filter should behave like a pure “shape/border/shadow” style pass and avoid the expensive segmentation path.

## Built-in perf logging (optional)

There is an opt-in build-time feature that logs average timings once per second into the OBS log.

1) Build the plugin with the feature:

```bash
cargo build -p styledcamera --release --features perf
```

2) Launch OBS with perf logs enabled:

```bash
STYLEDCAMERA_PERF=1 /Applications/OBS.app/Contents/MacOS/OBS
```

3) View logs in OBS:

`Help → Log Files → View Current Log`

You should see lines like:

- `StyledCamera: perf(render): ...`
- `StyledCamera: perf(seg): ...`

## Sampling CPU hotspots (no Xcode required)

`sample` can capture stack traces for a running process:

```bash
sample OBS 10 -file /tmp/obs.sample.txt
```

Open `/tmp/obs.sample.txt` and search for `styledcamera` / `onnxruntime` to spot hotspots.

## Xcode Instruments (optional)

If you have Xcode installed, use Instruments:

- **Time Profiler** for CPU hotspots
- **Metal System Trace** for GPU work

For best symbols, build with debug info:

```bash
RUSTFLAGS="-C debuginfo=2" cargo build -p styledcamera --release
```

