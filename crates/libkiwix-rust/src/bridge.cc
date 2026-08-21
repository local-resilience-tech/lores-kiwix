#include "bridge.h"
#pragma diag_suppress 1696
#include "libkiwix-rust/src/lib.rs.h"
#pragma diag_default 1696

#include <cstdint>
#include <kiwix/manager.h>

std::shared_ptr<kiwix::Library> create_library() {
    return kiwix::Library::create();
}

std::shared_ptr<kiwix::Book> create_book() {
    return std::make_shared<kiwix::Book>();
}

std::shared_ptr<kiwix::Filter> create_filter() {
    return std::make_shared<kiwix::Filter>();
}

kiwix::Filter& filter_valid(kiwix::Filter& filter, bool accept) {
    return filter.valid(accept);
}

kiwix::Filter& filter_local(kiwix::Filter& filter, bool accept) {
    return filter.local(accept);
}

kiwix::Filter& filter_remote(kiwix::Filter& filter, bool accept) {
    return filter.remote(accept);
}

kiwix::Filter& filter_query(kiwix::Filter& filter, rust::Str query) {
    return filter.query(std::string(query));
}

kiwix::Filter& filter_lang(kiwix::Filter& filter, rust::Str lang) {
    return filter.lang(std::string(lang));
}

kiwix::Filter& filter_category(kiwix::Filter& filter, rust::Str category) {
    return filter.category(std::string(category));
}

kiwix::Filter& filter_name(kiwix::Filter& filter, rust::Str name) {
    return filter.name(std::string(name));
}

kiwix::Filter& filter_accept_tags(kiwix::Filter& filter, const rust::Vec<rust::String>& tags) {
    std::vector<std::string> vec(tags.begin(), tags.end());
    return filter.acceptTags(vec);
}

kiwix::Filter& filter_reject_tags(kiwix::Filter& filter, const rust::Vec<rust::String>& tags) {
    std::vector<std::string> vec(tags.begin(), tags.end());
    return filter.rejectTags(vec);
}

kiwix::Filter& filter_max_size(kiwix::Filter& filter, size_t size) {
    return filter.maxSize(size);
}

bool filter_has_query(const kiwix::Filter& filter) {
    return filter.hasQuery();
}

rust::String filter_get_query(const kiwix::Filter& filter) {
    return rust::String(filter.getQuery());
}

bool filter_has_lang(const kiwix::Filter& filter) {
    return filter.hasLang();
}

rust::String filter_get_lang(const kiwix::Filter& filter) {
    return rust::String(filter.getLang());
}

rust::Vec<rust::String> library_filter(kiwix::Library& library, const kiwix::Filter& filter) {
    const auto bookIds = library.filter(filter);
    rust::Vec<rust::String> result;
    for (const auto& id : bookIds) {
        result.push_back(rust::String(id));
    }
    return result;
}

void book_set_path(kiwix::Book& book, rust::Str path) {
    book.setPath(std::string(path));
}

bool library_add_book(kiwix::Library& library, const kiwix::Book& book) {
    return library.addBook(book);
}

rust::String library_add_book_from_path(kiwix::Library& library, rust::Str path) {
    kiwix::Manager manager(library.shared_from_this());
    return rust::String(manager.addBookFromPathAndGetId(std::string(path)));
}
std::shared_ptr<kiwix::Book> library_get_book_by_id(kiwix::Library& library, rust::Str id) {
  try {
    auto book = library.getBookByIdThreadSafe(std::string(id));
    return std::make_shared<kiwix::Book>(std::move(book));
  } catch (...) {
    return nullptr;
  }
}

rust::String book_get_id(const kiwix::Book& book) { return rust::String(book.getId()); }
rust::String book_get_name(const kiwix::Book& book) { return rust::String(book.getName()); }
rust::String book_get_date(const kiwix::Book& book) { return rust::String(book.getDate()); }
rust::String book_get_flavour(const kiwix::Book& book) { return rust::String(book.getFlavour()); }
rust::String book_get_title(const kiwix::Book& book) { return rust::String(book.getTitle()); }
rust::String book_get_description(const kiwix::Book& book) { return rust::String(book.getDescription()); }
rust::String book_get_language(const kiwix::Book& book) { return rust::String(book.getCommaSeparatedLanguages()); }
rust::String book_get_creator(const kiwix::Book& book) { return rust::String(book.getCreator()); }
rust::String book_get_publisher(const kiwix::Book& book) { return rust::String(book.getPublisher()); }
rust::String book_get_category(const kiwix::Book& book) { return rust::String(book.getCategory()); }
rust::String book_get_tags(const kiwix::Book& book) { return rust::String(book.getTags()); }
rust::String book_get_url(const kiwix::Book& book) { return rust::String(book.getUrl()); }
uint64_t book_get_article_count(const kiwix::Book& book) { return book.getArticleCount(); }
uint64_t book_get_media_count(const kiwix::Book& book) { return book.getMediaCount(); }
uint64_t book_get_size(const kiwix::Book& book) { return book.getSize(); }
bool book_is_path_valid(const kiwix::Book& book) { return book.isPathValid(); }

rust::Vec<BookIllustration> book_get_illustrations(const kiwix::Book& book) {
    rust::Vec<BookIllustration> result;
    for (const auto& illustration : book.getIllustrations()) {
        result.push_back(BookIllustration{
            illustration->width,
            illustration->height,
            rust::String(illustration->mimeType),
            rust::String(illustration->url)
        });
    }
    return result;
}
std::shared_ptr<kiwix::Server> create_server(std::shared_ptr<kiwix::Library> library) {
    return std::make_shared<kiwix::Server>(library);
}

void server_set_address(kiwix::Server& server, rust::Str address) {
    server.setAddress(std::string(address));
}

void server_set_port(kiwix::Server& server, int port) {
    server.setPort(port);
}

void server_set_ip_mode(kiwix::Server& server, std::int32_t mode) {
    server.setIpMode(static_cast<kiwix::IpMode>(mode));
}

bool server_start(kiwix::Server& server) {
    return server.start();
}

void server_stop(kiwix::Server& server) {
    server.stop();
}
