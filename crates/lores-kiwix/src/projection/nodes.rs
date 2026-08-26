use lores_app_node::LoResNodeId;
use sqlx::{Executor, Sqlite, SqlitePool, prelude::FromRow};

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

/// Insert a node record if it doesn't exist yet; never overwrites the `local` flag.
pub async fn ensure_node<'e, E>(executor: E, node_id: &str) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("INSERT INTO nodes (id, local) VALUES (?, FALSE) ON CONFLICT (id) DO NOTHING")
        .bind(node_id)
        .execute(executor)
        .await?;
    Ok(())
}

#[derive(FromRow)]
pub struct NodeRow {
    pub id: String,
    pub local: bool,
}

pub async fn list_nodes_holding_book(pool: &SqlitePool, book_id: &str) -> Result<Vec<NodeRow>, sqlx::Error> {
    let results = sqlx::query_as::<_, NodeRow>(
        "
        SELECT nodes.id, nodes.local
        FROM nodes
        INNER JOIN holdings ON holdings.node_id = nodes.id
        WHERE
            book_id = ?
        ORDER BY nodes.id",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}
