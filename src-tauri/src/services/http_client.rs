use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use crate::error::Result;

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutableRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    #[serde(rename = "json")]
    Json { content: serde_json::Value },
    #[serde(rename = "form")]
    Form { content: HashMap<String, String> },
    #[serde(rename = "raw")]
    Raw { content: String },
}

#[derive(Debug, Serialize, Clone)]
pub struct HttpResponse {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
    #[serde(rename = "bodyTruncated")]
    pub body_truncated: bool,
}

const MAX_BODY_SIZE: usize = 1_048_576; // 1MB

/// Builds a reqwest::Request from an ExecutableRequest (used for testing, no network call).
pub fn build_request(req: &ExecutableRequest) -> Result<reqwest::Request> {
    build_for_client(&HTTP_CLIENT, req)
}

fn build_for_client(client: &reqwest::Client, req: &ExecutableRequest) -> Result<reqwest::Request> {
    let method = reqwest::Method::from_bytes(req.method.to_uppercase().as_bytes())
        .map_err(|e| crate::error::FlupiError::Custom(format!("Invalid HTTP method: {}", e)))?;
    let mut builder = client.request(method, &req.url)
        .timeout(Duration::from_millis(req.timeout_ms));

    for (key, value) in &req.headers {
        builder = builder.header(key, value);
    }

    if let Some(body) = &req.body {
        builder = match body {
            RequestBody::Json { content } => builder.json(content),
            RequestBody::Form { content } => builder.form(content),
            RequestBody::Raw { content } => builder.body(content.clone()),
        };
    }

    Ok(builder.build()?)
}

pub async fn execute_request(req: &ExecutableRequest) -> Result<HttpResponse> {
    let request = build_for_client(&HTTP_CLIENT, req)?;

    let start = std::time::Instant::now();
    let response = HTTP_CLIENT.execute(request).await?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();
    let headers: HashMap<String, String> = response.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Stream response body up to MAX_BODY_SIZE to avoid OOM on large responses
    let mut body_bytes = Vec::new();
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    let mut body_truncated = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        body_bytes.extend_from_slice(&chunk);
        if body_bytes.len() > MAX_BODY_SIZE {
            body_bytes.truncate(MAX_BODY_SIZE);
            body_truncated = true;
            break;
        }
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        duration_ms,
        body_truncated,
    })
}

#[cfg(test)]
#[path = "tests/http_client.rs"]
mod tests;
