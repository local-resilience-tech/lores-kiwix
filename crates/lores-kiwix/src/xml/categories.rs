use elementtree::Element;

use super::{ATOM_NS, OPDS_NS, rfc3339_now, url_encode};

/// Build the root `<feed>` element for the categories navigation feed.
pub fn build_feed_root() -> Element {
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
    feed.append_new_child((ATOM_NS, "updated")).set_text(rfc3339_now());

    feed
}

/// Build an Atom `<entry>` element for a single category.
pub fn build_category(category: &str, feed: &Element) -> Element {
    let encoded = url_encode(category);
    let mut entry = Element::new_with_namespaces((ATOM_NS, "entry"), feed);
    entry.append_new_child((ATOM_NS, "title")).set_text(category);
    entry
        .append_new_child((ATOM_NS, "link"))
        .set_attr("rel", "subsection")
        .set_attr("href", format!("/catalog/v2/entries?category={}", encoded))
        .set_attr("type", "application/atom+xml;profile=opds-catalog;kind=acquisition");
    entry.append_new_child((ATOM_NS, "updated")).set_text(rfc3339_now());
    entry
        .append_new_child((ATOM_NS, "id"))
        .set_text(format!("urn:uuid:lores-kiwix:categories:{}", encoded));
    entry
        .append_new_child((ATOM_NS, "content"))
        .set_attr("type", "text")
        .set_text(format!("All entries with category of '{}'.", category));
    entry
}
