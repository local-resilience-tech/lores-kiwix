fn main() {
    // Discover libkiwix via pkg-config. The crate does not hardcode any local
    // path. Either install libkiwix system-wide, or build it locally and point
    // PKG_CONFIG_PATH at its lib/pkgconfig directory.
    let libkiwix = pkg_config::Config::new()
        .atleast_version("14.0")
        .probe("libkiwix")
        .expect(
            "libkiwix not found via pkg-config. \
             Build/install libkiwix and set PKG_CONFIG_PATH if it is not in the default path.",
        );

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
