use std::path::PathBuf;

use lores_dev_server::proto::panda_server::PandaServer;
use lores_dev_server::service::DevPandaService;
use lores_kiwix::{BootConfig, ProjectionDbConfig, boot};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

pub const SMALL_BOOK_ID: &str = "eeb924bb-0f5b-60f6-9d13-1259f6516ae7";
pub const SMALL_BOOK_TITLE: &str = "=Test ZIM file";
pub const APP_ID: &str = "lores-kiwix-test";
pub const INSTANCE_ID: &str = "test-instance";
pub const REMOTE_INSTANCE_ID: &str = "remote-instance";
pub const REMOTE_BOOK_ID: &str = "22222222-2222-2222-2222-222222222222";
pub const REMOTE_BOOK_TITLE: &str = "Remote Book";

pub fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

/// Start an in-memory lores-dev-server on an ephemeral port and return its
/// gRPC endpoint URL together with a handle that can be used to inspect
/// published operations.
pub async fn start_dev_server() -> (String, DevPandaService) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind dev server");
    let addr = listener.local_addr().expect("failed to get local address");
    let service = DevPandaService::with_operation_recording();
    let server_service = service.clone();

    tokio::spawn(async move {
        Server::builder()
            .add_service(PandaServer::new(server_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("dev server failed");
    });

    (format!("http://{addr}"), service)
}

pub fn temp_data_dir() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    (temp_dir, data_dir)
}

fn boot_config(app_id: &str, data_dir: &std::path::Path, path: String) -> BootConfig {
    BootConfig {
        path,
        panda_grpc_addr: data_dir
            .parent()
            .map(|p| p.join("grpc_addr_placeholder"))
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_default(),
        app_id: app_id.to_string(),
        instance_id: INSTANCE_ID.to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        internal_bind: "127.0.0.1:0".to_string(),
        projection_db: ProjectionDbConfig::OnDisk(data_dir.join("projection.sqlite").to_string_lossy().to_string()),
    }
}

pub async fn boot_with_fixture(
    grpc_addr: String,
    app_id: &str,
    data_dir: std::path::PathBuf,
    fixture: &str,
) -> lores_kiwix::BootResult {
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("failed to create data dir");

    let mut config = boot_config(app_id, &data_dir, fixture_path(fixture).to_string_lossy().to_string());
    config.panda_grpc_addr = grpc_addr;

    boot(&config).await.expect("failed to boot lores-kiwix")
}

/// Boot lores-kiwix with an empty fixtures directory (no ZIM files present).
pub async fn boot_with_empty_dir(
    grpc_addr: String,
    app_id: &str,
    data_dir: std::path::PathBuf,
) -> lores_kiwix::BootResult {
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("failed to create data dir");

    let empty_dir = data_dir.parent().unwrap().join("empty");
    tokio::fs::create_dir_all(&empty_dir)
        .await
        .expect("failed to create empty dir");

    let mut config = boot_config(app_id, &data_dir, empty_dir.to_string_lossy().to_string());
    config.panda_grpc_addr = grpc_addr;

    boot(&config).await.expect("failed to boot lores-kiwix")
}

/// Seed the projection database with the small book already held by the local node.
pub async fn seed_projection_with_local_holding(data_dir: &std::path::Path) {
    tokio::fs::create_dir_all(data_dir)
        .await
        .expect("failed to create data dir for seeding");

    let db_path = data_dir.join("projection.sqlite");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite://{}?mode=rwc", db_path.to_string_lossy()))
        .await
        .expect("failed to connect to projection database");

    let schema = include_str!("../../src/projection/schema.sql");
    let hash = Sha256::digest(schema.as_bytes());
    let hash = format!("{:x}", hash);

    sqlx::raw_sql("CREATE TABLE _schema (hash TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("failed to create _schema table");

    sqlx::query("INSERT INTO _schema (hash) VALUES (?)")
        .bind(&hash)
        .execute(&pool)
        .await
        .expect("failed to insert schema hash");

    sqlx::raw_sql(schema)
        .execute(&pool)
        .await
        .expect("failed to apply projection schema");

    sqlx::query(
        "INSERT INTO books (
            id, name, date, flavour, title, description, language,
            creator, publisher, category, tags, query_text
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(SMALL_BOOK_ID)
    .bind("")
    .bind("2020-11-15")
    .bind("")
    .bind(SMALL_BOOK_TITLE)
    .bind("=")
    .bind("=en")
    .bind("=")
    .bind("=")
    .bind("")
    .bind("_ftindex:yes;_ftindex:yes;_pictures:yes;_videos:yes;_details:yes")
    .bind("=test zim file = =en = = = _ftindex:yes;_ftindex:yes;_pictures:yes;_videos:yes;_details:yes")
    .execute(&pool)
    .await
    .expect("failed to insert book");

    // The node_id must match the hash of the instance_id used by the dev server
    // so that sync_filesystem sees the book as already held by this node.
    let local_node_id = hex::encode(Sha256::digest(INSTANCE_ID));
    sqlx::query("INSERT INTO nodes (id, local) VALUES (?, TRUE)")
        .bind(&local_node_id)
        .execute(&pool)
        .await
        .expect("failed to insert local node");

    sqlx::query("INSERT INTO holdings (book_id, node_id) VALUES (?, ?)")
        .bind(SMALL_BOOK_ID)
        .bind(&local_node_id)
        .execute(&pool)
        .await
        .expect("failed to insert holding");

    pool.close().await;
}

pub async fn wait_for_operations(dev_server: &DevPandaService, app_id: &str) {
    for _ in 0..50 {
        if !dev_server.operations_for_app(app_id).await.is_empty() {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

/// The dev-server-derived node id for the fake remote instance.
pub fn remote_node_id() -> String {
    hex::encode(Sha256::digest(REMOTE_INSTANCE_ID.as_bytes()))
}

/// Publish an operation to the dev server as if it came from a different
/// app instance (i.e. a remote node).
pub async fn publish_remote_operation(grpc_addr: &str, app_id: &str, instance_id: &str, payload: Vec<u8>) {
    use lores_dev_server::proto::PublishRequest;
    use lores_dev_server::proto::panda_client::PandaClient;

    let mut client = PandaClient::connect(grpc_addr.to_string())
        .await
        .expect("failed to connect to dev server");

    let request = PublishRequest {
        app_id: app_id.to_string(),
        instance_id: instance_id.to_string(),
        payload,
        idempotency_key: Vec::new(),
    };

    client
        .publish(request)
        .await
        .expect("failed to publish remote operation");
}

/// Poll the projection database until the given book appears.
pub async fn wait_for_projection_book(pool: &sqlx::SqlitePool, book_id: &str) {
    for _ in 0..50 {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
            .bind(book_id)
            .fetch_one(pool)
            .await
            .expect("failed to query books projection");
        if row.0 > 0 {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!("book {book_id} was not projected in time");
}
