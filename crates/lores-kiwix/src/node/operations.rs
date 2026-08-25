use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BookRegisteredDataV1 {
    pub filename: String,
    pub book_id: String,
    pub name: String,
    pub date: String,
    pub flavour: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub creator: String,
    pub publisher: String,
    pub category: String,
    pub tags: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AppOperation {
    BookRegisteredV1(BookRegisteredDataV1),
}
