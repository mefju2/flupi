use std::path::Path;
use sha2::{Sha256, Digest};
use crate::error::{FlupiError, Result};
use crate::models::openapi::ImportableOperation;
use crate::models::request::{BodyConfig, Request, TemplateRef};
use crate::services::{file_io, schema_defaults};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head", "options", "trace"];

pub async fn fetch_spec_from_url(url: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let value = resp.json::<serde_json::Value>().await?;
    Ok(value)
}

pub fn read_spec_from_file(path: &Path) -> Result<serde_json::Value> {
    file_io::read_json(path)
}

fn derive_operation_id(method: &str, path: &str) -> String {
    let mut parts = vec![method.to_lowercase()];
    for segment in path.split('/') {
        if segment.is_empty() { continue; }
        let clean = segment
            .trim_start_matches('{').trim_end_matches('}')
            .trim_start_matches(':');
        let camel: String = clean.split(|c| c == '-' || c == '.')
            .filter(|s| !s.is_empty())
            .enumerate()
            .map(|(_, word)| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect();
        parts.push(camel);
    }
    parts.join("")
}

pub fn parse_operations(spec: &serde_json::Value) -> Result<Vec<(ImportableOperation, serde_json::Value)>> {
    let paths = spec["paths"].as_object().ok_or_else(|| {
        FlupiError::Custom("OpenAPI spec missing 'paths' object".to_string())
    })?;

    let mut result = Vec::new();

    for (path_str, path_item) in paths {
        let path_obj = match path_item.as_object() {
            Some(o) => o,
            None => continue,
        };

        for (method, operation) in path_obj {
            if !HTTP_METHODS.contains(&method.as_str()) {
                continue;
            }

            let op_obj = match operation.as_object() {
                Some(o) => o,
                None => continue,
            };

            let operation_id = match op_obj.get("operationId").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => derive_operation_id(method, path_str),
            };

            let tag = op_obj
                .get("tags")
                .and_then(|t| t.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            let summary = op_obj
                .get("summary")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let importable = ImportableOperation {
                tag,
                operation_id,
                method: method.clone(),
                path: path_str.clone(),
                summary,
            };

            result.push((importable, operation.clone()));
        }
    }

    Ok(result)
}

pub fn compute_operation_hash(operation: &serde_json::Value) -> String {
    let serialized = serde_json::to_string(operation).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

fn extract_request_schema(operation: &serde_json::Value, spec: &serde_json::Value) -> serde_json::Value {
    let raw = operation
        .pointer("/requestBody/content/application~1json/schema")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if raw.is_null() {
        return serde_json::Value::Null;
    }
    schema_defaults::resolve_refs(&raw, spec, 0)
}

fn extract_response_schema(operation: &serde_json::Value, spec: &serde_json::Value) -> serde_json::Value {
    let resolve = |raw: &serde_json::Value| schema_defaults::resolve_refs(raw, spec, 0);

    if let Some(schema) = operation.pointer("/responses/200/content/application~1json/schema") {
        return resolve(schema);
    }
    if let Some(responses) = operation["responses"].as_object() {
        for code in &["201", "202", "203", "204"] {
            if let Some(schema) = responses
                .get(*code)
                .and_then(|r| r.pointer("/content/application~1json/schema"))
            {
                return resolve(schema);
            }
        }
    }
    serde_json::Value::Null
}

pub fn import_operations(
    project_path: &Path,
    source_id: &str,
    operations: &[(ImportableOperation, serde_json::Value)],
    collection_folder: &str,
    spec: &serde_json::Value,
) -> Result<Vec<String>> {
    let import_timestamp = chrono::Utc::now().to_rfc3339();
    let mut created_ids = Vec::new();

    for (op, op_json) in operations {
        let schema_hash = compute_operation_hash(op_json);
        let request_schema = extract_request_schema(op_json, spec);
        let response_schema = extract_response_schema(op_json, spec);

        let body = if request_schema.is_null() {
            None
        } else {
            let content = schema_defaults::generate_default_body(&request_schema, &import_timestamp);
            Some(BodyConfig::Json { content })
        };

        let request = Request {
            name: op.summary.clone().unwrap_or_else(|| op.operation_id.clone()),
            method: op.method.to_uppercase(),
            path: op.path.clone(),
            auth: None,
            headers: indexmap::IndexMap::new(),
            path_params: indexmap::IndexMap::new(),
            body,
            template_ref: Some(TemplateRef {
                source_id: source_id.to_string(),
                operation_id: op.operation_id.clone(),
                schema_hash,
                request_schema,
                response_schema,
            }),
            disabled_headers: vec![],
            disabled_collection_headers: vec![],
            extractions: vec![],
        };

        let file_path = project_path
            .join("collections")
            .join(collection_folder)
            .join("requests")
            .join(format!("{}.json", op.operation_id));

        file_io::write_json(&file_path, &request)?;

        let request_id = format!("{}/{}", collection_folder, op.operation_id);
        created_ids.push(request_id);
    }

    Ok(created_ids)
}

#[cfg(test)]
#[path = "tests/openapi_import.rs"]
mod tests;
