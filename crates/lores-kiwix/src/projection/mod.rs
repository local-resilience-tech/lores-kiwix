pub mod books;
pub mod holdings;

use lores_app_node::ProjectionDb;
use sqlx::SqlitePool;

const SCHEMA: &str = include_str!("schema.sql");

/// Create the in-memory projection database with the current schema applied.
pub async fn create_projection_db() -> Result<(SqlitePool, bool), sqlx::Error> {
    ProjectionDb::in_memory(SCHEMA).await
}
