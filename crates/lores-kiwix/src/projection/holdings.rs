use sqlx::{Executor, Sqlite};

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
