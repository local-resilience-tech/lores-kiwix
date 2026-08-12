use libkiwix_rust::{self as kiwix, BookMetadata};

/// A ZIM file that was successfully added to the Kiwix library.
#[derive(Debug, Clone)]
pub struct RegisteredZim {
    pub path: String,
    pub metadata: BookMetadata,
}

#[derive(Debug, Clone)]
pub enum AddZimError {
    AddFailed,
    MetadataMissing { book_id: String },
}

impl std::fmt::Display for AddZimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddZimError::AddFailed => write!(f, "failed to add ZIM to library"),
            AddZimError::MetadataMissing { book_id } => {
                write!(f, "book added but metadata missing (book_id={})", book_id)
            }
        }
    }
}

impl std::error::Error for AddZimError {}

/// Add the file or directory at `path` to the Kiwix library.
///
/// Returns a list of [`RegisteredZim`] entries for each ZIM file that was
/// successfully added to the library.
pub fn add_path_to_library(library: &mut kiwix::Library, path: &str) -> Vec<RegisteredZim> {
    let meta = std::fs::metadata(path).expect("cannot access path");
    let mut registered = Vec::new();

    if meta.is_file() {
        match add_zim(library, path) {
            Ok(zim) => registered.push(zim),
            Err(e) => eprintln!("Failed to add {}: {}", path, e),
        }
        return registered;
    }

    if meta.is_dir() {
        for entry in std::fs::read_dir(path).expect("cannot read directory") {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zim") {
                let path_str = path.to_str().unwrap();
                match add_zim(library, path_str) {
                    Ok(zim) => registered.push(zim),
                    Err(e) => eprintln!("Failed to add {}: {}", path_str, e),
                }
            }
        }
        return registered;
    }

    panic!("path is neither a file nor a directory: {}", path);
}

pub fn add_zim(library: &mut kiwix::Library, path: &str) -> Result<RegisteredZim, AddZimError> {
    let Some(book_id) = kiwix::library_add_book_from_path(library, path) else {
        return Err(AddZimError::AddFailed);
    };
    let Some(metadata) = kiwix::library_get_book_metadata(library, &book_id) else {
        return Err(AddZimError::MetadataMissing { book_id });
    };
    eprintln!("Added: {} (id={})", path, metadata.id);
    Ok(RegisteredZim {
        path: path.to_string(),
        metadata,
    })
}
