use libkiwix_rust::BookMetadata;
use lores_kiwix_node::operations::ZimRegisteredDataV1;
use sqlx::{FromRow, SqlitePool};

/// A row from the `zims` projection table.
#[derive(Debug, Clone, FromRow, Default)]
#[allow(dead_code)]
pub struct Zim {
    pub id: String,
    pub filename: String,
    pub name: Option<String>,
    pub date: Option<String>,
    pub flavour: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
}

/// Return every ZIM recorded in the projection database.
pub async fn list_zims(pool: &SqlitePool) -> Result<Vec<Zim>, sqlx::Error> {
    sqlx::query_as::<_, Zim>("SELECT * FROM zims ORDER BY title")
        .fetch_all(pool)
        .await
}

impl Into<BookMetadata> for Zim {
    /// Convert this projection row into a libkiwix `BookMetadata` value.
    ///
    /// Fields that are not stored in the projection are left blank or zeroed,
    /// so no dummy data is invented.
    fn into(self) -> BookMetadata {
        BookMetadata {
            id: self.id,
            name: self.name.unwrap_or_default(),
            date: self.date.unwrap_or_default(),
            flavour: self.flavour.unwrap_or_default(),
            title: self.title.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            language: self.language.unwrap_or_default(),
            creator: self.creator.unwrap_or_default(),
            publisher: self.publisher.unwrap_or_default(),
            category: String::new(),
            tags: String::new(),
            url: String::new(),
            article_count: 0,
            media_count: 0,
            size: 0,
            path_valid: false,
            illustrations: Vec::new(),
        }
    }
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
