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

        /// Return true if the filter has a query set.
        fn filter_has_query(filter: &Filter) -> bool;
        /// Get the filter's query string, if any.
        fn filter_get_query(filter: &Filter) -> String;

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

mod book;
mod library;
mod server;

pub use book::{BookIllustration, BookMetadata, book_set_path, new_book};
pub use library::{
    Filter, LibraryHandle, library_add_book, library_add_book_from_path, library_filter, library_get_book_metadata,
    new_library,
};
pub use server::{IpMode, ServerConfig, new_server, server_start, server_stop};

/// Handle to a `kiwix::Library`.
pub type Library = cxx::SharedPtr<ffi::Library>;
/// Handle to a `kiwix::Server`.
pub type Server = cxx::SharedPtr<ffi::Server>;

#[cfg(test)]
mod tests {
    use crate::new_library;

    #[test]
    fn can_create_library() {
        let _lib = new_library();
    }
}
