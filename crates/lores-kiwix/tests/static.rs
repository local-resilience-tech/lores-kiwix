mod common;

use common::{APP_ID, boot_with_empty_dir, start_api_server, start_dev_server, temp_data_dir};

#[tokio::test]
async fn serves_static_override_for_skin_index_css() {
    let (grpc_addr, _dev_server) = start_dev_server().await;
    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;
    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/skin/index.css"))
        .send()
        .await
        .expect("failed to request /skin/index.css");

    assert_eq!(response.status(), 200, "expected static override to return OK");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()
        .expect("content-type is not valid UTF-8");
    assert_eq!(content_type, "text/css", "expected text/css content type");

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("div.book__wrapper.source-remote"),
        "expected static override body, got: {body}"
    );
}

#[tokio::test]
async fn serves_static_override_for_skin_index_js() {
    let (grpc_addr, _dev_server) = start_dev_server().await;
    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;
    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/skin/index.js"))
        .send()
        .await
        .expect("failed to request /skin/index.js");

    assert_eq!(response.status(), 200, "expected static override to return OK");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()
        .expect("content-type is not valid UTF-8");
    assert_eq!(
        content_type, "application/javascript; charset=utf-8",
        "expected javascript content type"
    );

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("class FragmentParams extends URLSearchParams"),
        "expected static override body, got: {body}"
    );
}

#[tokio::test]
async fn serves_static_override_for_remote_content_css() {
    let (grpc_addr, _dev_server) = start_dev_server().await;
    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;
    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/skin/remote_content.css"))
        .send()
        .await
        .expect("failed to request /skin/remote_content.css");

    assert_eq!(response.status(), 200, "expected static override to return OK");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()
        .expect("content-type is not valid UTF-8");
    assert_eq!(content_type, "text/css", "expected text/css content type");

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("body.remote-content"),
        "expected static override body, got: {body}"
    );
}

#[tokio::test]
async fn serves_static_override_for_remote_content_js() {
    let (grpc_addr, _dev_server) = start_dev_server().await;
    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;
    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/skin/remote_content.js"))
        .send()
        .await
        .expect("failed to request /skin/remote_content.js");

    assert_eq!(response.status(), 200, "expected static override to return OK");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()
        .expect("content-type is not valid UTF-8");
    assert_eq!(
        content_type, "application/javascript; charset=utf-8",
        "expected javascript content type"
    );

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("async function loadTranslations()"),
        "expected static override body, got: {body}"
    );
}

#[tokio::test]
async fn serves_static_override_for_remote_content_i18n_json() {
    let (grpc_addr, _dev_server) = start_dev_server().await;
    let (_temp_dir, data_dir) = temp_data_dir();
    let result = boot_with_empty_dir(grpc_addr, APP_ID, data_dir).await;
    let api_url = start_api_server(result).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{api_url}/skin/remote_content.i18n.json"))
        .send()
        .await
        .expect("failed to request /skin/remote_content.i18n.json");

    assert_eq!(response.status(), 200, "expected static override to return OK");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()
        .expect("content-type is not valid UTF-8");
    assert_eq!(
        content_type, "application/json; charset=utf-8",
        "expected application/json content type"
    );

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("not-at-this-location"),
        "expected static override body, got: {body}"
    );
}
