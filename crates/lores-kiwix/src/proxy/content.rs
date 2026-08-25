use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, Uri, header},
    response::Response,
};
use libkiwix_rust::library_get_book_metadata;

use super::proxy_error;
use crate::api::ApiState;

pub async fn handler(State(state): State<ApiState>, Path(book_id): Path<String>, uri: Uri) -> Response {
    let is_local = {
        let mut library = state.library.lock().unwrap_or_else(|p| p.into_inner());
        library_get_book_metadata(&mut library, &book_id).is_some()
    };

    if is_local {
        proxy_content_upstream(&state, &book_id, "", &uri).await
    } else {
        serve_remote_viewer().await
    }
}

async fn proxy_content_upstream(state: &ApiState, book_id: &str, path: &str, uri: &Uri) -> Response {
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    let upstream_url = format!("{}/content/{}/{}{}", state.upstream, book_id, path, query);

    let upstream_resp = match state.no_redirect_client.get(&upstream_url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!(error = %err, url = %upstream_url, "failed to fetch upstream content");
            return proxy_error(StatusCode::BAD_GATEWAY, "failed to fetch upstream content");
        }
    };

    let status = upstream_resp.status();
    let headers = upstream_resp.headers().clone();

    let body = match upstream_resp.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(error = %err, "failed to read upstream content body");
            return proxy_error(StatusCode::BAD_GATEWAY, "failed to read upstream content body");
        }
    };

    let mut builder = Response::builder().status(status);
    for (name, value) in &headers {
        builder = builder.header(name, value);
    }
    builder.body(Body::from(body)).unwrap()
}

async fn serve_remote_viewer() -> Response {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/html/remote_viewer.html");
    match tokio::fs::read_to_string(path).await {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(contents))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to read remote_viewer.html");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to read remote_viewer.html")
        }
    }
}
