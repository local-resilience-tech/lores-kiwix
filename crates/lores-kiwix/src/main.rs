use std::env;

use libkiwix_rust::{self as kiwix, IpMode, ServerConfig};

mod library;

const PANDA_GRPC_ADDR_ENV: &str = "PANDA_GRPC_ADDR";
const PANDA_GRPC_ADDR_DEFAULT: &str = "http://127.0.0.1:50051";

const APP_ID_ENV: &str = "LORES_APP_ID";
const APP_ID_DEFAULT: &str = "lores-websites";

const INSTANCE_ID_ENV: &str = "LORES_INSTANCE_ID";
const INSTANCE_ID_DEFAULT: &str = "default";

const DATA_DIR_ENV: &str = "DATA_DIR";
const DATA_DIR_DEFAULT: &str = "./data";

fn usage(program: &str) {
    eprintln!("Usage: {} <zim-file-or-dir> [address:port]", program);
    eprintln!("  address:port defaults to 0.0.0.0:8080");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.first().map(|s| s.as_str()).unwrap_or("lores-kiwix");

    if args.len() < 2 {
        usage(program);
        std::process::exit(1);
    }

    let panda_grpc_addr = std::env::var(PANDA_GRPC_ADDR_ENV).unwrap_or_else(|_| PANDA_GRPC_ADDR_DEFAULT.to_string());
    let app_id = std::env::var(APP_ID_ENV).unwrap_or_else(|_| APP_ID_DEFAULT.to_string());
    let instance_id = std::env::var(INSTANCE_ID_ENV).unwrap_or_else(|_| INSTANCE_ID_DEFAULT.to_string());

    let data_dir = std::env::var(DATA_DIR_ENV).unwrap_or_else(|_| DATA_DIR_DEFAULT.to_string());

    let (db, should_replay) = lores_kiwix_node::create_projection_db()
        .await
        .expect("failed to create projection database");

    let operations_db_path = format!("{data_dir}/operations.sqlite");
    let operations_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite://{operations_db_path}?mode=rwc"))
        .await
        .expect("failed to open operations database");

    let node = lores_kiwix_node::connect(operations_pool, panda_grpc_addr, &app_id, &instance_id)
        .await
        .expect("failed to connect node");

    let run_node = node.clone();

    let (ready_tx, ready_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if should_replay {
            run_node.replay().await.expect("replay failed");
        }
        let _ = ready_tx.send(true);
        run_node.run().await;
    });

    let path = &args[1];
    let bind = args.get(2).map(|s| s.as_str()).unwrap_or("0.0.0.0:8080");
    let (address, port) = parse_bind(bind);

    let mut library = kiwix::new_library();

    library::add_path_to_library(&mut library, path);

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
