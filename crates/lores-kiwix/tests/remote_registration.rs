use lores_kiwix::node::operations::{AppOperation, BookRegisteredDataV1};

mod common;

use common::{
    APP_ID, REMOTE_BOOK_ID, REMOTE_BOOK_TITLE, REMOTE_INSTANCE_ID, boot_with_empty_dir, publish_remote_operation,
    remote_node_id, start_dev_server, temp_data_dir, wait_for_projection_book,
};

#[tokio::test]
async fn projects_book_registered_by_remote_node() {
    let (grpc_addr, _dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr.clone(), APP_ID, data_dir).await;

    // Give the local node time to subscribe before the remote operation is published.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let operation = AppOperation::BookRegisteredV1(BookRegisteredDataV1 {
        book_id: REMOTE_BOOK_ID.to_string(),
        name: "remote-book".to_string(),
        date: "2024-01-01".to_string(),
        flavour: "".to_string(),
        title: REMOTE_BOOK_TITLE.to_string(),
        description: "A book registered by a remote node".to_string(),
        language: "eng".to_string(),
        creator: "Remote Creator".to_string(),
        publisher: "Remote Publisher".to_string(),
        category: "remote".to_string(),
        tags: "_ftindex:yes".to_string(),
    });
    let payload = serde_json::to_vec(&operation).expect("failed to serialize operation");

    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload).await;

    wait_for_projection_book(&result.projection_pool, REMOTE_BOOK_ID).await;

    // The remote book should be recorded in the projection.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(REMOTE_BOOK_ID)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query books projection");
    assert_eq!(row.0, 1, "expected remote book to be projected");

    // The remote node should be recorded as non-local.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nodes WHERE id = ? AND local IS FALSE")
        .bind(remote_node_id())
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query nodes projection");
    assert_eq!(row.0, 1, "expected remote node to be recorded as non-local");

    // The remote holding should be recorded.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
        .bind(REMOTE_BOOK_ID)
        .bind(remote_node_id())
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query holdings projection");
    assert_eq!(row.0, 1, "expected remote holding to be projected");
}
