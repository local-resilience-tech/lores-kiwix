use libkiwix_rust::BookMetadata;
use sqlx::{FromRow, SqlitePool};

use crate::node::operations::BookRegisteredDataV1;
use crate::utilities::filter::{FilterCriteria, build_filter_clauses};

#[derive(Debug, Clone, FromRow, Default)]
#[allow(dead_code)]
pub struct BookRow {
    pub id: String,
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

const SELECT_BOOK_COLUMNS: &str = "
    books.id,
    books.name,
    books.date,
    books.flavour,
    books.title,
    books.description,
    books.language,
    books.creator,
    books.publisher,
    books.category,
    books.tags
";

/// Return distinct non-empty categories recorded in the projection database.
pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT DISTINCT category FROM books WHERE category != '' ORDER BY category")
        .fetch_all(pool)
        .await
}

/// Return language codes with book counts from the projection database.
///
/// The `language` column may be comma-separated (e.g. `"eng,fra"`), so each
/// code is counted individually.
pub async fn list_languages(pool: &SqlitePool) -> Result<Vec<(String, u32)>, sqlx::Error> {
    let rows: Vec<String> = sqlx::query_scalar("SELECT language FROM books WHERE language != ''")
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

/// Return books matching `criteria`.
///
/// The text query is matched against the combined `query_text` column as a
/// substring. The language and category filters match any value in their
/// comma-separated strings against the corresponding columns.
///
/// The text match is a simplified stand-in for libkiwix's Xapian-backed search.
/// It is case-insensitive for ASCII and uses `LIKE` with an escape character so
/// literal `%`, `_` and `\` characters in the query are treated literally.
pub async fn list_remote_books_filtered(
    pool: &SqlitePool,
    criteria: FilterCriteria<'_>,
) -> Result<Vec<BookRow>, sqlx::Error> {
    let base = format!(
        "SELECT DISTINCT {SELECT_BOOK_COLUMNS} FROM books \
         JOIN holdings ON holdings.book_id = books.id \
         JOIN nodes ON nodes.id = holdings.node_id \
         WHERE nodes.local = FALSE"
    );

    let (sql, params) = match build_filter_clauses(criteria) {
        Some((where_clause, params)) => (format!("{base} AND {where_clause} ORDER BY title"), params),
        None => (format!("{base} ORDER BY title"), vec![]),
    };

    let mut query_builder = sqlx::query_as::<_, BookRow>(&sql);
    for param in &params {
        query_builder = query_builder.bind(param);
    }

    query_builder.fetch_all(pool).await
}

impl Into<BookMetadata> for BookRow {
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

pub async fn insert_book<'e, E>(executor: E, data: &BookRegisteredDataV1) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let query_text = build_query_text(data);

    sqlx::query(
        "INSERT OR REPLACE INTO books (
            id,
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
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&data.book_id)
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
    .execute(executor)
    .await?;

    Ok(())
}

/// Build a single lowercase string containing all text fields that libkiwix
/// indexes for full-text search.
fn build_query_text(data: &BookRegisteredDataV1) -> String {
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
    use crate::node::operations::BookRegisteredDataV1;
    use sqlx::SqlitePool;

    fn test_data(id: &str, title: &str, description: &str, language: &str, category: &str) -> BookRegisteredDataV1 {
        BookRegisteredDataV1 {
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
        sqlx::query(include_str!("schema.sql")).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO nodes (id, local) VALUES ('remote-node', FALSE)")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    async fn insert_remote_holding(pool: &SqlitePool, book_id: &str) {
        sqlx::query("INSERT OR IGNORE INTO holdings (book_id, node_id) VALUES (?, 'remote-node')")
            .bind(book_id)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_remote_books_filtered_matches_query_substring() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cooking", "Recipes from France", "fra", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_matches_language() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_matches_any_language_in_comma_list() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;
        insert_book(
            &pool,
            &test_data("id-3", "Kochen", "Deutsche Rezepte", "deu", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-3").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_combines_query_and_language() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Golf en France", "A book about golf", "fra", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_returns_all_when_no_criteria() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cuisine", "Recettes de France", "fra", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_is_case_insensitive() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_escapes_like_special_chars() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "100% Golf", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_matches_category() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cuisine", "Recipes from France", "eng", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_matches_any_category_in_comma_list() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Cuisine", "Recipes from France", "eng", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;
        insert_book(
            &pool,
            &test_data("id-3", "Physics", "Intro to physics", "eng", "science"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-3").await;

        let results = list_remote_books_filtered(
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
    async fn list_remote_books_filtered_combines_query_language_and_category() {
        let pool = setup_pool().await;
        insert_book(
            &pool,
            &test_data("id-1", "Golf Rules", "A book about golf", "eng", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-1").await;
        insert_book(
            &pool,
            &test_data("id-2", "Golf en France", "A book about golf", "fra", "sports"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-2").await;
        insert_book(
            &pool,
            &test_data("id-3", "Golf Cooking", "A book about golf", "fra", "cooking"),
        )
        .await
        .unwrap();
        insert_remote_holding(&pool, "id-3").await;

        let results = list_remote_books_filtered(
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
