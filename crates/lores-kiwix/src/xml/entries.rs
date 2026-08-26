use chrono::{DateTime, Utc};
use elementtree::Element;

use crate::utilities::book::{LoResBook, LoResBookSource};

use super::{ATOM_NS, DC_NS, OPDS_NS, OPENSEARCH_NS};

/// Build an Atom `<entry>` element from a libkiwix `BookMetadata`.
///
/// This mirrors the libkiwix `catalog_v2_entry.xml` mustache template as
/// closely as possible.
pub fn build_entry(lores_book: &LoResBook, feed: &Element) -> Element {
    let book = &lores_book.book;
    let mut entry = Element::new_with_namespaces((ATOM_NS, "entry"), feed);
    entry
        .append_new_child((ATOM_NS, "id"))
        .set_text(format!("urn:uuid:{}", book.id));
    entry.append_new_child((ATOM_NS, "title")).set_text(&book.title);

    let updated = if !book.date.is_empty() {
        Some(format!("{}T00:00:00Z", book.date))
    } else {
        None
    };
    if let Some(updated) = &updated {
        entry.append_new_child((ATOM_NS, "updated")).set_text(updated);
    }

    if !book.description.is_empty() {
        entry.append_new_child((ATOM_NS, "summary")).set_text(&book.description);
    }

    if !book.language.is_empty() {
        entry.append_new_child((ATOM_NS, "language")).set_text(&book.language);
    }

    if !book.name.is_empty() {
        entry.append_new_child((ATOM_NS, "name")).set_text(&book.name);
    }

    if !book.flavour.is_empty() {
        entry.append_new_child((ATOM_NS, "flavour")).set_text(&book.flavour);
    }

    if !book.category.is_empty() {
        entry.append_new_child((ATOM_NS, "category")).set_text(&book.category);
    }

    if !book.tags.is_empty() {
        entry.append_new_child((ATOM_NS, "tags")).set_text(&book.tags);
    }

    if book.article_count > 0 {
        entry
            .append_new_child((ATOM_NS, "articleCount"))
            .set_text(book.article_count.to_string());
    }

    if book.media_count > 0 {
        entry
            .append_new_child((ATOM_NS, "mediaCount"))
            .set_text(book.media_count.to_string());
    }

    for illustration in &book.illustrations {
        let size = illustration.width;
        entry
            .append_new_child((ATOM_NS, "link"))
            .set_attr("rel", "http://opds-spec.org/image/thumbnail")
            .set_attr("href", format!("/catalog/v2/illustration/{}/?size={}", book.id, size))
            .set_attr(
                "type",
                format!("{};width={};height={};scale=1", illustration.mime_type, size, size),
            );
    }

    entry
        .append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "http://opds-spec.org/acquisition/open-access")
        .set_attr("href", format!("/content/{}", book.id))
        .set_attr("type", "text/html");

    entry
        .append_new_child((ATOM_NS, "link"))
        .set_attr("type", "text/html")
        .set_attr("href", format!("/content/{}", book.id));

    if !book.creator.is_empty() {
        entry
            .append_new_child((ATOM_NS, "author"))
            .append_new_child((ATOM_NS, "name"))
            .set_text(&book.creator);
    }

    if !book.publisher.is_empty() {
        entry
            .append_new_child((ATOM_NS, "publisher"))
            .append_new_child((ATOM_NS, "name"))
            .set_text(&book.publisher);
    }

    if let Some(updated) = updated {
        entry.append_new_child((ATOM_NS, "dc:issued")).set_text(updated);
    }

    match lores_book.source {
        LoResBookSource::Local => {
            entry.append_new_child((ATOM_NS, "source")).set_text("local");
        }
        LoResBookSource::Remote => {
            entry.append_new_child((ATOM_NS, "source")).set_text("remote");
            let holdings_el = entry.append_new_child((ATOM_NS, "holdings"));
            for node_id in &lores_book.holdings {
                holdings_el.append_new_child((ATOM_NS, "holding")).set_text(node_id);
            }
        }
    }

    entry
}

