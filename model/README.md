# Model files

This folder is for model artifacts used by StyledCamera (e.g. person segmentation ONNX).

- Download the default model:
  - `./scripts/fetch-model.sh`
- Output path (default):
  - `model/mediapipe_selfie_segmentation.onnx`

Notes:
- Keep large model binaries out of Git unless you intentionally want to vendor them.
- Packaging scripts copy the model into the plugin bundle under `Contents/Resources/models/`.

