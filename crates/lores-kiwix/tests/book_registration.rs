use std::path::PathBuf;

use lores_dev_server::proto::panda_server::PandaServer;
use lores_dev_server::service::DevPandaService;
use lores_kiwix::node::operations::{AppOperation, BookRegisteredDataV1};
use lores_kiwix::{BootConfig, boot};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

const SMALL_BOOK_ID: &str = "eeb924bb-0f5b-60f6-9d13-1259f6516ae7";
const SMALL_BOOK_TITLE: &str = "=Test ZIM file";

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

/// Start an in-memory lores-dev-server on an ephemeral port and return its
/// gRPC endpoint URL together with a handle that can be used to inspect
/// published operations.
async fn start_dev_server() -> (String, DevPandaService) {
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

fn temp_data_dir() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    (temp_dir, data_dir)
}

async fn boot_with_fixture(
    grpc_addr: String,
    app_id: &str,
    data_dir: std::path::PathBuf,
    fixture: &str,
) -> lores_kiwix::BootResult {
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("failed to create data dir");

    let config = BootConfig {
        path: fixture_path(fixture).to_string_lossy().to_string(),
        panda_grpc_addr: grpc_addr,
        app_id: app_id.to_string(),
        instance_id: "test-instance".to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        internal_bind: "127.0.0.1:0".to_string(),
        projection_db: lores_kiwix::ProjectionDbConfig::OnDisk(
            data_dir.join("projection.sqlite").to_string_lossy().to_string(),
        ),
    };

    boot(&config).await.expect("failed to boot lores-kiwix")
}

/// Seed the projection database with the small book already held by the local node.
async fn seed_projection_with_local_holding(data_dir: &std::path::Path) {
    tokio::fs::create_dir_all(data_dir)
        .await
        .expect("failed to create data dir for seeding");

    let db_path = data_dir.join("projection.sqlite");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite://{}?mode=rwc", db_path.to_string_lossy()))
        .await
        .expect("failed to connect to projection database");

    let schema = include_str!("../src/projection/schema.sql");
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
    let local_node_id = hex::encode(Sha256::digest("test-instance"));
    sqlx::query("INSERT INTO nodes (id, local) VALUES (?, TRUE)")
        .bind(&local_node_id)
        .execute(&pool)
        .await
        .expect("failed to insert local node");

    sqlx::query("INSERT INTO holdings (book_id, node_id) VALUES (?, ?)")
        .bind(SMALL_BOOK_ID)
        .bind(&local_node_id)
        .bind(SMALL_BOOK_ID)
        .execute(&pool)
        .await
        .expect("failed to insert holding");

    pool.close().await;
}

async fn wait_for_operations(dev_server: &DevPandaService, app_id: &str) {
    for _ in 0..50 {
        if !dev_server.operations_for_app(app_id).await.is_empty() {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn publishes_book_registered_for_single_new_zim_file() {
    let (grpc_addr, dev_server) = start_dev_server().await;
    let app_id = "lores-kiwix-test";

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_fixture(grpc_addr, app_id, data_dir, "small.zim").await;

    wait_for_operations(&dev_server, app_id).await;

    let operations = dev_server.operations_for_app(app_id).await;
    assert_eq!(operations.len(), 1, "expected exactly one published operation");

    let event: AppOperation = serde_json::from_slice(&operations[0].payload).expect("payload is valid JSON");

    let AppOperation::BookRegisteredV1(BookRegisteredDataV1 {
        book_id,
        name,
        date,
        flavour,
        title,
        description,
        language,
        creator,
        publisher,
        category,
        tags,
    }) = event
    else {
        panic!("expected BookRegisteredV1, got {:?}", event);
    };

    assert_eq!(book_id, SMALL_BOOK_ID);
    assert_eq!(name, "");
    assert_eq!(date, "2020-11-15");
    assert_eq!(flavour, "");
    assert_eq!(title, SMALL_BOOK_TITLE);
    assert_eq!(description, "=");
    assert_eq!(language, "=en");
    assert_eq!(creator, "=");
    assert_eq!(publisher, "=");
    assert_eq!(category, "");
    assert_eq!(tags, "_ftindex:yes;_ftindex:yes;_pictures:yes;_videos:yes;_details:yes");

    // The projection should also record the book.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(&book_id)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query projection");
    assert_eq!(row.0, 1, "expected book to be recorded in projection");
}

#[tokio::test]
async fn does_not_republish_already_held_book() {
    let (grpc_addr, dev_server) = start_dev_server().await;
    let app_id = "lores-kiwix-test";

    let (_temp_dir, data_dir) = temp_data_dir();

    // Seed the projection database as if this node already holds the book.
    seed_projection_with_local_holding(&data_dir).await;

    // Boot against the seeded projection: the book should not be re-registered.
    let result = boot_with_fixture(grpc_addr, app_id, data_dir, "small.zim").await;

    // Give any async publishes time to arrive at the dev server before asserting.
    sleep(Duration::from_millis(100)).await;

    let operations = dev_server.operations_for_app(app_id).await;
    assert_eq!(operations.len(), 0, "expected no operations for an already-held book");

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(SMALL_BOOK_ID)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query books projection");
    assert_eq!(row.0, 1, "expected book row count to stay at one");

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM holdings
         INNER JOIN nodes ON holdings.node_id = nodes.id
         WHERE book_id = ? AND nodes.local IS TRUE",
    )
    .bind(SMALL_BOOK_ID)
    .fetch_one(&result.projection_pool)
    .await
    .expect("failed to query holdings projection");
    assert_eq!(row.0, 1, "expected local holding row count to stay at one");
}
