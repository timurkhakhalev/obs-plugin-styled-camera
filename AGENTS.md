# Agent notes (Codex / contributors)

## Quick commands

- Init submodules: `git submodule update --init --recursive`
- Tests: `cargo test` (tests live in `crates/styledcamera-core/`)
- Build plugin: `cargo build -p styledcamera`

## Project structure

- `crates/styledcamera/` is the OBS plugin crate (`cdylib`). It is loaded by OBS, so avoid adding runtime work to the render thread.
- `crates/styledcamera-core/` is where “pure” logic goes (mask postprocess, math, etc.). Prefer adding tests here.
- `data/effects/` contains the shader pipeline used by the plugin.
- `PACKAGING.md` documents how release artifacts are assembled.

## Conventions

- Keep OBS/FFI surfaces narrow and localized (prefer wrapper functions/modules).
- Put testable logic into `styledcamera-core` and cover it with unit tests.
- Avoid large refactors that mix behavior changes with formatting/renames.

## Releases / versioning

- Single source of truth: bump `[workspace.package].version` in `Cargo.toml` (all crates use `version.workspace = true`).
- Versioning scheme: SemVer (`MAJOR.MINOR.PATCH`)
  - `PATCH`: bugfixes / internal changes
  - `MINOR`: backwards-compatible features
  - `MAJOR`: breaking changes
- GitHub release flow (macOS arm64):
  - Push a tag matching the Cargo version: `vX.Y.Z`
  - The workflow `.github/workflows/release-macos.yml` builds + packages and attaches `StyledCamera-macos-arm64-vX.Y.Z.zip` to the GitHub Release.
- Local packaging (macOS arm64): `./scripts/package-macos-arm64.sh --download-onnxruntime` (see `PACKAGING.md`).
