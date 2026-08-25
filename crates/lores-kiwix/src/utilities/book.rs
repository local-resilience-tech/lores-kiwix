use libkiwix_rust::BookMetadata;

use crate::projection::books::BookRow;

#[derive(Clone)]
pub enum LoResBookSource {
    Local,
    Remote,
}

#[derive(Clone)]
pub struct LoResBook {
    pub book: BookMetadata,
    pub source: LoResBookSource,
    /// Node IDs holding this book; empty for local books.
    pub holdings: Vec<String>,
}

impl Into<LoResBook> for BookMetadata {
    fn into(self) -> LoResBook {
        LoResBook {
            book: self,
            source: LoResBookSource::Local,
            holdings: Vec::new(),
        }
    }
}

impl Into<LoResBook> for BookRow {
    fn into(self) -> LoResBook {
        LoResBook {
            book: self.into(),
            source: LoResBookSource::Remote,
            holdings: Vec::new(),
        }
    }
}
