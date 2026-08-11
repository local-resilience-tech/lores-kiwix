use std::env;

use libkiwix_rust::{self as kiwix, IpMode, ServerConfig};

fn usage(program: &str) {
    eprintln!("Usage: {} <zim-file-or-dir> [address:port]", program);
    eprintln!("  address:port defaults to 0.0.0.0:8080");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.first().map(|s| s.as_str()).unwrap_or("lores-kiwix");

    if args.len() < 2 {
        usage(program);
        std::process::exit(1);
    }

    let path = &args[1];
    let bind = args.get(2).map(|s| s.as_str()).unwrap_or("0.0.0.0:8080");
    let (address, port) = parse_bind(bind);

    let mut library = kiwix::new_library();

    add_path_to_library(&mut library, path);

    let mut server = kiwix::new_server(
        library,
        &ServerConfig {
            address,
            port,
            ip_mode: IpMode::Auto,
        },
    );

    eprintln!("Starting lores-kiwix on {}", bind);
    if !kiwix::server_start(&mut server) {
        eprintln!("Failed to start server");
        std::process::exit(1);
    }

    eprintln!("Server running. Press Ctrl+C to stop.");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn parse_bind(bind: &str) -> (String, i32) {
    match bind.rsplit_once(':') {
        Some((addr, port_str)) => {
            let port = port_str.parse::<i32>().unwrap_or(8080);
            (addr.to_string(), port)
        }
        None => (bind.to_string(), 8080),
    }
}

fn add_path_to_library(library: &mut kiwix::Library, path: &str) {
    let meta = std::fs::metadata(path).expect("cannot access path");

    if meta.is_file() {
        add_zim(library, path);
        return;
    }

    if meta.is_dir() {
        for entry in std::fs::read_dir(path).expect("cannot read directory") {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zim") {
                add_zim(library, path.to_str().unwrap());
            }
        }
        return;
    }

    panic!("path is neither a file nor a directory: {}", path);
}

fn add_zim(library: &mut kiwix::Library, path: &str) {
    let mut book = kiwix::new_book();
    kiwix::book_set_path(&mut book, path);
    kiwix::library_add_book(library, &book);
    eprintln!("Added: {}", path);
}
