use lores_kiwix::node::operations::{AppOperation, BookDeregisteredDataV1, BookRegisteredDataV1};

mod common;

use common::{
    APP_ID, REMOTE_BOOK_ID, REMOTE_BOOK_TITLE, REMOTE_INSTANCE_ID, REMOTE_INSTANCE_ID_2, boot_with_empty_dir,
    publish_remote_operation, remote_node_id, remote_node_id_2, start_dev_server, temp_data_dir,
    wait_for_projection_book,
};

#[tokio::test]
async fn removes_remote_holding_when_book_deregistered_by_remote_node() {
    let (grpc_addr, _dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr.clone(), APP_ID, data_dir).await;

    // Give the local node time to subscribe before the remote operation is published.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // First the remote node registers a book.
    let register_operation = AppOperation::BookRegisteredV1(BookRegisteredDataV1 {
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
    let payload = serde_json::to_vec(&register_operation).expect("failed to serialize operation");
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload).await;

    wait_for_projection_book(&result.projection_pool, REMOTE_BOOK_ID).await;

    // Then the remote node deregisters the book.
    let deregister_operation = AppOperation::BookDeregisteredV1(BookDeregisteredDataV1 {
        book_id: REMOTE_BOOK_ID.to_string(),
    });
    let payload = serde_json::to_vec(&deregister_operation).expect("failed to serialize operation");
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload).await;

    // Wait for the holding to be removed.
    for _ in 0..50 {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
            .bind(REMOTE_BOOK_ID)
            .bind(remote_node_id())
            .fetch_one(&result.projection_pool)
            .await
            .expect("failed to query holdings projection");
        if row.0 == 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // The remote holding should be removed.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
        .bind(REMOTE_BOOK_ID)
        .bind(remote_node_id())
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query holdings projection");
    assert_eq!(row.0, 0, "expected remote holding to be removed");

    // The book record should remain.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(REMOTE_BOOK_ID)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query books projection");
    assert_eq!(row.0, 1, "expected book row to remain");
}

#[tokio::test]
async fn keeps_book_and_remaining_remote_holding_when_one_remote_node_deregisters() {
    let (grpc_addr, _dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr.clone(), APP_ID, data_dir).await;

    // Give the local node time to subscribe before the remote operations are published.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let register_operation = AppOperation::BookRegisteredV1(BookRegisteredDataV1 {
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
    let payload = serde_json::to_vec(&register_operation).expect("failed to serialize operation");

    // Two different remote instances register the same book.
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload.clone()).await;
    wait_for_projection_book(&result.projection_pool, REMOTE_BOOK_ID).await;
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID_2, payload).await;

    // Wait for the second holding to be recorded.
    for _ in 0..50 {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ?")
            .bind(REMOTE_BOOK_ID)
            .fetch_one(&result.projection_pool)
            .await
            .expect("failed to query holdings projection");
        if row.0 == 2 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // One remote node deregisters the book.
    let deregister_operation = AppOperation::BookDeregisteredV1(BookDeregisteredDataV1 {
        book_id: REMOTE_BOOK_ID.to_string(),
    });
    let payload = serde_json::to_vec(&deregister_operation).expect("failed to serialize operation");
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload).await;

    // Wait for the first remote holding to be removed.
    for _ in 0..50 {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
            .bind(REMOTE_BOOK_ID)
            .bind(remote_node_id())
            .fetch_one(&result.projection_pool)
            .await
            .expect("failed to query holdings projection");
        if row.0 == 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // The deregistering node's holding should be removed.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
        .bind(REMOTE_BOOK_ID)
        .bind(remote_node_id())
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query holdings projection");
    assert_eq!(row.0, 0, "expected deregistering node's holding to be removed");

    // The other remote holding should remain.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holdings WHERE book_id = ? AND node_id = ?")
        .bind(REMOTE_BOOK_ID)
        .bind(remote_node_id_2())
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query holdings projection");
    assert_eq!(row.0, 1, "expected remaining remote holding to stay");

    // The book record should remain.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books WHERE id = ?")
        .bind(REMOTE_BOOK_ID)
        .fetch_one(&result.projection_pool)
        .await
        .expect("failed to query books projection");
    assert_eq!(row.0, 1, "expected book row to remain");
}
