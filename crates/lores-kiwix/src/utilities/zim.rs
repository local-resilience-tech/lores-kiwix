use libkiwix_rust::BookMetadata;
use crate::node::operations::ZimRegisteredDataV1;

/// Build a `ZimRegisteredDataV1` from a filesystem path and the
/// `BookMetadata` returned by libkiwix.
pub fn registered_data_from_path_and_metadata(path: &str, meta: &BookMetadata) -> ZimRegisteredDataV1 {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string();

    ZimRegisteredDataV1 {
        filename,
        book_id: meta.id.clone(),
        name: meta.name.clone(),
        date: meta.date.clone(),
        flavour: meta.flavour.clone(),
        title: meta.title.clone(),
        description: meta.description.clone(),
        language: meta.language.clone(),
        creator: meta.creator.clone(),
        publisher: meta.publisher.clone(),
        category: meta.category.clone(),
        tags: meta.tags.clone(),
    }
}
