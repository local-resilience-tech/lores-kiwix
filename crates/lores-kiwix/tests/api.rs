use lores_kiwix::node::operations::{AppOperation, BookRegisteredDataV1};

mod common;

use common::{
    APP_ID, REMOTE_BOOK_ID, REMOTE_BOOK_TITLE, REMOTE_INSTANCE_ID, SMALL_BOOK_ID, SMALL_BOOK_TITLE,
    boot_with_empty_dir, boot_with_fixture, publish_remote_operation, remote_node_id, start_api_server,
    start_dev_server, temp_data_dir, wait_for_projection_book,
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

#[tokio::test]
async fn catalog_entries_does_not_duplicate_local_books() {
    let (grpc_addr, _dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_fixture(grpc_addr.clone(), APP_ID, data_dir, "small.zim").await;

    // Give the local node time to subscribe before the remote operation is published.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Register the same book as if it also exists on a remote node.
    let operation = AppOperation::BookRegisteredV1(BookRegisteredDataV1 {
        book_id: SMALL_BOOK_ID.to_string(),
        name: "".to_string(),
        date: "2020-11-15".to_string(),
        flavour: "".to_string(),
        title: SMALL_BOOK_TITLE.to_string(),
        description: "=".to_string(),
        language: "=en".to_string(),
        creator: "=".to_string(),
        publisher: "=".to_string(),
        category: "".to_string(),
        tags: "_ftindex:yes;_ftindex:yes;_pictures:yes;_videos:yes;_details:yes".to_string(),
    });
    let payload = serde_json::to_vec(&operation).expect("failed to serialize operation");
    publish_remote_operation(&grpc_addr, APP_ID, REMOTE_INSTANCE_ID, payload).await;

    wait_for_projection_book(&result.projection_pool, SMALL_BOOK_ID).await;

    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/catalog/v2/entries"))
        .send()
        .await
        .expect("failed to request catalog entries");

    assert_eq!(response.status(), 200, "expected catalog entries to return OK");

    let body = response.text().await.expect("failed to read response body");

    // Count <entry> elements in the feed. The shared book should appear as exactly one entry.
    let entries = body.matches("<entry").count();
    assert_eq!(
        entries, 1,
        "expected exactly one catalog entry for the shared book, got {entries}"
    );
}

#[tokio::test]
async fn catalog_categories_merges_local_and_remote_categories() {
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
        .get(format!("{api_url}/catalog/v2/categories"))
        .send()
        .await
        .expect("failed to request categories");

    assert_eq!(response.status(), 200, "expected categories to return OK");

    let body = response.text().await.expect("failed to read response body");

    // The response should contain exactly one category entry for "remote".
    let entries = body.matches("<entry").count();
    assert_eq!(entries, 1, "expected exactly one category entry, got {entries}");
    assert!(
        body.contains("remote"),
        "expected categories to include 'remote', got: {body}"
    );
}

#[tokio::test]
async fn catalog_languages_merges_local_and_remote_languages() {
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
        .get(format!("{api_url}/catalog/v2/languages"))
        .send()
        .await
        .expect("failed to request languages");

    assert_eq!(response.status(), 200, "expected languages to return OK");

    let body = response.text().await.expect("failed to read response body");

    // The response should contain exactly one language entry for "eng".
    let entries = body.matches("<entry").count();
    assert_eq!(entries, 1, "expected exactly one language entry, got {entries}");
    assert!(body.contains("eng"), "expected languages to include 'eng', got: {body}");
}

#[tokio::test]
async fn holding_libraries_returns_remote_node_holding_the_book() {
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
        .get(format!(
            "{api_url}/catalog/v2/entries/{REMOTE_BOOK_ID}/holding_libraries"
        ))
        .send()
        .await
        .expect("failed to request holding libraries");

    assert_eq!(response.status(), 200, "expected holding libraries to return OK");

    let body = response.text().await.expect("failed to read response body");

    // The response should contain exactly one entry for the remote node.
    let entries = body.matches("<entry").count();
    assert_eq!(entries, 1, "expected exactly one holding library entry, got {entries}");
    assert!(
        body.contains(&remote_node_id()),
        "expected holding libraries to include remote node id, got: {body}"
    );
}

#[tokio::test]
async fn content_serves_local_book_via_proxy() {
    let (grpc_addr, _dev_server) = start_dev_server().await;

    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_fixture(grpc_addr.clone(), APP_ID, data_dir, "small.zim").await;

    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/content/{SMALL_BOOK_ID}"))
        .send()
        .await
        .expect("failed to request local content");

    assert_eq!(response.status(), 200, "expected local content to return OK");

    let body = response.text().await.expect("failed to read response body");
    assert!(
        !body.contains("remote_content"),
        "expected local content response, got remote placeholder: {body}"
    );
}

#[tokio::test]
async fn content_returns_remote_placeholder_for_remote_only_book() {
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
        .get(format!("{api_url}/content/{REMOTE_BOOK_ID}"))
        .send()
        .await
        .expect("failed to request remote content");

    assert_eq!(response.status(), 200, "expected remote content to return OK");

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("remote_content"),
        "expected remote content placeholder, got: {body}"
    );
}
