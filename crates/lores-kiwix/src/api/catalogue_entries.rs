use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};
use libkiwix_rust::Filter;

use crate::projection::zims;
use crate::utilities::pagination::Paginator;
use crate::xml::atom::{ATOM_NS, build_entry, build_entry_from_metadata, build_feed_root};
use crate::{api::ApiState, proxy::proxy_error};

/// Serve the `/catalog/v2/entries` endpoint directly from the local libkiwix
/// library instead of proxying to the upstream kiwix server.
///
/// Results from libkiwix are filtered, ranked, and paginated. Any extra ZIMs
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
    let (total, start, page_metadata) = {
        // The lock guard must not be held across an await point. Keep all
        // libkiwix work inside this synchronous block.
        let mut library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let book_ids = libkiwix_rust::library_filter(&mut library, &filter);

        let page = params.paginator.page(&book_ids);

        let mut page_metadata = Vec::with_capacity(page.items.len());
        for id in page.items {
            if let Some(metadata) = libkiwix_rust::library_get_book_metadata(&mut library, id) {
                page_metadata.push(metadata);
            }
        }

        (page.total, page.start, page_metadata)
    };

    let extra_zims = zims::list_zims(&state.pool).await.unwrap_or_default();

    let mut feed = build_feed_root(raw_query, total, start, page_metadata.len());

    for metadata in &page_metadata {
        feed.append_child(build_entry_from_metadata(metadata, ATOM_NS, &feed));
    }

    for zim in &extra_zims {
        feed.append_child(build_entry(zim, ATOM_NS, &feed));
    }

    let mut buf = Vec::new();
    if let Err(err) = feed.to_writer(&mut buf) {
        tracing::error!(error = %err, "failed to serialize catalog feed");
        return proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize catalog feed");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
        .body(Body::from(buf))
        .unwrap()
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
                "count" => params.paginator = Paginator::new(params.paginator.start(), Some(value.parse().ok().unwrap_or(0))),
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
            filter = filter.query(query);
        }
        if let Some(lang) = &self.lang {
            filter = filter.lang(lang);
        }
        if let Some(category) = &self.category {
            filter = filter.category(category);
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
