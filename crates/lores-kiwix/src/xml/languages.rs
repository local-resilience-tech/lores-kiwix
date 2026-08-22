use chrono::{DateTime, Utc};
use elementtree::Element;
use libkiwix_rust::LanguageEntry;

use super::{ATOM_NS, DC_NS, OPDS_NS};

const THR_NS: &str = "http://purl.org/syndication/thread/1.0";

/// Build the root `<feed>` element for the languages navigation feed.
pub fn build_feed_root(now: DateTime<Utc>) -> Element {
    let mut feed = Element::new((ATOM_NS, "feed"));
    feed.register_namespace(DC_NS, Some("dc"));
    feed.register_namespace(OPDS_NS, Some("opds"));
    feed.register_namespace(THR_NS, Some("thr"));

    feed.append_new_child((ATOM_NS, "id"))
        .set_text("urn:uuid:lores-kiwix:languages");
    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "self")
        .set_attr("href", "/catalog/v2/languages")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");
    feed.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "start")
        .set_attr("href", "/catalog/v2/root.xml")
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=navigation");
    feed.append_new_child((ATOM_NS, "title")).set_text("List of languages");
    feed.append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    feed
}

/// Build an Atom `<entry>` element for a single language.
pub fn build_language(entry: &LanguageEntry, now: DateTime<Utc>, feed: &Element) -> Element {
    let mut el = Element::new_with_namespaces((ATOM_NS, "entry"), feed);
    el.append_new_child((ATOM_NS, "title")).set_text(entry.lang_name.as_str());
    el.append_new_child((DC_NS, "language")).set_text(entry.lang_code.as_str());
    el.append_new_child((THR_NS, "count"))
        .set_text(entry.book_count.to_string());
    el.append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "subsection")
        .set_attr("href", format!("/catalog/v2/entries?lang={}", entry.lang_code))
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=acquisition");
    el.append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    el.append_new_child((ATOM_NS, "id"))
        .set_text(format!("urn:uuid:lores-kiwix:languages:{}", entry.lang_code));
    el
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
    fn build_language_feed_xml() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
            .unwrap()
            .to_utc();
        let mut feed = build_feed_root(now);
        let languages = vec![
            LanguageEntry { lang_code: "eng".to_string(), lang_name: "English".to_string(), book_count: 3 },
        ];
        for lang in &languages {
            let entry = build_language(lang, now, &feed);
            feed.append_child(entry);
        }

        let xml = render_xml(&feed).expect("render failed");
        let xml_str = String::from_utf8(xml).unwrap();

        let expected = indoc! {r#"
            <?xml version="1.0" encoding="utf-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/terms/" xmlns:opds="https://specs.opds.io/opds-1.2" xmlns:thr="http://purl.org/syndication/thread/1.0">
              <id>urn:uuid:lores-kiwix:languages</id>
              <link href="/catalog/v2/languages" rel="self" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <link href="/catalog/v2/root.xml" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation" />
              <title>List of languages</title>
              <updated>2026-08-22T12:00:00Z</updated>
              <entry>
                <title>English</title>
                <dc:language>eng</dc:language>
                <thr:count>3</thr:count>
                <link href="/catalog/v2/entries?lang=eng" rel="subsection" type="application/atom+xml;profile=opds-catalog;kind=acquisition" />
                <updated>2026-08-22T12:00:00Z</updated>
                <id>urn:uuid:lores-kiwix:languages:eng</id>
              </entry>
            </feed>
        "#};

        assert_eq!(normalize(&xml_str), normalize(expected));
    }
}
