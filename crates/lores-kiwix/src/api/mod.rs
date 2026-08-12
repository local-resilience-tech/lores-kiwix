use reqwest::Client;
use sqlx::SqlitePool;

pub mod catalogue_entries;

/// Shared state passed to API route handlers.
#[derive(Clone)]
pub struct ApiState {
    pub client: Client,
    pub upstream: String,
    pub pool: SqlitePool,
}

impl ApiState {
    pub fn new(upstream: impl Into<String>, pool: SqlitePool) -> Self {
        Self {
            client: Client::new(),
            upstream: upstream.into(),
            pool,
        }
    }
}
