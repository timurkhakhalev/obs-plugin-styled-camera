# Contributing

Thanks for contributing!

## Development setup

1) Install Rust (stable).
2) Initialize the OBS submodule:

```bash
git submodule update --init --recursive
```

3) Run tests:

```bash
cargo test
```

## What to work on

- Bugs, stability, and performance improvements.
- Improvements to the shader pipeline and settings UX.
- More unit-test coverage inside `crates/styledcamera-core/`.

## Notes

- `crates/styledcamera/` links against OBS (`libobs`) and is intended to run inside OBS.
- Please keep third-party attribution up to date in `third_party/NOTICE.md` if packaging changes.

