use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "tests/extraction.rs"]
mod tests;

fn default_scope() -> String {
    "env".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Extraction {
    pub variable: String,
    pub from: String,
    pub path: String,
    #[serde(default = "default_scope")]
    pub scope: String, // "env" | "scenario"
}
