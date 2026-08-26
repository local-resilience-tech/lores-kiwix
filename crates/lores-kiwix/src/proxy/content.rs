use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::Response,
};
use libkiwix_rust::library_get_book_metadata;

use super::proxy_error;
use crate::{api::ApiState, proxy::static_override::serve_static_file};

pub async fn handler(State(state): State<ApiState>, Path(book_id): Path<String>, uri: Uri) -> Response {
    let is_local = {
        let mut library = state.library.lock().unwrap_or_else(|p| p.into_inner());
        library_get_book_metadata(&mut library, &book_id).is_some()
    };

    if is_local {
        proxy_content_upstream(&state, &book_id, "", &uri).await
    } else {
        serve_remote_content().await
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

async fn serve_remote_content() -> Response {
    serve_static_file("/html/remote_content.html", "text/html; charset=utf-8").await
}
