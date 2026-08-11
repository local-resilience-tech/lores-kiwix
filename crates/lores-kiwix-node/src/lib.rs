use lores_app_node::{AppNode, ProjectionDb};
use sqlx::SqlitePool;

use crate::operations::AppOperation;

pub mod operations;

pub type LoresWebsiteNode = AppNode<AppOperation>;

const SCHEMA: &str = include_str!("schema.sql");

pub async fn connect(
    local_operations_pool: SqlitePool,
    grpc_addr: String,
    app_id: impl Into<String>,
    instance_id: impl Into<String>,
) -> Result<LoresWebsiteNode, sqlx::Error> {
    AppNode::grpc_with_local(local_operations_pool, grpc_addr, app_id, instance_id).await
}

/// Create the in-memory projection database with the current schema applied.

pub async fn create_projection_db() -> Result<(SqlitePool, bool), sqlx::Error> {
    ProjectionDb::in_memory(SCHEMA).await
}
