use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Body,
    http::{StatusCode, header},
    response::Response,
    routing::any,
};
use axum_reverse_proxy::ReverseProxy;
use libkiwix_rust::LibraryHandle;
use sqlx::SqlitePool;

use crate::api::{ApiState, categories, entries, languages};

mod append_text;
mod content;
mod static_override;

/// Build an Axum application that proxies every request to `upstream`,
/// except for routes that are specifically overwritten.
pub fn app(upstream: impl Into<String>, pool: SqlitePool, library: Arc<Mutex<LibraryHandle>>) -> Router {
    let state = ApiState::new(upstream, pool, library);

    Router::new()
        .route("/content/{book_id}", any(content::handler))
        .route("/catalog/v2/entries", any(entries::handler))
        .route("/catalog/v2/categories", any(categories::handler))
        .route("/catalog/v2/languages", any(languages::handler))
        .route(
            "/skin/index.css",
            any(append_text::handler(
                "/skin/index.css",
                concat!(env!("CARGO_MANIFEST_DIR"), "/static/css/index.css"),
            )),
        )
        .route(
            "/skin/index.js",
            any(static_override::handler(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static/js/index.js"
            ))),
        )
        .fallback_service(ReverseProxy::new("/", state.upstream.as_str()))
        .with_state(state)
}

/// Build a simple text error response for proxy failures.
pub fn proxy_error(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message.to_string()))
        .unwrap()
}
