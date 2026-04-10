use crate::models::extraction::Extraction;
use crate::models::pre_request_action::PreRequestAction;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
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
    #[serde(
        default,
        skip_serializing_if = "IndexMap::is_empty",
        rename = "pathParams"
    )]
    pub path_params: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "templateRef")]
    pub template_ref: Option<TemplateRef>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "disabledHeaders"
    )]
    pub disabled_headers: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "disabledCollectionHeaders"
    )]
    pub disabled_collection_headers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extractions: Vec<Extraction>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "preRequestActions"
    )]
    pub pre_request_actions: Vec<PreRequestAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    Custom { headers: IndexMap<String, String> },
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum BodyConfig {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "form-urlencoded")]
    FormUrlEncoded {
        content: IndexMap<String, String>,
        #[serde(
            default,
            skip_serializing_if = "Vec::is_empty",
            rename = "disabledFields"
        )]
        disabled_fields: Vec<String>,
    },
    #[serde(rename = "raw")]
    Raw { format: RawFormat, content: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RawFormat {
    Json,
    Xml,
    Text,
}

impl<'de> serde::Deserialize<'de> for BodyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum Helper {
            #[serde(rename = "none")]
            None,
            #[serde(rename = "form-urlencoded")]
            FormUrlEncoded {
                content: IndexMap<String, String>,
                #[serde(default, rename = "disabledFields")]
                disabled_fields: Vec<String>,
            },
            #[serde(rename = "raw")]
            Raw {
                #[serde(default)]
                format: Option<RawFormat>,
                content: String,
            },
            #[serde(rename = "json")]
            LegacyJson { content: serde_json::Value },
            #[serde(rename = "form")]
            LegacyForm {
                content: IndexMap<String, String>,
                #[serde(default, rename = "disabledFields")]
                disabled_fields: Vec<String>,
            },
        }

        Ok(match Helper::deserialize(deserializer)? {
            Helper::None => BodyConfig::None,
            Helper::FormUrlEncoded {
                content,
                disabled_fields,
            } => BodyConfig::FormUrlEncoded {
                content,
                disabled_fields,
            },
            Helper::Raw { format, content } => BodyConfig::Raw {
                format: format.unwrap_or(RawFormat::Text),
                content,
            },
            Helper::LegacyJson { content } => {
                let s = match &content {
                    serde_json::Value::String(s) => s.clone(),
                    v => serde_json::to_string_pretty(v).unwrap_or_default(),
                };
                BodyConfig::Raw {
                    format: RawFormat::Json,
                    content: s,
                }
            }
            Helper::LegacyForm {
                content,
                disabled_fields,
            } => BodyConfig::FormUrlEncoded {
                content,
                disabled_fields,
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

pub fn derive_request_id(project_root: &Path, request_path: &Path) -> crate::error::Result<String> {
    let relative = request_path.strip_prefix(project_root).map_err(|_| {
        crate::error::FlupiError::Custom("Request path is not under project root".to_string())
    })?;
    let parts: Vec<&str> = relative
        .iter()
        .map(|s| {
            s.to_str().ok_or_else(|| {
                crate::error::FlupiError::Custom("Invalid path component".to_string())
            })
        })
        .collect::<crate::error::Result<Vec<_>>>()?;

    if parts.is_empty() {
        return Err(crate::error::FlupiError::Custom(
            "Request path has no components".to_string(),
        ));
    }

    if parts[0] == "collections" {
        // collections/{folderName}/requests/[subpath/]fileName.json
        if parts.len() < 4 {
            return Err(crate::error::FlupiError::Custom(
                "Invalid collection request path structure".to_string(),
            ));
        }
        let folder_name = parts[1];
        // Skip "requests" at parts[2]
        let rest: Vec<&str> = parts[3..].to_vec();
        if rest.is_empty() {
            return Err(crate::error::FlupiError::Custom(
                "Collection request path has no file".to_string(),
            ));
        }
        let file_stem = Path::new(rest.last().unwrap())
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                crate::error::FlupiError::Custom("Unable to extract file stem".to_string())
            })?;
        let mut id_parts = vec![folder_name];
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        Ok(id_parts.join("/"))
    } else {
        // requests/[subpath/]fileName.json
        if parts.len() < 2 {
            return Err(crate::error::FlupiError::Custom(
                "Invalid root request path structure".to_string(),
            ));
        }
        let rest: Vec<&str> = parts[1..].to_vec();
        if rest.is_empty() {
            return Err(crate::error::FlupiError::Custom(
                "Root request path has no file".to_string(),
            ));
        }
        let file_stem = Path::new(rest.last().unwrap())
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                crate::error::FlupiError::Custom("Unable to extract file stem".to_string())
            })?;
        let mut id_parts: Vec<&str> = Vec::new();
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        Ok(id_parts.join("/"))
    }
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
