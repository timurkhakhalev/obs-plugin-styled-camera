# `obs-sys`

Rust FFI bindings for OBS Studio (`libobs`), generated with `bindgen`.

## Setup

This repo vendors OBS Studio sources as a git submodule:

```sh
git submodule update --init --recursive
```

## Environment variables

- `OBS_SYS_HEADERS=/path/to/obs-studio`  
  Override the default `vendor/obs-studio` headers location.
- `OBS_APP_BUNDLE=/path/to/OBS.app` (macOS)  
  Link against an installed OBS.app bundle (defaults to `/Applications/OBS.app` if present).
- `BINDGEN_EXTRA_CLANG_ARGS="..."`  
  Extra `clang` args passed to bindgen (e.g. additional `-I...` include paths).

