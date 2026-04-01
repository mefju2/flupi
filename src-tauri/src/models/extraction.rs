use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extraction {
    pub variable: String,
    pub from: String,
    pub path: String,
}
