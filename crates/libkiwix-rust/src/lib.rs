//! Minimal Rust bindings to libkiwix via CXX.
//!
//! This crate dynamically links against libkiwix discovered through pkg-config.
//! It does not bundle or vendor libkiwix.

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("src/bridge.h");

        /// Opaque handle to a `kiwix::Book`.
        #[namespace = "kiwix"]
        type Book;

        /// Opaque handle to a `kiwix::Library`.
        #[namespace = "kiwix"]
        type Library;

        /// Opaque handle to a `kiwix::Server`.
        #[namespace = "kiwix"]
        type Server;

        /// Create a new, empty `kiwix::Library`.
        fn create_library() -> SharedPtr<Library>;

        /// Create a new, empty `kiwix::Book`.
        fn create_book() -> SharedPtr<Book>;

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
    Some(BookMetadata {
        id: ffi::book_get_id(book.as_ref().unwrap()),
        name: ffi::book_get_name(book.as_ref().unwrap()),
        date: ffi::book_get_date(book.as_ref().unwrap()),
        flavour: ffi::book_get_flavour(book.as_ref().unwrap()),
        title: ffi::book_get_title(book.as_ref().unwrap()),
        description: ffi::book_get_description(book.as_ref().unwrap()),
        language: ffi::book_get_language(book.as_ref().unwrap()),
        creator: ffi::book_get_creator(book.as_ref().unwrap()),
        publisher: ffi::book_get_publisher(book.as_ref().unwrap()),
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
