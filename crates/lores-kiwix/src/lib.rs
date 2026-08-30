pub mod api;
pub mod events;
pub mod library;
pub mod node;
pub mod projection;
pub mod proxy;
pub mod utilities;
pub mod xml;

use std::sync::{Arc, Mutex};

use libkiwix_rust::{self as kiwix, IpMode, ServerConfig};
use sqlx::SqlitePool;

use crate::library::sync_filesystem;
use crate::projection::{create_projection_db, open_projection_db};

/// Where the projection database should live.
#[derive(Clone, Debug)]
pub enum ProjectionDbConfig {
    /// Use an in-memory database, rebuilt on every boot.
    InMemory,
    /// Use an on-disk database at the given path.
    OnDisk(String),
}

/// Configuration used to boot a `lores-kiwix` instance.
#[derive(Clone, Debug)]
pub struct BootConfig {
    /// Path to a ZIM file or a directory containing ZIM files.
    pub path: String,
    /// Address of the lores-node gRPC endpoint.
    pub panda_grpc_addr: String,
    /// Application identifier used for p2panda topics.
    pub app_id: String,
    /// Instance identifier used for p2panda topics.
    pub instance_id: String,
    /// Directory where the local operations SQLite database is stored.
    pub data_dir: String,
    /// Bind address for the internal libkiwix HTTP server.
    pub internal_bind: String,
    /// Where the projection database should live.
    pub projection_db: ProjectionDbConfig,
}

/// Result of a successful [`boot`] call.
pub struct BootResult {
    /// Connected application node.
    pub node: node::LoresKiwixNode,
    /// Projection database pool.
    pub projection_pool: SqlitePool,
    /// Loaded Kiwix library.
    pub library: kiwix::Library,
    /// Thread-safe handle to the library, suitable for the HTTP proxy.
    pub shared_library: Arc<Mutex<kiwix::LibraryHandle>>,
    /// Base URL of the internal libkiwix server.
    pub upstream: String,
}

/// Errors that can occur while booting.
#[derive(Debug)]
pub enum BootError {
    Database(sqlx::Error),
    Node(lores_app_node::ConnectError),
    Bind(String),
    Server(String),
}

impl std::fmt::Display for BootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootError::Database(err) => write!(f, "database error: {err}"),
            BootError::Node(err) => write!(f, "node error: {err}"),
            BootError::Bind(msg) => write!(f, "bind error: {msg}"),
            BootError::Server(msg) => write!(f, "server error: {msg}"),
        }
    }
}

impl std::error::Error for BootError {}

impl From<sqlx::Error> for BootError {
    fn from(err: sqlx::Error) -> Self {
        BootError::Database(err)
    }
}

impl From<lores_app_node::ConnectError> for BootError {
    fn from(err: lores_app_node::ConnectError) -> Self {
        BootError::Node(err)
    }
}

/// Boot a `lores-kiwix` instance without starting the public HTTP proxy.
///
/// This creates the projection and operations databases, connects to the
/// configured lores-node, starts the internal libkiwix server, waits for the
/// node to finish replay, and then synchronises the filesystem against the
/// library — publishing any registration or deregistration operations.
pub async fn boot(config: &BootConfig) -> Result<BootResult, BootError> {
    let (projection_pool, should_replay) = match &config.projection_db {
        ProjectionDbConfig::InMemory => create_projection_db().await?,
        ProjectionDbConfig::OnDisk(path) => open_projection_db(path).await?,
    };

    std::fs::create_dir_all(&config.data_dir).map_err(|err| BootError::Database(sqlx::Error::Io(err)))?;
    let operations_db_path = format!("{}/operations.sqlite", config.data_dir);
    let operations_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite://{}?mode=rwc", operations_db_path))
        .await?;

    let node = node::connect(
        operations_pool,
        config.panda_grpc_addr.clone(),
        &config.app_id,
        &config.instance_id,
    )
    .await?;

    let run_node = node.clone();
    events::register_event_handlers(&node, projection_pool.clone());

    let (ready_tx, mut ready_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if should_replay {
            if let Err(err) = run_node.replay().await {
                tracing::error!(error = %err, "replay failed");
            }
        }
        let _ = ready_tx.send(true);
        run_node.run().await;
    });

    // Wait for the node to finish replay before publishing startup operations.
    let _ = ready_rx.changed().await;

    let internal_bind = find_available_bind(&config.internal_bind, 10).ok_or_else(|| {
        BootError::Bind(format!(
            "could not find an available bind near {}",
            config.internal_bind
        ))
    })?;
    let upstream = format!("http://{internal_bind}");

    let mut library = kiwix::new_library();
    let shared_library = Arc::new(Mutex::new(kiwix::LibraryHandle::new(library.clone())));
    let server_ready = start_internal_kiwix_server(kiwix::LibraryHandle::new(library.clone()), &internal_bind);

    server_ready
        .recv()
        .map_err(|_| BootError::Server("internal kiwix server did not report ready".to_string()))?;

    wait_for_upstream(&upstream).await;

    sync_filesystem(&mut library, &projection_pool, &node, &config.path).await;

    Ok(BootResult {
        node,
        projection_pool,
        library,
        shared_library,
        upstream,
    })
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
/// handing it off to the internal libkiwix server. When `requested` uses port
/// `0`, the OS-assigned port is returned.
fn find_available_bind(requested: &str, fallback_count: usize) -> Option<String> {
    let (addr, base_port) = parse_bind(requested);

    for offset in 0..=fallback_count {
        let port = base_port + offset as i32;
        let candidate = format!("{addr}:{port}");
        match std::net::TcpListener::bind(&candidate) {
            Ok(listener) => {
                let local_addr = listener.local_addr().ok()?;
                return Some(format!("{}:{}", local_addr.ip(), local_addr.port()));
            }
            Err(_) => continue,
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
