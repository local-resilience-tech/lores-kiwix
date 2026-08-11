use serde::{Deserialize, Serialize};

/// ZIM file registration data.
///
/// `book_id` is the ZIM archive's embedded UUID, read by libkiwix from the
/// file itself. It is stable across runs and across different machines/servers
/// loading the same `.zim` file, so it is the canonical identifier for
/// comparing installations.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZimRegisteredDataV1 {
    pub filename: String,
    pub book_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppOperation {
    ZimRegisteredV1(ZimRegisteredDataV1),
}
