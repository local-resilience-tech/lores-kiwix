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

use crate::api::{ApiState, categories, entries, holding_libraries, languages};
use crate::node::LoresKiwixNode;

mod append_text;
mod content;
mod static_override;

/// Build an Axum application that proxies every request to `upstream`,
/// except for routes that are specifically overwritten.
pub fn app(upstream: impl Into<String>, pool: SqlitePool, library: Arc<Mutex<LibraryHandle>>, node: LoresKiwixNode) -> Router {
    let state = ApiState::new(upstream, pool, library, node);

    Router::new()
        .route("/content/{book_id}", any(content::handler))
        .route("/catalog/v2/entries", any(entries::handler))
        .route("/catalog/v2/categories", any(categories::handler))
        .route("/catalog/v2/languages", any(languages::handler))
        .route(
            "/catalog/v2/entries/{book_id}/holding_libraries",
            any(holding_libraries::handler),
        )
        .route(
            "/skin/index.css",
            any(append_text::handler("/skin/index.css", "/css/index.css")),
        )
        .route(
            "/skin/index.js",
            any(static_override::handler(
                "/js/index.js",
                "application/javascript; charset=utf-8",
            )),
        )
        .route(
            "/skin/remote_content.css",
            any(static_override::handler("/css/remote_content.css", "text/css")),
        )
        .route(
            "/skin/remote_content.js",
            any(static_override::handler(
                "/js/remote_content.js",
                "application/javascript; charset=utf-8",
            )),
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
