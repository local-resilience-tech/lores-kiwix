pub mod categories;
pub mod entries;

use elementtree::Element;

pub const ATOM_NS: &str = "http://www.w3.org/2005/Atom";
pub const DC_NS: &str = "http://purl.org/dc/terms/";
pub const OPDS_NS: &str = "https://specs.opds.io/opds-1.2";
pub const OPENSEARCH_NS: &str = "http://a9.com/-/spec/opensearch/1.1/";

pub fn render_xml(element: &Element) -> Result<Vec<u8>, elementtree::Error> {
    let mut buf = Vec::new();
    element.to_writer_with_options(&mut buf, elementtree::WriteOptions::new().set_perform_indent(true))?;
    Ok(buf)
}
