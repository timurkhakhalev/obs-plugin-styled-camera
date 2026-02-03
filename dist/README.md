# dist/ (generated artifacts)

This folder is the default output location for packaging scripts.

Expected macOS output (arm64 packaging):

```
dist/
  StyledCamera-macos-arm64.zip
  macos/
    StyledCamera.plugin/
      Contents/
        MacOS/
          StyledCamera          (plugin binary/module; produced by your build)
        Frameworks/
          libonnxruntime.dylib  (bundled by packaging script)
        Resources/
          NOTICE.md
          *.effect
          models/
            selfie_segmentation.onnx
```

Notes:
- The packaging scripts overwrite `dist/macos/StyledCamera.plugin` on each run.
