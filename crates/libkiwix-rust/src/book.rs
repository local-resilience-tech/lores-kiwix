use crate::ffi;

/// Metadata about a book illustration.
#[derive(Debug, Clone)]
pub struct BookIllustration {
    pub width: u16,
    pub height: u16,
    pub mime_type: String,
    pub url: String,
}

/// Metadata extracted from a ZIM archive.
#[derive(Debug, Clone)]
pub struct BookMetadata {
    pub id: String,
    pub name: String,
    pub date: String,
    pub flavour: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub creator: String,
    pub publisher: String,
    pub category: String,
    pub tags: String,
    pub url: String,
    pub article_count: u64,
    pub media_count: u64,
    pub size: u64,
    pub path_valid: bool,
    pub illustrations: Vec<BookIllustration>,
}

/// Create a new, empty book.
pub fn new_book() -> cxx::SharedPtr<ffi::Book> {
    ffi::create_book()
}

/// Set the filesystem path of a book.
pub fn book_set_path(book: &mut cxx::SharedPtr<ffi::Book>, path: &str) {
    // SAFETY: `Book` is an opaque C++ type; `setPath` is a non-const member
    // function and `pin_mut_unchecked` is documented as safe for this case.
    unsafe { ffi::book_set_path(book.pin_mut_unchecked(), path) };
}
