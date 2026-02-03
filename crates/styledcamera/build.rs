use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple-darwin") {
        // OBS plugins on macOS resolve libobs symbols from the host application (OBS) at runtime.
        // CI environments typically don't have OBS.app installed, so we allow unresolved symbols
        // at link time.
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}

