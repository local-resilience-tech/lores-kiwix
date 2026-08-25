use std::sync::{Arc, Mutex};

use libkiwix_rust::LibraryHandle;
use reqwest::Client;
use sqlx::SqlitePool;

pub mod categories;
pub mod entries;
pub mod languages;

/// Shared state passed to API route handlers.
#[derive(Clone)]
pub struct ApiState {
    pub client: Client,
    pub no_redirect_client: Client,
    pub upstream: String,
    pub pool: SqlitePool,
    pub library: Arc<Mutex<LibraryHandle>>,
}

impl ApiState {
    pub fn new(upstream: impl Into<String>, pool: SqlitePool, library: Arc<Mutex<LibraryHandle>>) -> Self {
        Self {
            client: Client::new(),
            no_redirect_client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            upstream: upstream.into(),
            pool,
            library,
        }
    }
}

#[allow(dead_code)]
fn _assert_api_state_traits() {
    fn assert_send_sync_clone_static<T: Send + Sync + Clone + 'static>() {}
    assert_send_sync_clone_static::<ApiState>();
}
