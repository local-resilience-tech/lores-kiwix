//! Minimal Rust bindings to libkiwix via CXX.
//!
//! This crate dynamically links against libkiwix discovered through pkg-config.
//! It does not bundle or vendor libkiwix.

#[cxx::bridge]
mod ffi {
    /// Information about a book illustration.
    #[derive(Debug, Clone)]
    pub struct BookIllustration {
        pub width: u16,
        pub height: u16,
        pub mime_type: String,
        pub url: String,
    }

    unsafe extern "C++" {
        include!("src/bridge.h");

        /// Opaque handle to a `kiwix::Book`.
        #[namespace = "kiwix"]
        type Book;

        /// Opaque handle to a `kiwix::Library`.
        #[namespace = "kiwix"]
        type Library;

        /// Opaque handle to a `kiwix::Filter`.
        #[namespace = "kiwix"]
        type Filter;

        /// Opaque handle to a `kiwix::Server`.
        #[namespace = "kiwix"]
        type Server;

        /// Create a new, empty `kiwix::Library`.
        fn create_library() -> SharedPtr<Library>;

        /// Create a new, empty `kiwix::Book`.
        fn create_book() -> SharedPtr<Book>;

        /// Create a new, empty `kiwix::Filter`.
        fn create_filter() -> SharedPtr<Filter>;
        /// Include only valid books in the filter.
        fn filter_valid(filter: Pin<&mut Filter>, accept: bool) -> Pin<&mut Filter>;
        /// Include only local books in the filter.
        fn filter_local(filter: Pin<&mut Filter>, accept: bool) -> Pin<&mut Filter>;
        /// Include only remote books in the filter.
        fn filter_remote(filter: Pin<&mut Filter>, accept: bool) -> Pin<&mut Filter>;
        /// Filter by full-text query.
        fn filter_query<'a>(filter: Pin<&'a mut Filter>, query: &str) -> Pin<&'a mut Filter>;
        /// Filter by language(s) as a comma-separated list.
        fn filter_lang<'a>(filter: Pin<&'a mut Filter>, lang: &str) -> Pin<&'a mut Filter>;
        /// Filter by category.
        fn filter_category<'a>(filter: Pin<&'a mut Filter>, category: &str) -> Pin<&'a mut Filter>;
        /// Filter by book name.
        fn filter_name<'a>(filter: Pin<&'a mut Filter>, name: &str) -> Pin<&'a mut Filter>;
        /// Filter by accepted tags.
        fn filter_accept_tags<'a>(filter: Pin<&'a mut Filter>, tags: &Vec<String>) -> Pin<&'a mut Filter>;
        /// Filter by rejected tags.
        fn filter_reject_tags<'a>(filter: Pin<&'a mut Filter>, tags: &Vec<String>) -> Pin<&'a mut Filter>;
        /// Filter by maximum size in bytes.
        fn filter_max_size(filter: Pin<&mut Filter>, size: usize) -> Pin<&mut Filter>;

        /// Return the IDs of books matching the filter.
        fn library_filter(library: Pin<&mut Library>, filter: &Filter) -> Vec<String>;

        /// Set the filesystem path of a book (usually a `.zim` file).
        fn book_set_path(book: Pin<&mut Book>, path: &str);

        /// Add a book to a library. Returns `true` if a new book was added,
        /// `false` if an existing book was updated.
        fn library_add_book(library: Pin<&mut Library>, book: &Book) -> bool;

        /// Add a book to a library by path, opening the ZIM and populating
        /// metadata. Returns the book ID, or an empty string on failure.
        fn library_add_book_from_path(library: Pin<&mut Library>, path: &str) -> String;

        /// Look up a book by ID. Returns `null` if no book with that ID exists.
        fn library_get_book_by_id(library: Pin<&mut Library>, id: &str) -> SharedPtr<Book>;

        /// Get the embedded ZIM UUID of the book.
        fn book_get_id(book: &Book) -> String;
        /// Get the archive name (e.g. `wikipedia_en_all_maxi`).
        fn book_get_name(book: &Book) -> String;
        /// Get the archive date.
        fn book_get_date(book: &Book) -> String;
        /// Get the archive flavour (e.g. `maxi`, `mini`, `nopic`).
        fn book_get_flavour(book: &Book) -> String;
        /// Get the archive title.
        fn book_get_title(book: &Book) -> String;
        /// Get the archive description.
        fn book_get_description(book: &Book) -> String;
        /// Get the archive language(s) as a comma-separated list.
        fn book_get_language(book: &Book) -> String;
        /// Get the archive creator.
        fn book_get_creator(book: &Book) -> String;
        /// Get the archive publisher.
        fn book_get_publisher(book: &Book) -> String;
        /// Get the archive category.
        fn book_get_category(book: &Book) -> String;
        /// Get the archive tags string.
        fn book_get_tags(book: &Book) -> String;
        /// Get the archive remote URL, if any.
        fn book_get_url(book: &Book) -> String;
        /// Get the number of articles in the archive.
        fn book_get_article_count(book: &Book) -> u64;
        /// Get the number of media files in the archive.
        fn book_get_media_count(book: &Book) -> u64;
        /// Get the archive size in bytes.
        fn book_get_size(book: &Book) -> u64;
        /// Return true if the archive path is valid.
        fn book_is_path_valid(book: &Book) -> bool;

        /// Get the list of illustrations for the book.
        fn book_get_illustrations(book: &Book) -> Vec<BookIllustration>;

        /// Create a server for the given library.
        fn create_server(library: SharedPtr<Library>) -> SharedPtr<Server>;

        /// Set the address the server binds to (e.g. `"127.0.0.1"` or `"::"`).
        fn server_set_address(server: Pin<&mut Server>, address: &str);

        /// Set the TCP port the server listens on.
        fn server_set_port(server: Pin<&mut Server>, port: i32);

        /// Set the IP protocol mode (IPv4=0, IPv6=1, ALL=2, AUTO=3).
        fn server_set_ip_mode(server: Pin<&mut Server>, mode: i32);

        /// Start the server. Returns `true` on success.
        fn server_start(server: Pin<&mut Server>) -> bool;

        /// Stop a running server.
        fn server_stop(server: Pin<&mut Server>);
    }
}

