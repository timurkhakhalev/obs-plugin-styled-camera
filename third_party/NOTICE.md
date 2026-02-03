# Third-party notices

This repository’s build/package scripts may download and/or bundle third-party components into release artifacts.
Keep this file up to date whenever the set of bundled third-party components changes.

## MediaPipe Selfie Segmentation model (ONNX)

- Component: person/selfie segmentation model used for background separation.
- Downloaded by: `scripts/fetch-model.sh`
- Default source URL:
  - `https://huggingface.co/onnx-community/mediapipe_selfie_segmentation`
- Upstream project:
  - MediaPipe (Google)
- License:
  - Apache License 2.0 (MediaPipe)

Notes:
- The hosted ONNX model is a converted artifact intended for inference.
- If you switch model sources (or re-export/re-train), update this section accordingly.

## ONNX Runtime (libonnxruntime.dylib)

- Component: ONNX Runtime dynamic library used for model inference.
- Bundled by: `scripts/package-macos-arm64.sh`
- Upstream project:
  - `https://github.com/microsoft/onnxruntime`
- License:
  - MIT License (ONNX Runtime)

Notes:
- The packaging script supports either a user-provided `libonnxruntime.dylib` or downloading a release build.

