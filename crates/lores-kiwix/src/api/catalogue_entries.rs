use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};

use elementtree::Element;

use crate::projection::zims;
use crate::proxy::proxy_error;
use crate::xml::serialize::build_entry;
use crate::{api::ApiState, xml::serialize::ATOM_NS};

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
fn merge_catalog_entries(upstream: &[u8], extra_zims: &[zims::Zim]) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // merge_catalog_entries

    #[test]
    fn merge_catalog_entries_returns_upstream_value() {
        let body = r#"
            <?xml version="1.0" encoding="utf-8"?><feed xmlns="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/terms/" xmlns:opds="https://specs.opds.io/opds-1.2" xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">
              <id>ece013ff-1e2c-37de-cb71-602f98c44a52</id>

              <link href="/catalog/v2/entries?start=0&amp;count=20&amp;lang=eng" rel="self" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
              <link href="/catalog/v2/root.xml" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <link href="/catalog/v2/root.xml" rel="up" type="application/atom+xml;profile=opds-catalog;kind=navigation" />

              <title>Filtered Entries (start=0&amp;count=20&amp;lang=eng)</title>
              <updated>2026-08-14T15:52:17Z</updated>
              <totalResults>1</totalResults>
              <startIndex>0</startIndex>
              <itemsPerPage>1</itemsPerPage>
              <entry>
                <id>urn:uuid:a5d5ec52-1652-4375-b62d-5380a40a353d</id>
                <title>Golf by Wikipedia</title>
                <updated>2026-07-17T00:00:00Z</updated>
                <summary>A selection of Wikipedia articles on golf</summary>
                <language>eng</language>
                <name>wikipedia_en_golf</name>
                <flavour>nopic</flavour>
                <category>wikipedia</category>
                <tags>wikipedia;_category:wikipedia;_pictures:no;_videos:no;_details:yes;_ftindex:yes</tags>
                <articleCount>25938</articleCount>
                <mediaCount>136</mediaCount>
                <link href="/catalog/v2/illustration/a5d5ec52-1652-4375-b62d-5380a40a353d/?size=48" rel="http://opds-spec.org/image/thumbnail" type="image/png;width=48;height=48;scale=1" />
                <link href="/content/a5d5ec52-1652-4375-b62d-5380a40a353d" type="text/html" />
                <author>
                  <name>Wikipedia</name>
                </author>
                <publisher>
                  <name>openZIM</name>
                </publisher>
                <dc:issued>2026-07-17T00:00:00Z</dc:issued>
              </entry>
            </feed>
        "#;

        let results = merge_catalog_entries(body.as_bytes(), &[]);
        let result_string = str::from_utf8(&results).unwrap();

        assert_eq!(body, result_string);
    }
}
