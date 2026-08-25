use libkiwix_rust::BookMetadata;

use crate::projection::zims::Zim;

#[derive(Clone)]
pub enum LoResBookSource {
    Local,
    Remote,
}

#[derive(Clone)]
pub struct LoResBook {
    pub book: BookMetadata,
    pub source: LoResBookSource,
}

impl Into<LoResBook> for BookMetadata {
    fn into(self) -> LoResBook {
        LoResBook {
            book: self,
            source: LoResBookSource::Local,
        }
    }
}

impl Into<LoResBook> for Zim {
    fn into(self) -> LoResBook {
        LoResBook {
            book: self.into(),
            source: LoResBookSource::Remote,
        }
    }
}
