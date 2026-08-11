#include "bridge.h"
#pragma diag_suppress 1696
#include "libkiwix-rust/src/lib.rs.h"
#pragma diag_default 1696

std::shared_ptr<kiwix::Library> create_library() {
    return kiwix::Library::create();
}
