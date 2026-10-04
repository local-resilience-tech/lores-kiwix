pub struct FilterCriteria<'a> {
    pub query: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub category: Option<&'a str>,
}

/// Returns `true` when any filter criterion is active.
pub fn has_filter_criteria(criteria: &FilterCriteria<'_>) -> bool {
    let query = criteria.query.map(|q| q.trim()).filter(|q| !q.is_empty());
    let langs = criteria
        .lang
        .map(|l| l.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).count())
        .unwrap_or(0);
    let categories = criteria
        .category
        .map(|c| c.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).count())
        .unwrap_or(0);

    query.is_some() || langs > 0 || categories > 0
}

/// Apply `criteria` to `query` by pushing conditions and bindings directly onto
/// the [`sqlx::QueryBuilder`].
///
/// This keeps user input out of the SQL string — all values are bound as
/// parameters. The caller is responsible for ensuring `query` already has a
/// suitable `WHERE` clause or starts this call with `.push(" WHERE 1=1 ")`.
pub fn push_filter_clauses<'a>(query: &mut sqlx::QueryBuilder<sqlx::Sqlite>, criteria: FilterCriteria<'a>) {
    let query_text = criteria.query.map(|q| q.trim()).filter(|q| !q.is_empty());
    let langs: Vec<&str> = criteria
        .lang
        .map(|l| l.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let categories: Vec<&str> = criteria
        .category
        .map(|c| c.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if let Some(query_text) = query_text {
        let pattern = format!(
            "%{}%",
            query_text
                .to_lowercase()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        query
            .push(" AND query_text LIKE ")
            .push_bind(pattern)
            .push(" ESCAPE '\\'");
    }

    if !langs.is_empty() {
        query.push(" AND (");
        for (i, lang) in langs.iter().enumerate() {
            if i > 0 {
                query.push(" OR ");
            }
            query.push("language = ").push_bind(*lang);
        }
        query.push(")");
    }

    if !categories.is_empty() {
        query.push(" AND (");
        for (i, category) in categories.iter().enumerate() {
            if i > 0 {
                query.push(" OR ");
            }
            query.push("category = ").push_bind(*category);
        }
        query.push(")");
    }
}
