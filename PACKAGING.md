# Packaging (macOS arm64)

This repo contains install/uninstall helpers plus packaging scripts that assemble a redistributable OBS plugin bundle.

## 1) Fetch the segmentation model

Downloads the default MediaPipe Selfie Segmentation ONNX model into `model/`:

```bash
./scripts/fetch-model.sh
```

Default output:
- `model/mediapipe_selfie_segmentation.onnx`

## 2) Provide ONNX Runtime (`libonnxruntime.dylib`)

The macOS bundle is expected to ship `libonnxruntime.dylib` alongside the plugin.

You have two options:

### Option A: Provide your own dylib (recommended for reproducibility)

Pass an absolute path:

```bash
./scripts/package-macos-arm64.sh --onnxruntime /absolute/path/to/libonnxruntime.dylib
```

Or via env var:

```bash
ONNXRUNTIME_DYLIB=/absolute/path/to/libonnxruntime.dylib ./scripts/package-macos-arm64.sh
```

### Option B: Download a release build

```bash
./scripts/package-macos-arm64.sh --download-onnxruntime
```

To pin a specific version:

```bash
./scripts/package-macos-arm64.sh --download-onnxruntime --onnxruntime-version 1.20.1
```

If you omit `--onnxruntime-version`, the script defaults to `1.23.0` (to match the `ort` runtime API requirements).

## 3) Package the plugin bundle

Example:

```bash
./scripts/package-macos-arm64.sh --download-onnxruntime
```

By default, the bundle `Info.plist` version fields are taken from the Cargo package version (`[workspace.package].version` in `Cargo.toml`).
Override with `--version` (or `PLUGIN_VERSION`) if you need to package a different version.

Outputs:
- `dist/macos/StyledCamera.plugin`
- `dist/StyledCamera-macos-arm64.zip`

Bundled resources:
- `Contents/Resources/*.effect` (copied from `data/effects/`)
- `Contents/Resources/models/selfie_segmentation.onnx`
- `Contents/Resources/NOTICE.md` (copied from `third_party/NOTICE.md`)
- `Contents/Frameworks/libonnxruntime.dylib`

## 4) Install into OBS (user-level)

```bash
./scripts/install-macos.sh ./dist/macos/StyledCamera.plugin
```

Uninstall:

```bash
./scripts/uninstall-macos.sh
```

## Notes

- Third-party attributions are tracked in `third_party/NOTICE.md` and copied into the bundle.
- The plugin binary must be built with an rpath/install-name strategy that can locate the bundled dylib, typically via `@loader_path/../Frameworks/`.
