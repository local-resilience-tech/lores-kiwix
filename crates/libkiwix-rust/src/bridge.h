#pragma once
#include <cstdint>
#include <memory>
#include <string>
#include <rust/cxx.h>
#include <kiwix/book.h>
#include <kiwix/library.h>
#include <kiwix/server.h>

std::shared_ptr<kiwix::Library> create_library();
std::shared_ptr<kiwix::Book> create_book();

void book_set_path(kiwix::Book& book, rust::Str path);
bool library_add_book(kiwix::Library& library, const kiwix::Book& book);

std::shared_ptr<kiwix::Server> create_server(std::shared_ptr<kiwix::Library> library);

void server_set_address(kiwix::Server& server, rust::Str address);
void server_set_port(kiwix::Server& server, int port);
void server_set_ip_mode(kiwix::Server& server, std::int32_t mode);
bool server_start(kiwix::Server& server);
void server_stop(kiwix::Server& server);
