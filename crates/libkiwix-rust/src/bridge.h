#pragma once
#include <cstdint>
#include <memory>
#include <string>
#include <rust/cxx.h>
#include <kiwix/book.h>
#include <kiwix/library.h>
#include <kiwix/manager.h>
#include <kiwix/server.h>

std::shared_ptr<kiwix::Library> create_library();
std::shared_ptr<kiwix::Book> create_book();

void book_set_path(kiwix::Book& book, rust::Str path);
bool library_add_book(kiwix::Library& library, const kiwix::Book& book);

rust::String library_add_book_from_path(kiwix::Library& library, rust::Str path);
std::shared_ptr<kiwix::Book> library_get_book_by_id(kiwix::Library& library, rust::Str id);

rust::String book_get_id(const kiwix::Book& book);
rust::String book_get_name(const kiwix::Book& book);
rust::String book_get_date(const kiwix::Book& book);
rust::String book_get_flavour(const kiwix::Book& book);
rust::String book_get_title(const kiwix::Book& book);
rust::String book_get_description(const kiwix::Book& book);
rust::String book_get_language(const kiwix::Book& book);
rust::String book_get_creator(const kiwix::Book& book);
rust::String book_get_publisher(const kiwix::Book& book);

std::shared_ptr<kiwix::Server> create_server(std::shared_ptr<kiwix::Library> library);

void server_set_address(kiwix::Server& server, rust::Str address);
void server_set_port(kiwix::Server& server, int port);
void server_set_ip_mode(kiwix::Server& server, std::int32_t mode);
bool server_start(kiwix::Server& server);
void server_stop(kiwix::Server& server);
