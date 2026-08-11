#include "bridge.h"
#pragma diag_suppress 1696
#include "libkiwix-rust/src/lib.rs.h"
#pragma diag_default 1696

#include <cstdint>

std::shared_ptr<kiwix::Library> create_library() {
    return kiwix::Library::create();
}

std::shared_ptr<kiwix::Book> create_book() {
    return std::make_shared<kiwix::Book>();
}

void book_set_path(kiwix::Book& book, rust::Str path) {
    book.setPath(std::string(path));
}

bool library_add_book(kiwix::Library& library, const kiwix::Book& book) {
    return library.addBook(book);
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
