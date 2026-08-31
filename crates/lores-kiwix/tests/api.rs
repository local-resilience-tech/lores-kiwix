use lores_kiwix::node::operations::{AppOperation, BookRegisteredDataV1};

mod common;

use common::{
    APP_ID, REMOTE_BOOK_ID, REMOTE_BOOK_TITLE, REMOTE_INSTANCE_ID, boot_with_empty_dir, publish_remote_operation,
    start_api_server, start_dev_server, temp_data_dir, wait_for_projection_book,
};

#[tokio::test]
async fn catalog_entries_includes_remote_books() {
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

    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/catalog/v2/entries"))
        .send()
        .await
        .expect("failed to request catalog entries");

    assert_eq!(response.status(), 200, "expected catalog entries to return OK");

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains(REMOTE_BOOK_ID),
        "expected catalog entries to include the remote book id, got: {body}"
    );
}
