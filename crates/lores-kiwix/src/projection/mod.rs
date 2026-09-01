pub mod books;
pub mod holdings;
pub mod nodes;

use lores_app_node::ProjectionDb;
use sqlx::SqlitePool;

const SCHEMA: &str = include_str!("schema.sql");

/// Create the in-memory projection database with the current schema applied.
pub async fn create_projection_db() -> Result<(SqlitePool, bool), sqlx::Error> {
    ProjectionDb::in_memory(SCHEMA).await
}

/// Open (or create) an on-disk projection database at `path` with the current
/// schema applied.
///
/// Returns a pool and a boolean indicating whether the schema was rebuilt and
/// operations should be replayed.
pub async fn open_projection_db(path: &str) -> Result<(SqlitePool, bool), sqlx::Error> {
    ProjectionDb::open(path, SCHEMA).await
}
