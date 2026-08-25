use lores_app_node::AppNode;
use sqlx::SqlitePool;

use self::operations::AppOperation;

pub mod operations;

pub type LoresKiwixNode = AppNode<AppOperation>;

pub async fn connect(
    local_operations_pool: SqlitePool,
    grpc_addr: String,
    app_id: impl Into<String>,
    instance_id: impl Into<String>,
) -> Result<LoresKiwixNode, sqlx::Error> {
    AppNode::grpc_with_local(local_operations_pool, grpc_addr, app_id, instance_id).await
}
