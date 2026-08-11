use libkiwix_rust::{self as kiwix};

pub fn add_path_to_library(library: &mut kiwix::Library, path: &str) {
    let meta = std::fs::metadata(path).expect("cannot access path");

    if meta.is_file() {
        add_zim(library, path);
        return;
    }

    if meta.is_dir() {
        for entry in std::fs::read_dir(path).expect("cannot read directory") {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zim") {
                add_zim(library, path.to_str().unwrap());
            }
        }
        return;
    }

    panic!("path is neither a file nor a directory: {}", path);
}

pub fn add_zim(library: &mut kiwix::Library, path: &str) {
    match kiwix::library_add_book_from_path(library, path) {
        Some(id) => eprintln!("Added: {} (id={})", path, id),
        None => eprintln!("Failed to add: {}", path),
    }
}
