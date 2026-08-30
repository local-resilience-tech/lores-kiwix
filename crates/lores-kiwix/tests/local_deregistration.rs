use lores_kiwix::node::operations::{AppOperation, BookDeregisteredDataV1};

mod common;

use common::{
    APP_ID, SMALL_BOOK_ID, boot_with_empty_dir, seed_projection_with_local_holding, start_dev_server, temp_data_dir,
    wait_for_operations,
};

#[tokio::test]
async fn publishes_book_deregistered_when_previously_held_book_is_missing() {
    let (grpc_addr, dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();

    // Seed the projection database as if this node previously held the book.
    seed_projection_with_local_holding(&data_dir).await;

    // Boot with an empty directory: the previously held book is now missing,
    // so a deregistration operation should be published.
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;

    wait_for_operations(&dev_server, APP_ID).await;

    let operations = dev_server.operations_for_app(APP_ID).await;
    assert_eq!(operations.len(), 1, "expected exactly one deregistration operation");

    let event: AppOperation = serde_json::from_slice(&operations[0].payload).expect("payload is valid JSON");

    let AppOperation::BookDeregisteredV1(BookDeregisteredDataV1 { book_id }) = event else {
        panic!("expected BookDeregisteredV1, got {:?}", event);
    };
    assert_eq!(book_id, SMALL_BOOK_ID);

    // The local holding should be removed, but the book record remains.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(SMALL_BOOK_ID)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query books projection");
    assert_eq!(row.0, 1, "expected book row to remain");

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM holdings
         INNER JOIN nodes ON holdings.node_id = nodes.id
         WHERE book_id = ? AND nodes.local IS TRUE",
    )
    .bind(SMALL_BOOK_ID)
    .fetch_one(&result.projection_pool)
    .await
    .expect("failed to query holdings projection");
    assert_eq!(row.0, 0, "expected local holding row to be removed");
}
