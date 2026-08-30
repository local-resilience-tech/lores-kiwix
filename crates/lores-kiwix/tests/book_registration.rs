use lores_kiwix::node::operations::{AppOperation, BookRegisteredDataV1};

mod common;

use common::{
    APP_ID, SMALL_BOOK_ID, SMALL_BOOK_TITLE, boot_with_fixture, seed_projection_with_local_holding, start_dev_server,
    temp_data_dir, wait_for_operations,
};

#[tokio::test]
async fn publishes_book_registered_for_single_new_zim_file() {
    let (grpc_addr, dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_fixture(grpc_addr, APP_ID, data_dir, "small.zim").await;

    wait_for_operations(&dev_server, APP_ID).await;

    let operations = dev_server.operations_for_app(APP_ID).await;
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

    let (_temp_dir, data_dir) = temp_data_dir();

    // Seed the projection database as if this node already holds the book.
    seed_projection_with_local_holding(&data_dir).await;

    // Boot against the seeded projection: the book should not be re-registered.
    let result = boot_with_fixture(grpc_addr, APP_ID, data_dir, "small.zim").await;

    // Give any async publishes time to arrive at the dev server before asserting.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let operations = dev_server.operations_for_app(APP_ID).await;
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
