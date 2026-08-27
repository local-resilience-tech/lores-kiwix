use crate::node::operations::BookRegisteredDataV1;
use libkiwix_rust::BookMetadata;

pub fn registered_data_from_metadata(meta: &BookMetadata) -> BookRegisteredDataV1 {
    BookRegisteredDataV1 {
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
