use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OpenApiSources {
    pub sources: Vec<OpenApiSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum OpenApiSource {
    #[serde(rename = "url")]
    Url {
        id: String,
        name: String,
        url: String,
        #[serde(rename = "lastFetchedAt")]
        last_fetched_at: Option<String>,
        #[serde(rename = "lastHash")]
        last_hash: Option<String>,
    },
    #[serde(rename = "file")]
    File {
        id: String,
        name: String,
        path: String,
        #[serde(rename = "lastFetchedAt")]
        last_fetched_at: Option<String>,
        #[serde(rename = "lastHash")]
        last_hash: Option<String>,
    },
}

impl OpenApiSource {
    pub fn id(&self) -> &str {
        match self {
            OpenApiSource::Url { id, .. } => id,
            OpenApiSource::File { id, .. } => id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ImportableOperation {
    pub tag: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
}

#[cfg(test)]
#[path = "tests/openapi.rs"]
mod tests;
