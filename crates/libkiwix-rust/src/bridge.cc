#include "src/bridge.h"
#include "libkiwix-rust/src/lib.rs.h"

std::shared_ptr<kiwix::Library> create_library() {
    return kiwix::Library::create();
}
