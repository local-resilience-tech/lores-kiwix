use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ZimRegisteredDataV1 {}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppOperation {
    ZimRegisteredV1(ZimRegisteredDataV1),
}
