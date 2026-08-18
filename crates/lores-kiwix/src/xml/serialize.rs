use elementtree::Element;

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
