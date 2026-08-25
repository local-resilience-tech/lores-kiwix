use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};
use libkiwix_rust::Filter;

use crate::utilities::pagination::Paginator;
use crate::xml::entries::{build_entry, build_feed_root};
use crate::xml::render_xml;
use crate::{api::ApiState, proxy::proxy_error};
use crate::{projection::books, utilities::book::LoResBook};

/// Serve the `/catalog/v2/entries` endpoint directly from the local libkiwix
/// library instead of proxying to the upstream kiwix server.
///
/// Results from libkiwix are filtered, ranked, and paginated. Any extra books
/// from the projection database are appended after the libkiwix results.
pub async fn handler(State(state): State<ApiState>, req: Request) -> Response {
    let raw_query = req.uri().query().unwrap_or("");
    let params = CatalogParams::parse(raw_query);

    let filter = match params.build_filter() {
        Some(filter) => filter,
        None => {
            return proxy_error(StatusCode::BAD_REQUEST, "failed to build catalog filter");
        }
    };

    // Gather all synchronous libkiwix work before the first await so that the
    // `Element` tree (which uses `Rc` and is not `Send`) is constructed after
    // the last await point.
    let (page_books, book_ids, lib_exhausted) = {
        // The lock guard must not be held across an await point. Keep all
        // libkiwix work inside this synchronous block.
        let mut library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let book_ids = libkiwix_rust::library_filter(&mut library, &filter);

        let page = params.paginator.page(&book_ids);
        let lib_exhausted = page.is_exhausted();
        let page_books = fetch_page_books(&mut library, page.items);

        (page_books, book_ids, lib_exhausted)
    };

    let extra_remote_books = if lib_exhausted {
        let remote_books = books::list_books_filtered(
            &state.pool,
            books::FilterCriteria {
                query: filter.query().as_deref(),
                lang: filter.lang().as_deref(),
                category: filter.category().as_deref(),
            },
        )
        .await
        .unwrap_or_default();
        remove_duplicate_books(&remote_books, &book_ids)
    } else {
        vec![]
    };

    let extra_page = params.paginator.tail(book_ids.len()).page(&extra_remote_books);

    let result = CatalogueEntriesResult {
        books: page_books
            .iter()
            .chain(&extra_page.items.iter().cloned().map(Into::into).collect::<Vec<_>>())
            .cloned()
            .collect(),
        total: book_ids.len() + extra_remote_books.len(),
        start: params.paginator.start_index(book_ids.len() + extra_remote_books.len()),
        items_per_page: page_books.len() + extra_page.items.len(),
        query: raw_query,
    };

    match render_result(result) {
        Ok(buf) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
            .body(Body::from(buf))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to serialize catalog feed");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize catalog feed")
        }
    }
}

/// Fetch metadata for each book ID in `page` while holding the library lock.
fn fetch_page_books(library: &mut libkiwix_rust::Library, page: &[String]) -> Vec<LoResBook> {
    let mut page_metadata: Vec<LoResBook> = Vec::with_capacity(page.len());
    for id in page {
        if let Some(metadata) = libkiwix_rust::library_get_book_metadata(library, id) {
            page_metadata.push(metadata.into());
        }
    }
    page_metadata
}

/// Return the extra books whose IDs do not already appear in the libkiwix results.
fn remove_duplicate_books(books: &[books::Book], book_ids: &[String]) -> Vec<books::Book> {
    let ids: std::collections::HashSet<&str> = book_ids.iter().map(|id| id.as_str()).collect();
    books
        .iter()
        .filter(|book| !ids.contains(book.id.as_str()))
        .cloned()
        .collect()
}

/// Result of collecting and paginating entries for the catalog response.
struct CatalogueEntriesResult<'a> {
    books: Vec<LoResBook>,
    total: usize,
    start: usize,
    items_per_page: usize,
    query: &'a str,
}

/// Render a `CatalogueEntriesResult` into an Atom feed byte buffer.
fn render_result(result: CatalogueEntriesResult<'_>) -> Result<Vec<u8>, elementtree::Error> {
    let now = chrono::Utc::now();
    let mut feed = build_feed_root(now, result.query, result.total, result.start, result.items_per_page);

    for book in &result.books {
        feed.append_child(build_entry(book, &feed));
    }

    render_xml(&feed)
}

/// Parsed query parameters for the entries endpoint.
#[derive(Debug, Default)]
struct CatalogParams {
    query: Option<String>,
    lang: Option<String>,
    category: Option<String>,
    name: Option<String>,
    tag: Option<String>,
    notag: Option<String>,
    max_size: Option<usize>,
    paginator: Paginator,
}

impl CatalogParams {
    /// Parse the URL query string into a structured representation.
    pub fn parse(query: &str) -> Self {
        let mut params = CatalogParams::default();
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                "q" => params.query = Some(value.into_owned()),
                "lang" => params.lang = Some(value.into_owned()),
                "category" => params.category = Some(value.into_owned()),
                "name" => params.name = Some(value.into_owned()),
                "tag" => params.tag = Some(value.into_owned()),
                "notag" => params.notag = Some(value.into_owned()),
                "maxsize" => params.max_size = value.parse().ok(),
                "start" => params.paginator = Paginator::new(value.parse().ok(), params.paginator.count()),
                "count" => params.paginator = Paginator::new(params.paginator.start(), value.parse().ok()),
                _ => {}
            }
        }
        params
    }

    /// Build a libkiwix `Filter` from the parsed query parameters.
    ///
    /// Returns `None` if an invalid `max_size` or `count` would cause the filter to
    /// be unusable.
    pub fn build_filter(&self) -> Option<Filter> {
        let mut filter = Filter::new().valid(true).local(true);

        if let Some(query) = &self.query {
            filter = filter.with_query(query);
        }
        if let Some(lang) = &self.lang {
            filter = filter.with_lang(lang);
        }
        if let Some(category) = &self.category {
            filter = filter.with_category(category);
        }
        if let Some(name) = &self.name {
            filter = filter.name(name);
        }
        if let Some(tag) = &self.tag {
            let tags: Vec<String> = tag.split(';').map(|s| s.to_string()).collect();
            filter = filter.accept_tags(&tags);
        }
        if let Some(notag) = &self.notag {
            let tags: Vec<String> = notag.split(';').map(|s| s.to_string()).collect();
            filter = filter.reject_tags(&tags);
        }
        if let Some(max_size) = self.max_size {
            filter = filter.max_size(max_size);
        }

        Some(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_params_extracts_known_fields() {
        let params = CatalogParams::parse("q=golf&lang=eng&start=5&count=20&tag=foo;bar");
        assert_eq!(params.query, Some("golf".to_string()));
        assert_eq!(params.lang, Some("eng".to_string()));
        assert_eq!(params.paginator.start(), Some(5));
        assert_eq!(params.paginator.count(), Some(20));
        assert_eq!(params.tag, Some("foo;bar".to_string()));
    }

    #[test]
    fn build_filter_ignores_empty_params() {
        let params = CatalogParams::default();
        let _filter = params.build_filter().unwrap();
    }
}
