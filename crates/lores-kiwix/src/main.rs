use std::env;
use std::sync::{Arc, Mutex};

use libkiwix_rust::{self as kiwix, IpMode, ServerConfig};

use crate::node::operations::AppOperation;

mod api;
mod events;
mod library;
mod node;
mod projection;
mod proxy;
mod utilities;
mod xml;

const PANDA_GRPC_ADDR_ENV: &str = "PANDA_GRPC_ADDR";
const PANDA_GRPC_ADDR_DEFAULT: &str = "http://127.0.0.1:50051";

const APP_ID_ENV: &str = "LORES_APP_ID";
const APP_ID_DEFAULT: &str = "lores-websites";

const INSTANCE_ID_ENV: &str = "LORES_INSTANCE_ID";
const INSTANCE_ID_DEFAULT: &str = "default";

const DATA_DIR_ENV: &str = "DATA_DIR";
const DATA_DIR_DEFAULT: &str = "./data";

const KIWIX_INTERNAL_BIND_ENV: &str = "KIWIX_INTERNAL_BIND";
const KIWIX_INTERNAL_BIND_DEFAULT: &str = "127.0.0.1:18080";

fn usage(program: &str) {
    eprintln!("Usage: {} <zim-file-or-dir> [address:port]", program);
    eprintln!("  address:port defaults to 0.0.0.0:8080");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();
    let program = args.first().map(|s| s.as_str()).unwrap_or("lores-kiwix");

    if args.len() < 2 {
        usage(program);
        std::process::exit(1);
    }

    let (node, mut ready_rx, _node_task, projection_pool) = start_node().await;

    let path = &args[1];

    let requested_internal_bind =
        std::env::var(KIWIX_INTERNAL_BIND_ENV).unwrap_or_else(|_| KIWIX_INTERNAL_BIND_DEFAULT.to_string());
    let internal_bind = find_available_bind(&requested_internal_bind, 10)
        .unwrap_or_else(|| panic!("could not find an available internal bind near {requested_internal_bind}"));
    let upstream = format!("http://{internal_bind}");

    let mut library = kiwix::new_library();
    let registered = library::add_path_to_library(&mut library, path);
    let shared_library = Arc::new(Mutex::new(kiwix::LibraryHandle::new(library.clone())));
    let server_ready = start_internal_kiwix_server(kiwix::LibraryHandle::new(library), &internal_bind);

    // Wait for the node to finish replay before publishing startup operations.
    let _ = ready_rx.changed().await;

    for zim in &registered {
        let op = AppOperation::BookRegisteredV1(utilities::books::registered_data_from_path_and_metadata(
            &zim.path,
            &zim.metadata,
        ));
        if let Err(e) = node.publish(&op).await {
            eprintln!("Failed to publish BookRegisteredV1 for {}: {}", zim.path, e);
        }
    }

    let public_bind = args
        .get(2)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());

    server_ready.recv().expect("internal kiwix server did not report ready");
    wait_for_upstream(&upstream).await;

    let app = proxy::app(&upstream, projection_pool, shared_library);
    let listener = tokio::net::TcpListener::bind(&public_bind)
        .await
        .expect("failed to bind public proxy port");

    println!("Starting lores-kiwix proxy on http://{}", public_bind);
    println!("Press Ctrl+C to stop.");

    axum::serve(listener, app).await.expect("proxy server failed");
}

async fn start_node() -> (
    node::LoresKiwixNode,
    tokio::sync::watch::Receiver<bool>,
    tokio::task::JoinHandle<()>,
    sqlx::SqlitePool,
) {
    let panda_grpc_addr = std::env::var(PANDA_GRPC_ADDR_ENV).unwrap_or_else(|_| PANDA_GRPC_ADDR_DEFAULT.to_string());
    let app_id = std::env::var(APP_ID_ENV).unwrap_or_else(|_| APP_ID_DEFAULT.to_string());
    let instance_id = std::env::var(INSTANCE_ID_ENV).unwrap_or_else(|_| INSTANCE_ID_DEFAULT.to_string());

    let data_dir = std::env::var(DATA_DIR_ENV).unwrap_or_else(|_| DATA_DIR_DEFAULT.to_string());

    let (projection_pool, should_replay) = projection::create_projection_db()
        .await
        .expect("failed to create projection database");

    let operations_db_path = format!("{data_dir}/operations.sqlite");
    let operations_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite://{operations_db_path}?mode=rwc"))
        .await
        .expect("failed to open operations database");

    let node = node::connect(operations_pool, panda_grpc_addr, &app_id, &instance_id)
        .await
        .expect("failed to connect node");

    let run_node = node.clone();

    events::register_event_handlers(&node, projection_pool.clone());

    let (ready_tx, ready_rx) = tokio::sync::watch::channel(false);

    let handle = tokio::spawn(async move {
        if should_replay {
            run_node.replay().await.expect("replay failed");
        }
        let _ = ready_tx.send(true);
        run_node.run().await;
    });

    (node, ready_rx, handle, projection_pool)
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

/// Find an available TCP bind address, starting from `requested` and scanning
/// the next `fallback_count` ports if the first one is already in use.
///
/// This is intentionally synchronous: we want to reserve the port before
/// handing it off to the internal libkiwix server.
fn find_available_bind(requested: &str, fallback_count: usize) -> Option<String> {
    let (addr, base_port) = parse_bind(requested);
    let socket_addr = |port: i32| format!("{addr}:{port}");

    for offset in 0..=fallback_count {
        let candidate = socket_addr(base_port + offset as i32);
        if std::net::TcpListener::bind(&candidate).is_ok() {
            return Some(candidate);
        }
    }
    None
}

/// Start libkiwix on an internal address in a background thread.
///
/// Takes ownership of the already populated `library`, starts the kiwix server,
/// and signals readiness through the returned channel.
fn start_internal_kiwix_server(library: kiwix::LibraryHandle, bind: &str) -> std::sync::mpsc::Receiver<()> {
    let (address, port) = parse_bind(bind);
    let config = ServerConfig {
        address,
        port,
        ip_mode: IpMode::Auto,
    };

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let library = library.into_inner();
        let mut server = kiwix::new_server(library, &config);
        eprintln!("Starting internal kiwix server on {}:{}", config.address, config.port);
        if !kiwix::server_start(&mut server) {
            eprintln!("Failed to start internal kiwix server");
            return;
        }
        let _ = ready_tx.send(());
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    });

    ready_rx
}

/// Wait until the upstream kiwix server is accepting HTTP connections.
async fn wait_for_upstream(upstream: &str) {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(10) {
        if reqwest::get(format!("{upstream}/"))
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    eprintln!("Warning: upstream kiwix server did not become ready in time");
}
