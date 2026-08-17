use elementtree::Element;

use crate::projection::zims;

pub fn build_entry(zim: &zims::ZimRow, namespace: &str, feed: &Element) -> Element {
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
