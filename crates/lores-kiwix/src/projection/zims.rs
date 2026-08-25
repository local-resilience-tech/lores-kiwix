use libkiwix_rust::BookMetadata;
use sqlx::{FromRow, SqlitePool};

use crate::node::operations::ZimRegisteredDataV1;

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

/// Return distinct non-empty categories recorded in the projection database.
pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT DISTINCT category FROM zims WHERE category != '' ORDER BY category")
        .fetch_all(pool)
        .await
}

/// Return language codes with book counts from the projection database.
///
/// The `language` column may be comma-separated (e.g. `"eng,fra"`), so each
/// code is counted individually.
pub async fn list_languages(pool: &SqlitePool) -> Result<Vec<(String, u32)>, sqlx::Error> {
    let rows: Vec<String> =
        sqlx::query_scalar("SELECT language FROM zims WHERE language != ''")
            .fetch_all(pool)
            .await?;

    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for row in rows {
        for code in row.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            *counts.entry(code.to_string()).or_insert(0) += 1;
        }
    }

    let mut result: Vec<(String, u32)> = counts.into_iter().collect();
    result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Criteria for filtering extra ZIMs from the projection database.
pub struct FilterCriteria<'a> {
    pub query: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub category: Option<&'a str>,
}

/// Return ZIMs matching `criteria`.
///
/// The text query is matched against the combined `query_text` column as a
/// substring. The language and category filters match any value in their
/// comma-separated strings against the corresponding columns.
///
/// The text match is a simplified stand-in for libkiwix's Xapian-backed search.
/// It is case-insensitive for ASCII and uses `LIKE` with an escape character so
/// literal `%`, `_` and `\` characters in the query are treated literally.
pub async fn list_zims_filtered(pool: &SqlitePool, criteria: FilterCriteria<'_>) -> Result<Vec<Zim>, sqlx::Error> {
    let query = criteria.query.map(|q| q.trim()).filter(|q| !q.is_empty());
    let langs: Vec<&str> = criteria
        .lang
        .map(|l| l.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let categories: Vec<&str> = criteria
        .category
        .map(|c| c.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if query.is_none() && langs.is_empty() && categories.is_empty() {
        return list_zims(pool).await;
    }

    let mut sql = format!("SELECT {SELECT_ZIM_COLUMNS} FROM zims WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(query) = query {
        let pattern = format!(
            "%{}%",
            query
                .to_lowercase()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        sql.push_str(" AND query_text LIKE ? ESCAPE '\\'");
        params.push(pattern);
    }

    if !langs.is_empty() {
        let conditions: Vec<String> = (0..langs.len()).map(|_| "language = ?".to_string()).collect();
        sql.push_str(&format!(" AND ({})", conditions.join(" OR ")));
        params.extend(langs.iter().map(|s| s.to_string()));
    }

    if !categories.is_empty() {
        let conditions: Vec<String> = (0..categories.len()).map(|_| "category = ?".to_string()).collect();
        sql.push_str(&format!(" AND ({})", conditions.join(" OR ")));
        params.extend(categories.iter().map(|s| s.to_string()));
    }

    sql.push_str(" ORDER BY title");

    let mut query_builder = sqlx::query_as::<_, Zim>(&sql);
    for param in &params {
        query_builder = query_builder.bind(param);
    }

    query_builder.fetch_all(pool).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::operations::ZimRegisteredDataV1;
    use sqlx::SqlitePool;

    fn test_data(
        id: &str,
        title: &str,
        description: &str,
        language: &str,
        category: &str,
    ) -> ZimRegisteredDataV1 {
        ZimRegisteredDataV1 {
            filename: format!("{id}.zim"),
            book_id: id.to_string(),
            name: id.to_string(),
            date: "2024-01-01".to_string(),
            flavour: "maxi".to_string(),
            title: title.to_string(),
            description: description.to_string(),
            language: language.to_string(),
            creator: "Creator".to_string(),
            publisher: "Publisher".to_string(),
            category: category.to_string(),
            tags: "tag".to_string(),
        }
    }

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE zims (
                id          TEXT PRIMARY KEY NOT NULL,
                filename    TEXT NOT NULL,
                name        TEXT NOT NULL,
                date        TEXT NOT NULL,
                flavour     TEXT NOT NULL,
                title       TEXT NOT NULL,
                description TEXT NOT NULL,
                language    TEXT NOT NULL,
                creator     TEXT NOT NULL,
                publisher   TEXT NOT NULL,
                category    TEXT NOT NULL,
                tags        TEXT NOT NULL,
                query_text  TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn list_zims_filtered_matches_query_substring() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cooking", "Recipes from France", "fra", "cooking"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: Some("golf"),
                lang: None,
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-1");
    }

    #[tokio::test]
    async fn list_zims_filtered_matches_language() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: None,
                lang: Some("fra"),
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-2");
    }

    #[tokio::test]
    async fn list_zims_filtered_matches_any_language_in_comma_list() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-3", "Kochen", "Deutsche Rezepte", "deu", "cooking"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: None,
                lang: Some("eng,deu"),
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|z| z.id.as_str()).collect();
        assert!(ids.contains(&"id-1"));
        assert!(ids.contains(&"id-3"));
    }

    #[tokio::test]
    async fn list_zims_filtered_combines_query_and_language() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Golf en France", "A book about golf", "fra", "sports"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: Some("golf"),
                lang: Some("fra"),
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-2");
    }

    #[tokio::test]
    async fn list_zims_filtered_returns_all_when_no_criteria() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: None,
                lang: None,
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn list_zims_filtered_is_case_insensitive() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: Some("GOLF"),
                lang: None,
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn list_zims_filtered_escapes_like_special_chars() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "100% Golf", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: Some("100%"),
                lang: None,
                category: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-1");
    }

    #[tokio::test]
    async fn list_zims_filtered_matches_category() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cuisine", "Recipes from France", "eng", "cooking"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: None,
                lang: None,
                category: Some("cooking"),
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-2");
    }

    #[tokio::test]
    async fn list_zims_filtered_matches_any_category_in_comma_list() {
        let pool = setup_pool().await;
        insert_zim(&pool, &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-2", "Cuisine", "Recipes from France", "eng", "cooking"))
            .await
            .unwrap();
        insert_zim(&pool, &test_data("id-3", "Physics", "Intro to physics", "eng", "science"))
            .await
            .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: None,
                lang: None,
                category: Some("sports,science"),
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|z| z.id.as_str()).collect();
        assert!(ids.contains(&"id-1"));
        assert!(ids.contains(&"id-3"));
    }

    #[tokio::test]
    async fn list_zims_filtered_combines_query_language_and_category() {
        let pool = setup_pool().await;
        insert_zim(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_zim(
            &pool,
            &test_data("id-2", "Golf en France", "A book about golf", "fra", "sports"),
        )
        .await
        .unwrap();
        insert_zim(
            &pool,
            &test_data("id-3", "Golf Cooking", "A book about golf", "fra", "cooking"),
        )
        .await
        .unwrap();

        let results = list_zims_filtered(
            &pool,
            FilterCriteria {
                query: Some("golf"),
                lang: Some("fra"),
                category: Some("sports"),
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-2");
    }
}
