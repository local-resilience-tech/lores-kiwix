use elementtree::Element;
use libkiwix_rust::BookMetadata;

use crate::projection::zims::Zim;

pub const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

pub fn build_entry(zim: &Zim, namespace: &str, feed: &Element) -> Element {
    let mut entry = Element::new_with_namespaces((namespace, "entry"), feed);
    entry.append_new_child((namespace, "id")).set_text(&zim.id);

    if let Some(title) = &zim.title {
        entry.append_new_child((namespace, "title")).set_text(title);
    }

    if let Some(date) = &zim.date
        && !date.is_empty()
    {
        entry.append_new_child((namespace, "updated")).set_text(date);
    }

    if let Some(creator) = &zim.creator
        && !creator.is_empty()
    {
        entry
            .append_new_child((namespace, "author"))
            .append_new_child((namespace, "name"))
            .set_text(creator);
    }

    if let Some(description) = &zim.description
        && !description.is_empty()
    {
        entry
            .append_new_child((namespace, "content"))
            .set_attr("type", "text/plain")
            .set_text(description);
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

/// Build an Atom `<entry>` element from a libkiwix `BookMetadata`.
///
/// This mirrors the libkiwix `catalog_v2_entry.xml` mustache template as
/// closely as possible.
pub fn build_entry_from_metadata(meta: &BookMetadata, namespace: &str, feed: &Element) -> Element {
    let mut entry = Element::new_with_namespaces((namespace, "entry"), feed);
    entry.append_new_child((namespace, "id"))
        .set_text(format!("urn:uuid:{}", meta.id));
    entry.append_new_child((namespace, "title"))
        .set_text(&meta.title);

    let updated = format!("{}T00:00:00Z", meta.date);
    entry.append_new_child((namespace, "updated"))
        .set_text(&updated);

    if !meta.description.is_empty() {
        entry.append_new_child((namespace, "summary"))
            .set_text(&meta.description);
    }

    if !meta.language.is_empty() {
        entry.append_new_child((namespace, "language"))
            .set_text(&meta.language);
    }

    if !meta.name.is_empty() {
        entry.append_new_child((namespace, "name"))
            .set_text(&meta.name);
    }

    if !meta.flavour.is_empty() {
        entry.append_new_child((namespace, "flavour"))
            .set_text(&meta.flavour);
    }

    if !meta.category.is_empty() {
        entry.append_new_child((namespace, "category"))
            .set_text(&meta.category);
    }

    if !meta.tags.is_empty() {
        entry.append_new_child((namespace, "tags"))
            .set_text(&meta.tags);
    }

    if meta.article_count > 0 {
        entry.append_new_child((namespace, "articleCount"))
            .set_text(meta.article_count.to_string());
    }

    if meta.media_count > 0 {
        entry.append_new_child((namespace, "mediaCount"))
            .set_text(meta.media_count.to_string());
    }

    for illustration in &meta.illustrations {
        let size = illustration.width;
        entry
            .append_new_child((namespace, "link"))
            .set_attr("rel", "http://opds-spec.org/image/thumbnail")
            .set_attr(
                "href",
                format!("/catalog/v2/illustration/{}/?size={}", meta.id, size),
            )
            .set_attr(
                "type",
                format!("{};width={};height={};scale=1", illustration.mime_type, size, size),
            );
    }

    entry
        .append_new_child((namespace, "link"))
        .set_attr("type", "text/html")
        .set_attr("href", format!("/content/{}", meta.id));

    if !meta.creator.is_empty() {
        entry
            .append_new_child((namespace, "author"))
            .append_new_child((namespace, "name"))
            .set_text(&meta.creator);
    }

    if !meta.publisher.is_empty() {
        entry
            .append_new_child((namespace, "publisher"))
            .append_new_child((namespace, "name"))
            .set_text(&meta.publisher);
    }

    if !meta.date.is_empty() {
        entry
            .append_new_child((namespace, "dc:issued"))
            .set_text(&updated);
    }

    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_entry_renders_xml() {
        let zim = Zim {
            id: "abc123".to_string(),
            filename: "abc123.zim".to_string(),
            date: Some("2026-06-01".to_string()),
            flavour: Some("nopic".to_string()),
            ..Zim::default()
        };
        let feed = Element::new((ATOM_NS, "feed"));

        let result = build_entry(&zim, ATOM_NS, &feed).to_string().unwrap();

        assert_eq!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><entry><id>abc123</id><updated>2026-06-01</updated><link href=\"/content/abc123\" rel=\"http://opds-spec.org/acquisition/open-access\" type=\"text/html\" /></entry>",
            result
        );
    }
}

// pub title: Option<String>,
// pub description: Option<String>,
// pub language: Option<String>,
// pub creator: Option<String>,
// pub publisher: Option<String>,
