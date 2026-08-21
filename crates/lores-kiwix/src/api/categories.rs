use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};
use elementtree::Element;

use crate::api::ApiState;
use crate::proxy::proxy_error;
use crate::xml::atom::ATOM_NS;

pub async fn handler(State(state): State<ApiState>) -> Response {
    let categories = {
        let library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        libkiwix_rust::library_get_books_categories(&library)
    };

    match render_categories(&categories) {
        Ok(buf) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
            .body(Body::from(buf))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to serialize categories feed");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize categories feed")
        }
    }
}

fn render_categories(categories: &[String]) -> Result<Vec<u8>, elementtree::Error> {
    const OPDS_NS: &str = "https://specs.opds.io/opds-1.2";

    let mut feed = Element::new((ATOM_NS, "feed"));
    feed.register_namespace(OPDS_NS, Some("opds"));

    feed.append_new_child((ATOM_NS, "id"))
        .set_text("urn:uuid:lores-kiwix:categories");
    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "self")
        .set_attr("href", "/catalog/v2/categories")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");
    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "start")
        .set_attr("href", "/catalog/v2/root.xml")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");
    feed.append_new_child((ATOM_NS, "title")).set_text("List of categories");
    feed.append_new_child((ATOM_NS, "updated"))
        .set_text(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    for category in categories {
        let encoded = url_encode(category);
        let mut entry = Element::new_with_namespaces((ATOM_NS, "entry"), &feed);
        entry.append_new_child((ATOM_NS, "title")).set_text(category);
        entry
            .append_new_child((ATOM_NS, "link"))
            .set_attr("rel", "subsection")
            .set_attr("href", format!("/catalog/v2/entries?category={}", encoded))
            .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=acquisition");
        entry.append_new_child((ATOM_NS, "updated")).set_text(&now);
        entry
            .append_new_child((ATOM_NS, "id"))
            .set_text(format!("urn:uuid:lores-kiwix:categories:{}", encoded));
        entry
            .append_new_child((ATOM_NS, "content"))
            .set_attr("type", "text")
            .set_text(format!("All entries with category of '{}'.", category));
        feed.append_child(entry);
    }

    let mut buf = Vec::new();
    feed.to_writer_with_options(&mut buf, elementtree::WriteOptions::new().set_perform_indent(true))?;
    Ok(buf)
}

fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
