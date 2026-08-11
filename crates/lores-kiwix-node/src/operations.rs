use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZimRegisteredDataV1 {
    pub path: String,
    pub book_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppOperation {
    ZimRegisteredV1(ZimRegisteredDataV1),
}
