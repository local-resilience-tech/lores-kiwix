use chrono::{DateTime, Utc};
use elementtree::Element;

use super::{ATOM_NS, OPDS_NS, url_encode};

/// Build the root `<feed>` element for the categories navigation feed.
pub fn build_feed_root(now: DateTime<Utc>) -> Element {
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
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    feed
}

/// Build an Atom `<entry>` element for a single category.
pub fn build_category(category: &str, now: DateTime<Utc>, feed: &Element) -> Element {
    let encoded = url_encode(category);
    let mut entry = Element::new_with_namespaces((ATOM_NS, "entry"), feed);
    entry.append_new_child((ATOM_NS, "title")).set_text(category);
    entry
        .append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "subsection")
        .set_attr("href", format!("/catalog/v2/entries?category={}", encoded))
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=acquisition");
    entry
        .append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    entry
        .append_new_child((ATOM_NS, "id"))
        .set_text(format!("urn:uuid:lores-kiwix:categories:{}", encoded));
    entry
        .append_new_child((ATOM_NS, "content"))
        .set_attr("type", "text")
        .set_text(format!("All entries with category of '{}'.", category));
    entry
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

    #[test]
    fn build_category_feed_xml() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
            .unwrap()
            .to_utc();
        let mut feed = build_feed_root(now);
        let categories = ["Science", "History", "Technology"];
        for cat in &categories {
            let entry = build_category(cat, now, &feed);
            feed.append_child(entry);
        }

        let xml = render_xml(&feed).expect("render failed");
        let xml_str = String::from_utf8(xml).unwrap();

        let expected = indoc! {r#"
            <?xml version="1.0" encoding="utf-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="https://specs.opds.io/opds-1.2">
              <id>urn:uuid:lores-kiwix:categories</id>
              <link href="/catalog/v2/categories" rel="self" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <link href="/catalog/v2/root.xml" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <title>List of categories</title>
              <updated>2026-08-22T12:00:00Z</updated>
              <entry>
                <title>Science</title>
                <link href="/catalog/v2/entries?category=Science" rel="subsection" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
                <updated>2026-08-22T12:00:00Z</updated>
                <id>urn:uuid:lores-kiwix:categories:Science</id>
                <content type="text">All entries with category of 'Science'.</content>
              </entry>
              <entry>
                <title>History</title>
                <link href="/catalog/v2/entries?category=History" rel="subsection" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
                <updated>2026-08-22T12:00:00Z</updated>
                <id>urn:uuid:lores-kiwix:categories:History</id>
                <content type="text">All entries with category of 'History'.</content>
              </entry>
              <entry>
                <title>Technology</title>
                <link href="/catalog/v2/entries?category=Technology" rel="subsection" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
                <updated>2026-08-22T12:00:00Z</updated>
                <id>urn:uuid:lores-kiwix:categories:Technology</id>
                <content type="text">All entries with category of 'Technology'.</content>
              </entry>
            </feed>
        "#};

        assert_eq!(normalize(&xml_str), normalize(expected));
    }
}
