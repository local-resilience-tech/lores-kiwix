mod zim_stores;

use libkiwix_rust::{self as kiwix, BookMetadata};
use lores_app_node::AppNode;
use sqlx::SqlitePool;

pub use zim_stores::add_path_to_library;

use crate::{
    node::operations::{AppOperation, BookDeregisteredDataV1},
    projection::holdings::locally_held_book_ids,
    utilities::books::registered_data_from_metadata,
};

pub async fn sync_filesystem(
    library: &mut kiwix::Library,
    pool: &SqlitePool,
    node: &AppNode<AppOperation>,
    path: &str,
) {
    let books = add_path_to_library(library, path);

    let persisted_book_ids = match locally_held_book_ids(pool).await {
        Ok(book_ids) => book_ids,
        Err(err) => {
            tracing::error!(error = %err, "Failed to get persisted book ids");
            return;
        }
    };

    println!("persisted book ids {:?}", persisted_book_ids);

    let new_books = new_books(&books, &persisted_book_ids);
    let missing_book_ids = missing_book_ids(&books, &persisted_book_ids);

    println!("new books {:?}", new_books);
    println!("missing book ids {:?}", missing_book_ids);

    publish_book_registrations(&new_books, node).await;
    publish_book_deregistrations(&missing_book_ids, node).await;
}

async fn publish_book_registrations(books: &[BookMetadata], node: &AppNode<AppOperation>) {
    for book in books {
        let op = AppOperation::BookRegisteredV1(registered_data_from_metadata(&book));
        if let Err(e) = node.publish(&op).await {
            eprintln!("Failed to publish BookRegisteredV1 for {}: {}", book.id, e);
        }
    }
}

async fn publish_book_deregistrations(book_ids: &[String], node: &AppNode<AppOperation>) {
    for book_id in book_ids {
        let op = AppOperation::BookDeregisteredV1(BookDeregisteredDataV1 {
            book_id: book_id.clone(),
        });
        if let Err(e) = node.publish(&op).await {
            eprintln!("Failed to publish BookDeregisteredV1 for {}: {}", book_id, e);
        }
    }
}

fn new_books(books: &[BookMetadata], existing_ids: &Vec<String>) -> Vec<BookMetadata> {
    books
        .iter()
        .filter(|book| !existing_ids.contains(&book.id))
        .cloned()
        .collect()
}

fn missing_book_ids(books: &[BookMetadata], existing_ids: &Vec<String>) -> Vec<String> {
    let book_ids: Vec<String> = books.iter().map(|book| book.id.clone()).collect();

    existing_ids
        .iter()
        .filter(|book_id| !book_ids.contains(&book_id))
        .cloned()
        .collect()
}