/// Create a new, empty Kiwix library.
pub fn new_library() -> Library {
    ffi::create_library()
}

/// Create a new, empty book.
pub fn new_book() -> cxx::SharedPtr<ffi::Book> {
    ffi::create_book()
}

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
    pub fn query(mut self, query: &str) -> Self {
        unsafe { ffi::filter_query(self.0.pin_mut_unchecked(), query) };
        self
    }

    /// Filter by language(s) as a comma-separated list.
    pub fn lang(mut self, lang: &str) -> Self {
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
}

/// Return the IDs of books in the library matching the filter.
pub fn library_filter(library: &mut Library, filter: &Filter) -> Vec<String> {
    unsafe { ffi::library_filter(library.pin_mut_unchecked(), &filter.0) }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Set the filesystem path of a book.
pub fn book_set_path(book: &mut cxx::SharedPtr<ffi::Book>, path: &str) {
    // SAFETY: `Book` is an opaque C++ type; `setPath` is a non-const member
    // function and `pin_mut_unchecked` is documented as safe for this case.
    unsafe { ffi::book_set_path(book.pin_mut_unchecked(), path) };
}

/// Add a book to a library.
pub fn library_add_book(library: &mut Library, book: &cxx::SharedPtr<ffi::Book>) -> bool {
    // SAFETY: `Library` is an opaque C++ type; `addBook` is thread-safe for
    // opaque types per the CXX `pin_mut_unchecked` documentation.
    unsafe { ffi::library_add_book(library.pin_mut_unchecked(), book.as_ref().unwrap()) }
}

/// Add a book to a library by opening the ZIM file at `path`.
///
/// Returns the book ID on success, or `None` if libkiwix could not read the
/// file or rejected it.
pub fn library_add_book_from_path(library: &mut Library, path: &str) -> Option<String> {
    // SAFETY: `Library` is an opaque C++ type; `addBookFromPathAndGetId` is a
    // non-const member function documented as safe via `pin_mut_unchecked`.
    let id = unsafe { ffi::library_add_book_from_path(library.pin_mut_unchecked(), path) };
    if id.is_empty() { None } else { Some(id.to_string()) }
}

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

/// Look up a book by ID and return its metadata.
///
/// Returns `None` if no book with the given ID exists in the library.
pub fn library_get_book_metadata(library: &mut Library, id: &str) -> Option<BookMetadata> {
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

/// Server address configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: i32,
    pub ip_mode: IpMode,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 8080,
            ip_mode: IpMode::Auto,
        }
    }
}

/// Handle to a `kiwix::Library`.
pub type Library = cxx::SharedPtr<ffi::Library>;
/// Handle to a `kiwix::Server`.
pub type Server = cxx::SharedPtr<ffi::Server>;

/// A thread-safe handle to a `kiwix::Library`.
///
/// libkiwix's `Library` uses internal locking (`std::recursive_mutex`) and is
/// designed to be accessed from multiple threads. This wrapper asserts that to
/// Rust's type system so it can be shared across async tasks and threads.
#[derive(Clone)]
pub struct LibraryHandle(Library);

unsafe impl Send for LibraryHandle {}
unsafe impl Sync for LibraryHandle {}

impl LibraryHandle {
    /// Wrap a raw `Library` handle.
    pub fn new(library: Library) -> Self {
        Self(library)
    }

    /// Consume the handle and return the underlying `Library`.
    pub fn into_inner(self) -> Library {
        self.0
    }
}

impl std::ops::Deref for LibraryHandle {
    type Target = Library;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LibraryHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// IP protocol selection for the server.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IpMode {
    Ipv4 = 0,
    Ipv6 = 1,
    All = 2,
    Auto = 3,
}

impl IpMode {
    fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Create and configure a Kiwix server.
pub fn new_server(library: Library, config: &ServerConfig) -> Server {
    let mut server = ffi::create_server(library);
    // SAFETY: `Server` is an opaque C++ type; these setters are non-const
    // member functions documented as safe to call via `pin_mut_unchecked`.
    unsafe {
        ffi::server_set_address(server.pin_mut_unchecked(), &config.address);
        ffi::server_set_port(server.pin_mut_unchecked(), config.port);
        ffi::server_set_ip_mode(server.pin_mut_unchecked(), config.ip_mode.as_i32());
    }
    server
}

/// Start the server.
pub fn server_start(server: &mut Server) -> bool {
    // SAFETY: `Server` is an opaque C++ type; `start` is a non-const member.
    unsafe { ffi::server_start(server.pin_mut_unchecked()) }
}

/// Stop the server.
pub fn server_stop(server: &mut Server) {
    // SAFETY: `Server` is an opaque C++ type; `stop` is a non-const member.
    unsafe { ffi::server_stop(server.pin_mut_unchecked()) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_library() {
        let _lib = new_library();
    }
}
