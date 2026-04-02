use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum FlupiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Http(String),

    #[error("{0}")]
    Custom(String),
}

impl From<reqwest::Error> for FlupiError {
    fn from(err: reqwest::Error) -> Self {
        if let Some(status) = err.status() {
            FlupiError::Http(format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")))
        } else if err.is_timeout() {
            FlupiError::Http("Request timed out".to_string())
        } else if err.is_connect() {
            FlupiError::Http(format!("Could not connect: {}", err.url().map(|u| u.as_str()).unwrap_or("unknown URL")))
        } else {
            FlupiError::Http(format!("Network error: {err}"))
        }
    }
}

impl Serialize for FlupiError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, FlupiError>;
