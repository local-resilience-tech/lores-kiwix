pub struct FilterCriteria<'a> {
    pub query: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub category: Option<&'a str>,
}

/// Returns `None` when no criteria are set (caller should use `list_books`).
/// Otherwise returns a WHERE clause string (without the `WHERE` keyword) and its bound parameters.
pub fn build_filter_clauses(criteria: FilterCriteria<'_>) -> Option<(String, Vec<String>)> {
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
        return None;
    }

    let mut conditions: Vec<String> = Vec::new();
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
        conditions.push("query_text LIKE ? ESCAPE '\\'".to_string());
        params.push(pattern);
    }

    if !langs.is_empty() {
        let placeholders: Vec<String> = (0..langs.len()).map(|_| "language = ?".to_string()).collect();
        conditions.push(format!("({})", placeholders.join(" OR ")));
        params.extend(langs.iter().map(|s| s.to_string()));
    }

    if !categories.is_empty() {
        let placeholders: Vec<String> = (0..categories.len()).map(|_| "category = ?".to_string()).collect();
        conditions.push(format!("({})", placeholders.join(" OR ")));
        params.extend(categories.iter().map(|s| s.to_string()));
    }

    Some((conditions.join(" AND "), params))
}
