use chrono::{DateTime, Utc};
use elementtree::Element;
use lores_app_node::NodeInfo;

use crate::projection::nodes::NodeRow;

use super::{ATOM_NS, OPDS_NS};

pub fn build_feed_root(now: DateTime<Utc>) -> Element {
    let mut feed = Element::new((ATOM_NS, "feed"));
    feed.register_namespace(OPDS_NS, Some("opds"));

    feed.append_new_child((ATOM_NS, "id"))
        .set_text("urn:uuid:lores-kiwix:holding_libraries");
    feed.append_new_child((ATOM_NS, "title"))
        .set_text("List of holding libraries");
    feed.append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    feed
}

pub fn build_holding_library(node: &NodeRow, info: Option<&NodeInfo>, now: DateTime<Utc>, feed: &Element) -> Element {
    let mut entry = Element::new_with_namespaces((ATOM_NS, "entry"), feed);

    let title = info.and_then(|i| i.name.as_deref()).unwrap_or(&node.id);
    entry.append_new_child((ATOM_NS, "title")).set_text(title);
    entry
        .append_new_child((ATOM_NS, "updated"))
        .set_text(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    entry
        .append_new_child((ATOM_NS, "id"))
        .set_text(format!("urn:uuid:lores-kiwix:categories:{}", node.id));
    entry
}
