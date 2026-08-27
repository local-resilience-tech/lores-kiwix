// The libkiwix major version this crate's bridge is written against. The bridge
// (src/bridge.cc / src/bridge.h) assumes the API/ABI shape of this major
// version; any minor/patch release within it is accepted.
const LIBKIWIX_MAJOR: u32 = 14;

fn main() {
    let min = format!("{LIBKIWIX_MAJOR}.0");
    let max_exclusive = format!("{}.0", LIBKIWIX_MAJOR + 1);

    // Discover libkiwix via pkg-config. The crate does not hardcode any local
    // path. Either install libkiwix system-wide, or build it locally and point
    // PKG_CONFIG_PATH at its lib/pkgconfig directory.
    let libkiwix = pkg_config::Config::new()
        .range_version(min.as_str()..max_exclusive.as_str())
        .probe("libkiwix")
        .unwrap_or_else(|e| {
            panic!(
                "libkiwix not found (or wrong version) via pkg-config. \
                 This crate targets libkiwix major version {LIBKIWIX_MAJOR} \
                 (>= {min}, < {max_exclusive}). Build/install a matching libkiwix \
                 and set PKG_CONFIG_PATH if it is not in the default path.\n\
                 pkg-config error: {e}"
            )
        });

    // Surface the exact version that was linked so it appears in build logs and
    // can be embedded/verified at runtime via env!("LIBKIWIX_VERSION").
    println!("cargo:rustc-env=LIBKIWIX_VERSION={}", libkiwix.version);
    println!("cargo:warning=linking libkiwix {}", libkiwix.version);
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    let mut build = cxx_build::bridge("src/lib.rs");
    build
        .file("src/bridge.cc")
        .cpp(true)
        .std("c++17")
        .flag_if_supported("-Wno-unused-parameter");

    for path in &libkiwix.include_paths {
        build.include(path);
    }

    // cxx_build::bridge already adds OUT_DIR to the include paths, but make it
    // explicit so the generated `src/lib.rs.h` header is found reliably.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    build.include(&out_dir);

    // Make crate-local headers (e.g. src/bridge.h) findable.
    let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    build.include(&crate_root);

    build.compile("libkiwix-rust-bridge");
}
