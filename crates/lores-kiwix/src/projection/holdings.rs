use std::collections::HashMap;

use sqlx::{Executor, Sqlite, SqlitePool};

/// Record that `node_id` holds `book_id`. Idempotent.
pub async fn insert_holding<'e, E>(executor: E, book_id: &str, node_id: &str) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("INSERT OR IGNORE INTO holdings (book_id, node_id) VALUES (?, ?)")
        .bind(book_id)
        .bind(node_id)
        .execute(executor)
        .await?;

    Ok(())
}

/// Return a map of book_id → node IDs for each of the given book IDs.
pub async fn fetch_holdings_for_books(
    pool: &SqlitePool,
    book_ids: &[String],
) -> Result<HashMap<String, Vec<String>>, sqlx::Error> {
    if book_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!("SELECT book_id, node_id FROM holdings WHERE book_id IN ({placeholders})");
    let mut query = sqlx::query_as::<_, (String, String)>(&sql);
    for id in book_ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(pool).await?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (book_id, node_id) in rows {
        map.entry(book_id).or_default().push(node_id);
    }
    Ok(map)
}

pub async fn locally_held_book_ids(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let result: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT(book_id)
        FROM holdings
        INNER JOIN nodes ON holdings.node_id = nodes.id
        WHERE nodes.local IS TRUE
    "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn delete_local_holding(pool: &SqlitePool, book_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM holdings
        WHERE book_id = ?
          AND node_id IN (SELECT id FROM nodes WHERE local IS TRUE)
    "#,
    )
    .bind(book_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove the holding of `book_id` by a specific `node_id`.
/// If the node is not known, falls back to deleting only local holdings so that
/// deregistrations from this node keep working in tests or single-node setups.
pub async fn delete_holding_for_node(pool: &SqlitePool, book_id: &str, node_id: &str) -> Result<(), sqlx::Error> {
    let known_node: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM nodes WHERE id = ?")
        .bind(node_id)
        .fetch_optional(pool)
        .await?
        .filter(|(count,): &(i64,)| *count > 0);

    if known_node.is_some() {
        sqlx::query("DELETE FROM holdings WHERE book_id = ? AND node_id = ?")
            .bind(book_id)
            .bind(node_id)
            .execute(pool)
            .await?;
    } else {
        // The node was never recorded; treat this as a local-only deregistration.
        delete_local_holding(pool, book_id).await?;
    }

    Ok(())
}
