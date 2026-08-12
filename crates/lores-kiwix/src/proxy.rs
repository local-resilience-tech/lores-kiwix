use axum::{Router, routing::any};
use axum_reverse_proxy::ReverseProxy;
use sqlx::SqlitePool;

use crate::api::{ApiState, catalogue_entries};

/// Build an Axum application that proxies every request to `upstream`,
/// except for `/catalog/v2/entries` which is handled separately so we can
/// merge in entries from the projection database.
pub fn app(upstream: impl Into<String>, pool: SqlitePool) -> Router {
    let state = ApiState::new(upstream, pool);

    Router::new()
        .route("/catalog/v2/entries", any(catalogue_entries::handler))
        .fallback_service(ReverseProxy::new("/", state.upstream.as_str()))
        .with_state(state)
}
