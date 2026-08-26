mod zim_stores;
use libkiwix_rust::{self as kiwix};

use lores_app_node::AppNode;
pub use zim_stores::add_path_to_library;

use crate::{node::operations::AppOperation, utilities::books::registered_data_from_metadata};

pub async fn sync_filesystem(library: &mut kiwix::Library, node: &AppNode<AppOperation>, path: &str) {
    let books = add_path_to_library(library, path);

    for book in &books {
        let op = AppOperation::BookRegisteredV1(registered_data_from_metadata(&book));
        if let Err(e) = node.publish(&op).await {
            eprintln!("Failed to publish BookRegisteredV1 for {}: {}", book.id, e);
        }
    }
}
