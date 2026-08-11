//! Minimal Rust bindings to libkiwix via CXX.
//!
//! This crate dynamically links against libkiwix discovered through pkg-config.
//! It does not bundle or vendor libkiwix.

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("src/bridge.h");

        /// Opaque handle to a `kiwix::Library`.
        #[namespace = "kiwix"]
        type Library;

        /// Create a new, empty `kiwix::Library`.
        fn create_library() -> SharedPtr<Library>;
    }
}

/// Create a new, empty Kiwix library.
pub fn new_library() -> cxx::SharedPtr<ffi::Library> {
    ffi::create_library()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_library() {
        let _lib = new_library();
    }
}
