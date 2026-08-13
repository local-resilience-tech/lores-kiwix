use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};

use elementtree::Element;

use crate::api::ApiState;
use crate::projection::zims;
use crate::proxy::proxy_error;

const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

/// Proxy the catalog request to the upstream kiwix server, then append any
/// extra ZIMs recorded in the projection database.
pub async fn handler(State(state): State<ApiState>, req: Request) -> Response {
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    let upstream_url = format!("{}/catalog/v2/entries{}", state.upstream, query);

    let upstream_resp = match state.client.get(&upstream_url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!(error = %err, url = %upstream_url, "failed to fetch upstream catalog");
            return proxy_error(StatusCode::BAD_GATEWAY, "failed to fetch upstream catalog");
        }
    };

    if !upstream_resp.status().is_success() {
        return proxy_error(
            StatusCode::from_u16(upstream_resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            "upstream catalog request failed",
        );
    }

    let body = match upstream_resp.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(error = %err, "failed to read upstream catalog body");
            return proxy_error(StatusCode::BAD_GATEWAY, "failed to read upstream catalog body");
        }
    };

    let extra_zims = zims::list_zims(&state.pool).await.unwrap_or_default();
    let merged = merge_catalog_entries(&body, &extra_zims);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
        .body(Body::from(merged))
        .unwrap()
}

/// Parse the upstream Atom/OPDS feed and inject `<entry>` elements for every
/// ZIM in `extra_zims` just before the closing `</feed>` tag.
fn merge_catalog_entries(upstream: &[u8], extra_zims: &[zims::ZimRow]) -> Vec<u8> {
    if extra_zims.is_empty() {
        return upstream.to_vec();
    }

    let mut feed = match Element::from_reader(upstream) {
        Ok(feed) => feed,
        Err(err) => {
            tracing::error!(error = %err, "failed to parse upstream catalog");
            return upstream.to_vec();
        }
    };

    let namespace = feed.tag().ns().unwrap_or(ATOM_NS).to_string();

    for zim in extra_zims {
        feed.append_child(build_entry(zim, &namespace, &feed));
    }

    let mut buf = Vec::new();
    if let Err(err) = feed.to_writer(&mut buf) {
        tracing::error!(error = %err, "failed to serialize merged catalog");
        return upstream.to_vec();
    }
    buf
}

fn build_entry(zim: &zims::ZimRow, namespace: &str, feed: &Element) -> Element {
    let mut entry = Element::new_with_namespaces((namespace, "entry"), feed);
    entry.append_new_child((namespace, "id")).set_text(&zim.id);
    entry.append_new_child((namespace, "title")).set_text(&zim.title);

    if !zim.date.is_empty() {
        entry.append_new_child((namespace, "updated")).set_text(&zim.date);
    }

    if !zim.creator.is_empty() {
        entry
            .append_new_child((namespace, "author"))
            .append_new_child((namespace, "name"))
            .set_text(&zim.creator);
    }

    if !zim.description.is_empty() {
        entry
            .append_new_child((namespace, "content"))
            .set_attr("type", "text/plain")
            .set_text(&zim.description);
    }

    // Acquisition link: points back through the proxy so the kiwix frontend
    // treats it like a local book. Remote content routing is still TODO.
    entry
        .append_new_child((namespace, "link"))
        .set_attr("rel", "http://opds-spec.org/acquisition/open-access")
        .set_attr("href", format!("/content/{}", zim.id))
        .set_attr("type", "text/html");

    entry
}
