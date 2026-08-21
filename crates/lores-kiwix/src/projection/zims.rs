use libkiwix_rust::BookMetadata;
use lores_kiwix_node::operations::ZimRegisteredDataV1;
use sqlx::{FromRow, SqlitePool};

/// A row from the `zims` projection table.
#[derive(Debug, Clone, FromRow, Default)]
#[allow(dead_code)]
pub struct Zim {
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
    pub category: String,
    pub tags: String,
}

const SELECT_ZIM_COLUMNS: &str = "
    id,
    filename,
    name,
    date,
    flavour,
    title,
    description,
    language,
    creator,
    publisher,
    category,
    tags
";

/// Return every ZIM recorded in the projection database.
pub async fn list_zims(pool: &SqlitePool) -> Result<Vec<Zim>, sqlx::Error> {
    sqlx::query_as::<_, Zim>(&format!("SELECT {SELECT_ZIM_COLUMNS} FROM zims ORDER BY title"))
        .fetch_all(pool)
        .await
}

/// Return ZIMs whose combined `query_text` contains `query` as a substring.
///
/// This is a simplified stand-in for libkiwix's Xapian-backed text search.
/// The match is case-insensitive for ASCII and uses `LIKE` with an escape
/// character so literal `%`, `_` and `\` characters in `query` are treated
/// literally.
pub async fn list_zims_filtered(pool: &SqlitePool, query: Option<&str>) -> Result<Vec<Zim>, sqlx::Error> {
    let Some(query) = query.map(|q| q.trim()).filter(|q| !q.is_empty()) else {
        return list_zims(pool).await;
    };

    let query = query.to_lowercase();
    let pattern = format!(
        "%{}%",
        query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
    );

    sqlx::query_as::<_, Zim>(&format!(
        "SELECT {SELECT_ZIM_COLUMNS} FROM zims
         WHERE query_text LIKE ? ESCAPE '\\'
         ORDER BY title"
    ))
    .bind(&pattern)
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
            name: self.name,
            date: self.date,
            flavour: self.flavour,
            title: self.title,
            description: self.description,
            language: self.language,
            creator: self.creator,
            publisher: self.publisher,
            category: self.category,
            tags: self.tags,
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
    let query_text = build_query_text(data);

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
            publisher,
            category,
            tags,
            query_text
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
    .bind(&data.category)
    .bind(&data.tags)
    .bind(&query_text)
    .execute(pool)
    .await?;

    Ok(())
}

/// Build a single lowercase string containing all text fields that libkiwix
/// indexes for full-text search.
fn build_query_text(data: &ZimRegisteredDataV1) -> String {
    [
        &data.title,
        &data.description,
        &data.name,
        &data.flavour,
        &data.language,
        &data.creator,
        &data.publisher,
        &data.category,
        &data.tags,
    ]
    .iter()
    .map(|s| s.as_str())
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase()
}
