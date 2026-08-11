use libkiwix_rust::{self as kiwix};

/// Add the file or directory at `path` to the Kiwix library.
///
/// Returns a list of `(path, book_id)` pairs for each ZIM file that was
/// successfully added to the library.
pub fn add_path_to_library(library: &mut kiwix::Library, path: &str) -> Vec<(String, String)> {
    let meta = std::fs::metadata(path).expect("cannot access path");
    let mut registered = Vec::new();

    if meta.is_file() {
        if let Some(id) = add_zim(library, path) {
            registered.push((path.to_string(), id));
        }
        return registered;
    }

    if meta.is_dir() {
        for entry in std::fs::read_dir(path).expect("cannot read directory") {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zim") {
                let path_str = path.to_str().unwrap();
                if let Some(id) = add_zim(library, path_str) {
                    registered.push((path_str.to_string(), id));
                }
            }
        }
        return registered;
    }

    panic!("path is neither a file nor a directory: {}", path);
}

pub fn add_zim(library: &mut kiwix::Library, path: &str) -> Option<String> {
    match kiwix::library_add_book_from_path(library, path) {
        Some(id) => {
            eprintln!("Added: {} (id={})", path, id);
            Some(id)
        }
        None => {
            eprintln!("Failed to add: {}", path);
            None
        }
    }
}
