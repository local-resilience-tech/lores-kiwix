use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};
use elementtree::Element;
use libkiwix_rust::Filter;

use crate::projection::zims;
use crate::xml::serialize::{ATOM_NS, build_entry, build_entry_from_metadata};
use crate::{api::ApiState, proxy::proxy_error};

const DC_NS: &str = "http://purl.org/dc/terms/";
const OPDS_NS: &str = "https://specs.opds.io/opds-1.2";
const OPENSEARCH_NS: &str = "http://a9.com/-/spec/opensearch/1.1/";
const DEFAULT_COUNT: i64 = 10;

/// Serve the `/catalog/v2/entries` endpoint directly from the local libkiwix
/// library instead of proxying to the upstream kiwix server.
///
/// Results from libkiwix are filtered, ranked, and paginated. Any extra ZIMs
/// from the projection database are appended after the libkiwix results.
pub async fn handler(State(state): State<ApiState>, req: Request) -> Response {
    let raw_query = req.uri().query().unwrap_or("");
    let params = parse_params(raw_query);

    // Gather all synchronous libkiwix work before the first await so that the
    // `Element` tree (which uses `Rc` and is not `Send`) is constructed after
    // the last await point.
    let (total, start, page_metadata) = {
        // The lock guard must not be held across an await point. Keep all
        // libkiwix work inside this synchronous block.
        let filter = match build_filter(&params) {
            Some(filter) => filter,
            None => {
                return proxy_error(StatusCode::BAD_REQUEST, "failed to build catalog filter");
            }
        };
        let mut library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let book_ids = libkiwix_rust::library_filter(&mut library, &filter);

        let total = book_ids.len();
        let start = params.start.unwrap_or(0).min(total);
        let count = params.count.unwrap_or(DEFAULT_COUNT);
        let intended_count = if count < 0 {
            total.saturating_sub(start)
        } else {
            count as usize
        };
        let end = (start + intended_count).min(total);
        let page = &book_ids[start..end];

        let mut page_metadata = Vec::with_capacity(page.len());
        for id in page {
            if let Some(metadata) = libkiwix_rust::library_get_book_metadata(&mut library, id) {
                page_metadata.push(metadata);
            }
        }

        (total, start, page_metadata)
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
    start: Option<usize>,
    count: Option<i64>,
}

/// Parse the URL query string into a structured representation.
fn parse_params(query: &str) -> CatalogParams {
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
            "start" => params.start = value.parse().ok(),
            "count" => params.count = value.parse().ok(),
            _ => {}
        }
    }
    params
}

/// Build a libkiwix `Filter` from the parsed query parameters.
///
/// Returns `None` if an invalid `max_size` or `count` would cause the filter to
/// be unusable.
fn build_filter(params: &CatalogParams) -> Option<Filter> {
    let mut filter = Filter::new().valid(true).local(true);

    if let Some(query) = &params.query {
        filter = filter.query(query);
    }
    if let Some(lang) = &params.lang {
        filter = filter.lang(lang);
    }
    if let Some(category) = &params.category {
        filter = filter.category(category);
    }
    if let Some(name) = &params.name {
        filter = filter.name(name);
    }
    if let Some(tag) = &params.tag {
        let tags: Vec<String> = tag.split(';').map(|s| s.to_string()).collect();
        filter = filter.accept_tags(&tags);
    }
    if let Some(notag) = &params.notag {
        let tags: Vec<String> = notag.split(';').map(|s| s.to_string()).collect();
        filter = filter.reject_tags(&tags);
    }
    if let Some(max_size) = params.max_size {
        filter = filter.max_size(max_size);
    }

    Some(filter)
}

/// Build the root `<feed>` element for the catalog response.
fn build_feed_root(query: &str, total: usize, start: usize, items_per_page: usize) -> Element {
    let mut feed = Element::new((ATOM_NS, "feed"));
    feed.register_namespace(DC_NS, Some("dc"));
    feed.register_namespace(OPDS_NS, Some("opds"));
    feed.register_namespace(OPENSEARCH_NS, Some("opensearch"));

    let feed_id = if query.is_empty() {
        "urn:uuid:lores-kiwix:entries".to_string()
    } else {
        format!("urn:uuid:lores-kiwix:entries?{}", query)
    };
    feed.append_new_child((ATOM_NS, "id")).set_text(feed_id);

    let self_url = if query.is_empty() {
        "/catalog/v2/entries".to_string()
    } else {
        format!("/catalog/v2/entries?{}", query)
    };
    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "self")
        .set_attr("href", self_url)
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=acquisition");

    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "start")
        .set_attr("href", "/catalog/v2/root.xml")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");

    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "up")
        .set_attr("href", "/catalog/v2/root.xml")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");

    let title = if query.is_empty() {
        "All Entries".to_string()
    } else {
        format!("Filtered Entries ({})", query)
    };
    feed.append_new_child((ATOM_NS, "title")).set_text(title);

    feed.append_new_child((ATOM_NS, "updated")).set_text(rfc3339_now());

    feed.append_new_child((ATOM_NS, "totalResults"))
        .set_text(total.to_string());
    feed.append_new_child((ATOM_NS, "startIndex"))
        .set_text(start.to_string());
    feed.append_new_child((ATOM_NS, "itemsPerPage"))
        .set_text(items_per_page.to_string());

    feed
}

/// Return the current time as an RFC 3339 string.
fn rfc3339_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_params_extracts_known_fields() {
        let params = parse_params("q=golf&lang=eng&start=5&count=20&tag=foo;bar");
        assert_eq!(params.query, Some("golf".to_string()));
        assert_eq!(params.lang, Some("eng".to_string()));
        assert_eq!(params.start, Some(5));
        assert_eq!(params.count, Some(20));
        assert_eq!(params.tag, Some("foo;bar".to_string()));
    }

    #[test]
    fn build_filter_ignores_empty_params() {
        let params = CatalogParams::default();
        let _filter = build_filter(&params).unwrap();
    }
}
