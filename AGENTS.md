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

