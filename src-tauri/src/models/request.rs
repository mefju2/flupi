use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub headers: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "templateRef")]
    pub template_ref: Option<TemplateRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AuthConfig {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "inherit")]
    Inherit,
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "basic")]
    Basic { username: String, password: String },
    #[serde(rename = "apiKey")]
    ApiKey { header: String, value: String },
    #[serde(rename = "custom")]
    Custom { headers: HashMap<String, String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum BodyConfig {
    #[serde(rename = "json")]
    Json { content: serde_json::Value },
    #[serde(rename = "form")]
    Form { content: HashMap<String, String> },
    #[serde(rename = "raw")]
    Raw { content: String },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateRef {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    #[serde(rename = "schemaHash")]
    pub schema_hash: String,
    #[serde(rename = "requestSchema")]
    pub request_schema: serde_json::Value,
    #[serde(rename = "responseSchema")]
    pub response_schema: serde_json::Value,
}

pub fn derive_request_id(project_root: &Path, request_path: &Path) -> String {
    let relative = request_path.strip_prefix(project_root).unwrap();
    let parts: Vec<&str> = relative.iter().map(|s| s.to_str().unwrap()).collect();

    if parts[0] == "collections" {
        // collections/{folderName}/requests/[subpath/]fileName.json
        let folder_name = parts[1];
        // Skip "requests" at parts[2]
        let rest: Vec<&str> = parts[3..].to_vec();
        let file_stem = Path::new(rest.last().unwrap()).file_stem().unwrap().to_str().unwrap();
        let mut id_parts = vec![folder_name];
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        id_parts.join("/")
    } else {
        // requests/[subpath/]fileName.json
        let rest: Vec<&str> = parts[1..].to_vec();
        let file_stem = Path::new(rest.last().unwrap()).file_stem().unwrap().to_str().unwrap();
        let mut id_parts: Vec<&str> = Vec::new();
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        id_parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_request_id_collection() {
        let project = Path::new("/project");
        let path = Path::new("/project/collections/auth-service/requests/get-token.json");
        assert_eq!(derive_request_id(project, path), "auth-service/get-token");
    }

    #[test]
    fn test_derive_request_id_collection_nested() {
        let project = Path::new("/project");
        let path = Path::new("/project/collections/auth-service/requests/admin/create-user.json");
        assert_eq!(derive_request_id(project, path), "auth-service/admin/create-user");
    }

    #[test]
    fn test_derive_request_id_root() {
        let project = Path::new("/project");
        let path = Path::new("/project/requests/health-check.json");
        assert_eq!(derive_request_id(project, path), "health-check");
    }

    #[test]
    fn test_derive_request_id_root_nested() {
        let project = Path::new("/project");
        let path = Path::new("/project/requests/monitoring/status.json");
        assert_eq!(derive_request_id(project, path), "monitoring/status");
    }
}
