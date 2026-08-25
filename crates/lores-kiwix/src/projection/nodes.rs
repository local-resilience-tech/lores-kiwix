use lores_app_node::LoResNodeId;
use sqlx::SqlitePool;

pub async fn set_local_node(pool: &SqlitePool, node_id: &LoResNodeId) -> Result<(), sqlx::Error> {
    let node_id = node_id.to_hex();
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE nodes SET local = FALSE WHERE id != ? AND local = TRUE")
        .bind(&node_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO nodes (id, local) VALUES (?, TRUE) ON CONFLICT (id) DO UPDATE SET local = TRUE")
        .bind(&node_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}
