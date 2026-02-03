# StyledCamera (OBS Studio plugin, Rust)

StyledCamera is an experimental OBS Studio video filter plugin written in Rust.

## What it does (today)

- Runs person/selfie segmentation (ONNX Runtime, loaded dynamically).
- Blurs the background behind the person based on the mask.
- Applies simple “card” styling (shape, feather, border, shadow).

## Repo layout

- `crates/styledcamera/` — the OBS plugin (`cdylib`).
- `crates/styledcamera-core/` — pure logic with unit tests (no OBS linkage).
- `crates/obs-sys/` — minimal `bindgen`-based OBS FFI.
- `data/effects/` — `.effect` shaders used by the plugin.
- `scripts/` — install/uninstall/packaging helpers.
- `third_party/NOTICE.md` — third-party attributions for bundled/downloaded components.

## Build

1) Fetch submodules (OBS headers):

```bash
git submodule update --init --recursive
```

2) Build:

```bash
cargo build -p styledcamera
```

## Tests

`styledcamera` is a `cdylib` that links against OBS (`libobs`), so its test harness is disabled.
Unit tests live in `styledcamera-core`:

```bash
cargo test
```

## Packaging / install

See `PACKAGING.md`.

## Versioning / releases

- Single source of truth: `[workspace.package].version` in `Cargo.toml`.
- To publish a macOS release artifact via GitHub Actions, push a tag matching that version (e.g. `v0.1.0`).
  The workflow builds and attaches `StyledCamera-macos-arm64-vX.Y.Z.zip` to the GitHub Release.

## License

GPL-2.0-or-later. See `LICENSE`.