/// Build the root `<feed>` element for the catalog response.
pub fn build_feed_root(now: DateTime<Utc>, query: &str, total: usize, start: usize, items_per_page: usize) -> Element {
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

    feed.append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    feed.append_new_child((ATOM_NS, "totalResults"))
        .set_text(total.to_string());
    feed.append_new_child((ATOM_NS, "startIndex"))
        .set_text(start.to_string());
    feed.append_new_child((ATOM_NS, "itemsPerPage"))
        .set_text(items_per_page.to_string());

    feed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::render_xml;
    use indoc::indoc;

    use pretty_assertions::assert_eq;

    fn normalize(s: &str) -> Vec<&str> {
        s.lines().map(str::trim).filter(|l| !l.is_empty()).collect()
    }

    fn make_book_row(id: &str) -> crate::projection::books::BookRow {
        crate::projection::books::BookRow {
            id: id.to_string(),
            date: "2026-06-01".to_string(),
            title: "Test Book".to_string(),
            description: "A test description.".to_string(),
            language: "eng".to_string(),
            creator: "Test Author".to_string(),
            ..crate::projection::books::BookRow::default()
        }
    }

    #[test]
    fn build_entry_renders_xml() {
        let book = crate::projection::books::BookRow {
            id: "abc123".to_string(),
            date: "2026-06-01".to_string(),
            flavour: "nopic".to_string(),
            ..crate::projection::books::BookRow::default()
        };
        let feed = Element::new((ATOM_NS, "feed"));

        let book: LoResBook = book.into();
        let xml = render_xml(&build_entry(&book, &feed)).expect("render failed");
        let xml_str = String::from_utf8(xml).unwrap();

        let expected = indoc! {r#"
            <?xml version="1.0" encoding="utf-8"?>
            <entry>
              <id>urn:uuid:abc123</id>
              <title />
              <updated>2026-06-01T00:00:00Z</updated>
              <flavour>nopic</flavour>
              <link href="/content/abc123" rel="http://opds-spec.org/acquisition/open-access" type="text/html" />
              <link href="/content/abc123" type="text/html" />
              <dc:issued>2026-06-01T00:00:00Z</dc:issued>
              <source>remote</source>
              <holdings />
            </entry>
        "#};

        assert_eq!(normalize(&xml_str), normalize(expected));
    }

    #[test]
    fn build_feed_renders_xml() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
            .unwrap()
            .to_utc();
        let mut feed = build_feed_root(now, "language=eng", 1, 0, 10);
        let book: LoResBook = make_book_row("abc123").into();
        feed.append_child(build_entry(&book, &feed));

        let xml = render_xml(&feed).expect("render failed");
        let xml_str = String::from_utf8(xml).unwrap();

        let expected = indoc! {r#"
            <?xml version="1.0" encoding="utf-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/terms/" xmlns:opds="https://specs.opds.io/opds-1.2" xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">
              <id>urn:uuid:lores-kiwix:entries?language=eng</id>
              <link href="/catalog/v2/entries?language=eng" rel="self" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
              <link href="/catalog/v2/root.xml" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <link href="/catalog/v2/root.xml" rel="up" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <title>Filtered Entries (language=eng)</title>
              <updated>2026-08-22T12:00:00Z</updated>
              <totalResults>1</totalResults>
              <startIndex>0</startIndex>
              <itemsPerPage>10</itemsPerPage>
              <entry>
                <id>urn:uuid:abc123</id>
                <title>Test Book</title>
                <updated>2026-06-01T00:00:00Z</updated>
                <summary>A test description.</summary>
                <language>eng</language>
                <link href="/content/abc123" rel="http://opds-spec.org/acquisition/open-access" type="text/html" />
                <link href="/content/abc123" type="text/html" />
                <author>
                  <name>Test Author</name>
                </author>
                <dc:issued>2026-06-01T00:00:00Z</dc:issued>
                <source>remote</source>
                <holdings />
              </entry>
            </feed>
        "#};

        assert_eq!(normalize(&xml_str), normalize(expected));
    }
}
