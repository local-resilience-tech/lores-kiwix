pub mod categories;
pub mod entries;
pub mod languages;

use elementtree::Element;

const ATOM_NS: &str = "http://www.w3.org/2005/Atom";
const DC_NS: &str = "http://purl.org/dc/terms/";
const OPDS_NS: &str = "https://specs.opds.io/opds-1.2";
const OPENSEARCH_NS: &str = "http://a9.com/-/spec/opensearch/1.1/";

pub fn render_xml(element: &Element) -> Result<Vec<u8>, elementtree::Error> {
    let mut buf = Vec::new();
    element.to_writer_with_options(&mut buf, elementtree::WriteOptions::new().set_perform_indent(true))?;
    Ok(buf)
}

fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
