use crate::{book::BookIllustration, book::BookMetadata, ffi};

/// Builder for a `kiwix::Filter`.
pub struct Filter(cxx::SharedPtr<ffi::Filter>);

unsafe impl Send for Filter {}
unsafe impl Sync for Filter {}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter {
    /// Create a new, empty filter.
    pub fn new() -> Self {
        Self(ffi::create_filter())
    }

    /// Include only valid books.
    pub fn valid(mut self, accept: bool) -> Self {
        unsafe { ffi::filter_valid(self.0.pin_mut_unchecked(), accept) };
        self
    }

    /// Include only local books.
    pub fn local(mut self, accept: bool) -> Self {
        unsafe { ffi::filter_local(self.0.pin_mut_unchecked(), accept) };
        self
    }

    /// Include only remote books.
    pub fn remote(mut self, accept: bool) -> Self {
        unsafe { ffi::filter_remote(self.0.pin_mut_unchecked(), accept) };
        self
    }

    /// Filter by full-text query.
    pub fn with_query(mut self, query: &str) -> Self {
        unsafe { ffi::filter_query(self.0.pin_mut_unchecked(), query) };
        self
    }

    /// Filter by language(s) as a comma-separated list.
    pub fn with_lang(mut self, lang: &str) -> Self {
        unsafe { ffi::filter_lang(self.0.pin_mut_unchecked(), lang) };
        self
    }

    /// Filter by category.
    pub fn category(mut self, category: &str) -> Self {
        unsafe { ffi::filter_category(self.0.pin_mut_unchecked(), category) };
        self
    }

    /// Filter by book name.
    pub fn name(mut self, name: &str) -> Self {
        unsafe { ffi::filter_name(self.0.pin_mut_unchecked(), name) };
        self
    }

    /// Filter by accepted tags.
    pub fn accept_tags(mut self, tags: &[String]) -> Self {
        let tags: Vec<String> = tags.to_vec();
        unsafe { ffi::filter_accept_tags(self.0.pin_mut_unchecked(), &tags) };
        self
    }

    /// Filter by rejected tags.
    pub fn reject_tags(mut self, tags: &[String]) -> Self {
        let tags: Vec<String> = tags.to_vec();
        unsafe { ffi::filter_reject_tags(self.0.pin_mut_unchecked(), &tags) };
        self
    }

    /// Filter by maximum size in bytes.
    pub fn max_size(mut self, size: usize) -> Self {
        unsafe { ffi::filter_max_size(self.0.pin_mut_unchecked(), size) };
        self
    }

    /// Return true if a query string has been set on this filter.
    pub fn has_query(&self) -> bool {
        ffi::filter_has_query(&self.0)
    }

    /// Return the query string set on this filter, if any.
    pub fn query(&self) -> Option<String> {
        get_optional_string(self.has_query(), || ffi::filter_get_query(&self.0))
    }

    /// Return true if a language filter has been set on this filter.
    pub fn has_lang(&self) -> bool {
        ffi::filter_has_lang(&self.0)
    }

    /// Return the language string set on this filter, if any.
    pub fn lang(&self) -> Option<String> {
        get_optional_string(self.has_lang(), || ffi::filter_get_lang(&self.0))
    }
}

fn get_optional_string<F>(has_value: bool, getter: F) -> Option<String>
where
    F: FnOnce() -> String,
{
    if has_value {
        let value = getter();
        if value.is_empty() { None } else { Some(value) }
    } else {
        None
    }
}

/// Create a new, empty Kiwix library.
pub fn new_library() -> crate::Library {
    ffi::create_library()
}

/// Return the IDs of books in the library matching the filter.
pub fn library_filter(library: &mut crate::Library, filter: &Filter) -> Vec<String> {
    // SAFETY: `Library` is an opaque C++ type; `filter` is a non-const member.
    unsafe { ffi::library_filter(library.pin_mut_unchecked(), &filter.0) }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Add a book to a library.
pub fn library_add_book(library: &mut crate::Library, book: &cxx::SharedPtr<ffi::Book>) -> bool {
    // SAFETY: `Library` is an opaque C++ type; `addBook` is thread-safe for
    // opaque types per the CXX `pin_mut_unchecked` documentation.
    unsafe { ffi::library_add_book(library.pin_mut_unchecked(), book.as_ref().unwrap()) }
}

/// Add a book to a library by opening the ZIM file at `path`.
///
/// Returns the book ID on success, or `None` if libkiwix could not read the
/// file or rejected it.
pub fn library_add_book_from_path(library: &mut crate::Library, path: &str) -> Option<String> {
    // SAFETY: `Library` is an opaque C++ type; `addBookFromPathAndGetId` is a
    // non-const member function documented as safe via `pin_mut_unchecked`.
    let id = unsafe { ffi::library_add_book_from_path(library.pin_mut_unchecked(), path) };
    if id.is_empty() { None } else { Some(id.to_string()) }
}

/// Look up a book by ID and return its metadata.
///
/// Returns `None` if no book with the given ID exists in the library.
pub fn library_get_book_metadata(library: &mut crate::Library, id: &str) -> Option<BookMetadata> {
    // SAFETY: `Library` is an opaque C++ type; `getBookByIdThreadSafe` is
    // invoked through a bridge function documented as safe via `pin_mut_unchecked`.
    let book = unsafe { ffi::library_get_book_by_id(library.pin_mut_unchecked(), id) };
    if book.is_null() {
        return None;
    }
    let book = book.as_ref().unwrap();
    Some(BookMetadata {
        id: ffi::book_get_id(book),
        name: ffi::book_get_name(book),
        date: ffi::book_get_date(book),
        flavour: ffi::book_get_flavour(book),
        title: ffi::book_get_title(book),
        description: ffi::book_get_description(book),
        language: ffi::book_get_language(book),
        creator: ffi::book_get_creator(book),
        publisher: ffi::book_get_publisher(book),
        category: ffi::book_get_category(book),
        tags: ffi::book_get_tags(book),
        url: ffi::book_get_url(book),
        article_count: ffi::book_get_article_count(book),
        media_count: ffi::book_get_media_count(book),
        size: ffi::book_get_size(book),
        path_valid: ffi::book_is_path_valid(book),
        illustrations: ffi::book_get_illustrations(book)
            .into_iter()
            .map(|ill| BookIllustration {
                width: ill.width,
                height: ill.height,
                mime_type: ill.mime_type.to_string(),
                url: ill.url.to_string(),
            })
            .collect(),
    })
}

/// A thread-safe handle to a `kiwix::Library`.
///
/// libkiwix's `Library` uses internal locking (`std::recursive_mutex`) and is
/// designed to be accessed from multiple threads. This wrapper asserts that to
/// Rust's type system so it can be shared across async tasks and threads.
#[derive(Clone)]
pub struct LibraryHandle(crate::Library);

unsafe impl Send for LibraryHandle {}
unsafe impl Sync for LibraryHandle {}

impl LibraryHandle {
    /// Wrap a raw `Library` handle.
    pub fn new(library: crate::Library) -> Self {
        Self(library)
    }

    /// Consume the handle and return the underlying `Library`.
    pub fn into_inner(self) -> crate::Library {
        self.0
    }
}

impl std::ops::Deref for LibraryHandle {
    type Target = crate::Library;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LibraryHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
