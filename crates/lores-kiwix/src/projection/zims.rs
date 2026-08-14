use lores_kiwix_node::operations::ZimRegisteredDataV1;
use sqlx::{FromRow, SqlitePool};

/// A row from the `zims` projection table.
#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct ZimRow {
    pub id: String,
    pub filename: String,
    pub name: String,
    pub date: String,
    pub flavour: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub creator: String,
    pub publisher: String,
}

/// Return every ZIM recorded in the projection database.
pub async fn list_zims(pool: &SqlitePool) -> Result<Vec<ZimRow>, sqlx::Error> {
    sqlx::query_as::<_, ZimRow>("SELECT * FROM zims ORDER BY title")
        .fetch_all(pool)
        .await
}

pub async fn insert_zim(pool: &SqlitePool, data: &ZimRegisteredDataV1) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO zims (
            id,
            filename,
            name,
            date,
            flavour,
            title,
            description,
            language,
            creator,
            publisher
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&data.book_id)
    .bind(&data.filename)
    .bind(&data.name)
    .bind(&data.date)
    .bind(&data.flavour)
    .bind(&data.title)
    .bind(&data.description)
    .bind(&data.language)
    .bind(&data.creator)
    .bind(&data.publisher)
    .execute(pool)
    .await?;

    Ok(())
}
