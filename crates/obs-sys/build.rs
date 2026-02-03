use std::env;
use std::path::{Path, PathBuf};

fn main() {
  println!("cargo:rerun-if-env-changed=OBS_APP_BUNDLE");
  println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
  println!("cargo:rerun-if-env-changed=OBS_SYS_HEADERS");
  println!("cargo:rerun-if-changed=wrapper.h");
  println!("cargo:rerun-if-changed=include/obsconfig.h");
  println!("cargo:rerun-if-changed=include/simde/x86/sse2.h");

  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));

  let (obs_headers_root, obs_headers_hint) = match env::var_os("OBS_SYS_HEADERS") {
    Some(p) => (PathBuf::from(p), Some("OBS_SYS_HEADERS".to_string())),
    None => {
      let workspace_root = manifest_dir
        .ancestors()
        .nth(2)
        .expect("expected crates/obs-sys to be nested under workspace root");
      (workspace_root.join("vendor/obs-studio"), None)
    }
  };

  let libobs_include = obs_headers_root.join("libobs");
  if !libobs_include.exists() {
    let mut msg = String::new();
    msg.push_str("obs-sys: OBS headers not found.\n");
    if let Some(hint) = obs_headers_hint {
      msg.push_str(&format!(
        "  {} points to: {}\n",
        hint,
        obs_headers_root.display()
      ));
    } else {
      msg.push_str(&format!(
        "  expected submodule checkout at: {}\n",
        obs_headers_root.display()
      ));
      msg.push_str("  did you run: git submodule update --init --recursive ?\n");
    }
    msg.push_str("  You can override by setting OBS_SYS_HEADERS=/path/to/obs-studio.\n");
    panic!("{msg}");
  }

  // If the OBS headers change, regenerate bindings.
  println!(
    "cargo:rerun-if-changed={}",
    libobs_include.join("obs-module.h").display()
  );
  println!(
    "cargo:rerun-if-changed={}",
    libobs_include.join("obs.h").display()
  );

  generate_bindings(&manifest_dir, &libobs_include);

  let target = env::var("TARGET").unwrap_or_default();
  if target.contains("apple-darwin") {
    maybe_link_macos_obs_app();
  }
}

fn generate_bindings(manifest_dir: &Path, libobs_include: &Path) {
  let wrapper = manifest_dir.join("wrapper.h");

  let mut builder = bindgen::Builder::default()
    .header(wrapper.to_string_lossy().to_string())
    .clang_arg(format!("-I{}", manifest_dir.join("include").display()))
    .clang_arg(format!("-I{}", libobs_include.display()))
    .clang_arg("-std=c11")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .derive_default(true)
    .layout_tests(false)
    .generate_comments(false);

  if let Ok(extra) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
    for arg in extra.split_whitespace().filter(|s| !s.is_empty()) {
      builder = builder.clang_arg(arg);
    }
  }

  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
  let out_path = out_dir.join("bindings.rs");

  let bindings = builder.generate().expect("obs-sys: bindgen failed");

  bindings
    .write_to_file(&out_path)
    .expect("obs-sys: failed to write bindings");
}

fn maybe_link_macos_obs_app() {
  let bundle = env::var_os("OBS_APP_BUNDLE")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/Applications/OBS.app"));

  if !bundle.exists() {
    println!(
      "cargo:warning=obs-sys: OBS.app not found at {} (set OBS_APP_BUNDLE=/path/to/OBS.app to link against an installed OBS).",
      bundle.display()
    );
    return;
  }

  let frameworks_dir = bundle.join("Contents/Frameworks");
  if !frameworks_dir.is_dir() {
    println!(
      "cargo:warning=obs-sys: expected Frameworks directory at {} (set OBS_APP_BUNDLE to a valid OBS.app).",
      frameworks_dir.display()
    );
    return;
  }

  // OBS macOS distribution ships libobs as a framework.
  let libobs_framework = frameworks_dir.join("libobs.framework");
  if !libobs_framework.is_dir() {
    println!(
      "cargo:warning=obs-sys: libobs.framework not found in {}. Linking is skipped.",
      frameworks_dir.display()
    );
    return;
  }

  println!("cargo:rustc-link-search=framework={}", frameworks_dir.display());
  println!("cargo:rustc-link-lib=framework=libobs");

  // Useful when running binaries outside of OBS (for quick dev tools/tests).
  println!(
    "cargo:rustc-link-arg=-Wl,-rpath,{}",
    frameworks_dir.display()
  );
}
