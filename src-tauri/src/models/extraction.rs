use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "tests/extraction.rs"]
mod tests;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Extraction {
    pub variable: String,
    pub from: String,
    pub path: String,
}
