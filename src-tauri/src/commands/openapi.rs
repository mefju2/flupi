use std::path::PathBuf;
use serde::Serialize;
use tauri::command;
use crate::error::{FlupiError, Result};
use crate::models::openapi::{ImportableOperation, OpenApiSource, OpenApiSources};
use crate::models::request::derive_request_id;
use crate::services::{file_io, openapi_import};
use crate::AppState;

/// What the frontend needs to render the drift explanation panel.
#[derive(Debug, Serialize)]
pub struct DriftDetails {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    /// The path stored in the request file (what was imported).
    #[serde(rename = "storedPath")]
    pub stored_path: String,
    /// The path currently in the spec — None if the operation was truly deleted.
    #[serde(rename = "currentPath")]
    pub current_path: Option<String>,
    /// When the operationId itself changed (e.g. path rename re-derived the ID),
    /// this holds the new operationId so the UI can show it.
    #[serde(rename = "currentOperationId")]
    pub current_operation_id: Option<String>,
    #[serde(rename = "pathChanged")]
    pub path_changed: bool,
    #[serde(rename = "schemaChanged")]
    pub schema_changed: bool,
    /// True only when the operation is genuinely gone with no plausible rename candidate.
    #[serde(rename = "operationRemoved")]
    pub operation_removed: bool,
}

/// Find a likely-renamed operation by same HTTP method + best path similarity.
/// Returns Some only when similarity exceeds 60% (common-prefix / min-length).
fn find_candidate_operation<'a>(
    stored_method: &str,
    stored_path: &str,
    operations: &'a [(ImportableOperation, serde_json::Value)],
) -> Option<&'a (ImportableOperation, serde_json::Value)> {
    let method_up = stored_method.to_uppercase();
    let min_req_len = (stored_path.len() as f64 * 0.6) as usize;

    operations
        .iter()
        .filter(|(op, _)| op.method.to_uppercase() == method_up)
        .filter_map(|entry| {
            let prefix = common_prefix_len(stored_path, &entry.0.path);
            if prefix >= min_req_len { Some((prefix, entry)) } else { None }
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, entry)| entry)
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

const SOURCES_FILE: &str = "openapi-sources.json";

fn load_sources(project_path: &PathBuf) -> Result<OpenApiSources> {
    let path = project_path.join(SOURCES_FILE);
    if !path.exists() {
        return Ok(OpenApiSources::default());
    }
    file_io::read_json(&path)
}

fn save_sources(project_path: &PathBuf, sources: &OpenApiSources) -> Result<()> {
    let path = project_path.join(SOURCES_FILE);
    file_io::write_json(&path, sources)
}

fn add_source_to_disk(project_path: &PathBuf, source: OpenApiSource) -> Result<()> {
    let mut sources = load_sources(project_path)?;
    sources.sources.push(source);
    save_sources(project_path, &sources)
}

fn remove_source_from_disk(project_path: &PathBuf, source_id: &str) -> Result<()> {
    let mut sources = load_sources(project_path)?;
    sources.sources.retain(|s| s.id() != source_id);
    save_sources(project_path, &sources)
}

#[command]
pub async fn add_openapi_source(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    source: OpenApiSource,
) -> Result<()> {
    let _guard = state.sources_lock.lock().await;
    add_source_to_disk(&project_path, source)
}

#[command]
pub async fn remove_openapi_source(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    source_id: String,
) -> Result<()> {
    let _guard = state.sources_lock.lock().await;
    remove_source_from_disk(&project_path, &source_id)
}

#[command]
pub fn list_openapi_sources(project_path: PathBuf) -> Result<Vec<OpenApiSource>> {
    let sources = load_sources(&project_path)?;
    Ok(sources.sources)
}

async fn get_spec_for_source(source: &OpenApiSource) -> Result<serde_json::Value> {
    match source {
        OpenApiSource::Url { url, .. } => openapi_import::fetch_spec_from_url(url).await,
        OpenApiSource::File { path, .. } => {
            openapi_import::read_spec_from_file(std::path::Path::new(path))
        }
    }
}

#[command]
pub async fn fetch_operations(project_path: PathBuf, source_id: String) -> Result<Vec<ImportableOperation>> {
    let sources = load_sources(&project_path)?;
    let source = sources
        .sources
        .iter()
        .find(|s| s.id() == source_id)
        .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", source_id)))?;

    let spec = get_spec_for_source(source).await?;
    let ops = openapi_import::parse_operations(&spec)?;
    Ok(ops.into_iter().map(|(op, _)| op).collect())
}

#[command]
pub async fn import_operations(
    project_path: PathBuf,
    source_id: String,
    operation_ids: Vec<String>,
    collection_folder: String,
) -> Result<Vec<String>> {
    let sources = load_sources(&project_path)?;
    let source = sources
        .sources
        .iter()
        .find(|s| s.id() == source_id)
        .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", source_id)))?;

    let spec = get_spec_for_source(source).await?;
    let all_ops = openapi_import::parse_operations(&spec)?;

    let filtered: Vec<_> = all_ops
        .into_iter()
        .filter(|(op, _)| operation_ids.contains(&op.operation_id))
        .collect();

    openapi_import::import_operations(&project_path, &source_id, &filtered, &collection_folder, &spec)
}

#[command]
pub async fn refresh_source(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    source_id: String,
) -> Result<Vec<String>> {
    // Fetch and parse the spec outside the lock — network I/O should not hold it.
    let source = {
        let sources = load_sources(&project_path)?;
        sources
            .sources
            .into_iter()
            .find(|s| s.id() == source_id)
            .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", source_id)))?
    };

    let spec = get_spec_for_source(&source).await?;
    let ops = openapi_import::parse_operations(&spec)?;

    let now = chrono::Utc::now().to_rfc3339();
    // Compute a whole-spec hash as metadata — kept for display purposes only.
    let new_hash = openapi_import::compute_spec_hash(&spec);

    {
        let _guard = state.sources_lock.lock().await;
        let mut sources = load_sources(&project_path)?;
        for s in &mut sources.sources {
            if s.id() == source_id {
                *s = match s.clone() {
                    OpenApiSource::Url { id, name, url, .. } => OpenApiSource::Url {
                        id, name, url,
                        last_fetched_at: Some(now.clone()),
                        last_hash: Some(new_hash.clone()),
                    },
                    OpenApiSource::File { id, name, path, .. } => OpenApiSource::File {
                        id, name, path,
                        last_fetched_at: Some(now.clone()),
                        last_hash: Some(new_hash.clone()),
                    },
                };
            }
        }
        save_sources(&project_path, &sources)?;
    } // lock released here

    let drifted = crate::services::drift_detection::detect_drift(&project_path, &source_id, &ops, &spec)?;
    Ok(drifted)
}

#[command]
pub async fn resolve_drift(project_path: PathBuf, request_id: String, source_id: String) -> Result<()> {
    let sources = load_sources(&project_path)?;
    let source = sources
        .sources
        .iter()
        .find(|s| s.id() == source_id)
        .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", source_id)))?;

    let spec = get_spec_for_source(source).await?;
    let ops = openapi_import::parse_operations(&spec)?;

    // Find all request files and locate the one matching request_id.
    let request_files = crate::services::drift_detection::collect_request_files(&project_path)?;
    let file_path = request_files
        .iter()
        .find(|p| {
            derive_request_id(&project_path, p)
                .map(|id| id == request_id)
                .unwrap_or(false)
        })
        .ok_or_else(|| FlupiError::Custom(format!("Request '{}' not found on disk", request_id)))?;

    let mut request: crate::models::request::Request = file_io::read_json(file_path)?;
    let template_ref = request
        .template_ref
        .as_ref()
        .ok_or_else(|| FlupiError::Custom("Request has no templateRef".to_string()))?;

    // Try exact operationId match first, then fall back to method+path similarity
    // to handle renames where the operationId was re-derived from the new path.
    let (current_op, op_json) = ops
        .iter()
        .find(|(op, _)| op.operation_id == template_ref.operation_id)
        .or_else(|| find_candidate_operation(&request.method, &request.path, &ops))
        .ok_or_else(|| FlupiError::Custom(format!(
            "Operation '{}' not found in current spec and no rename candidate found",
            template_ref.operation_id
        )))?;

    // Compute all new values before any mutation (avoids borrow conflict).
    let new_path = current_op.path.clone();
    let new_operation_id = current_op.operation_id.clone();
    let new_schema_hash = openapi_import::compute_operation_hash(op_json);
    let (new_request_schema, new_response_schema) = openapi_import::extract_schemas(op_json, &spec);

    // Now apply: update path and templateRef atomically (including operationId if renamed).
    request.path = new_path;
    if let Some(tr) = request.template_ref.as_mut() {
        tr.operation_id = new_operation_id;
        tr.schema_hash = new_schema_hash;
        tr.request_schema = new_request_schema;
        tr.response_schema = new_response_schema;
    }

    file_io::write_json(file_path, &request)
}

#[command]
pub async fn get_drift_details(project_path: PathBuf, request_id: String) -> Result<DriftDetails> {
    let request_files = crate::services::drift_detection::collect_request_files(&project_path)?;
    let file_path = request_files
        .iter()
        .find(|p| {
            derive_request_id(&project_path, p)
                .map(|id| id == request_id)
                .unwrap_or(false)
        })
        .ok_or_else(|| FlupiError::Custom(format!("Request '{}' not found on disk", request_id)))?;

    let request: crate::models::request::Request = file_io::read_json(file_path)?;
    let template_ref = request
        .template_ref
        .as_ref()
        .ok_or_else(|| FlupiError::Custom("Request has no templateRef".to_string()))?;

    let sources = load_sources(&project_path)?;
    let source = sources
        .sources
        .iter()
        .find(|s| s.id() == template_ref.source_id)
        .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", template_ref.source_id)))?;

    let spec = get_spec_for_source(source).await?;
    let ops = openapi_import::parse_operations(&spec)?;

    let found = ops.iter().find(|(op, _)| op.operation_id == template_ref.operation_id);

    match found {
        None => {
            // Exact operationId not found — check if it was renamed (same method, similar path).
            let candidate = find_candidate_operation(&request.method, &request.path, &ops);
            match candidate {
                Some((cand_op, cand_json)) => {
                    // Likely a rename: path (and derived operationId) both changed.
                    let current_hash = openapi_import::compute_operation_hash(cand_json);
                    let (new_req, new_res) = openapi_import::extract_schemas(cand_json, &spec);
                    let schema_changed = current_hash != template_ref.schema_hash
                        || new_req != template_ref.request_schema
                        || new_res != template_ref.response_schema;
                    Ok(DriftDetails {
                        source_id: template_ref.source_id.clone(),
                        operation_id: template_ref.operation_id.clone(),
                        stored_path: request.path.clone(),
                        current_path: Some(cand_op.path.clone()),
                        current_operation_id: Some(cand_op.operation_id.clone()),
                        path_changed: true,
                        schema_changed,
                        operation_removed: false,
                    })
                }
                None => Ok(DriftDetails {
                    source_id: template_ref.source_id.clone(),
                    operation_id: template_ref.operation_id.clone(),
                    stored_path: request.path,
                    current_path: None,
                    current_operation_id: None,
                    path_changed: false,
                    schema_changed: false,
                    operation_removed: true,
                }),
            }
        }
        Some((current_op, op_json)) => {
            let current_hash = openapi_import::compute_operation_hash(op_json);
            let (new_req, new_res) = openapi_import::extract_schemas(op_json, &spec);
            let schema_changed = current_hash != template_ref.schema_hash
                || new_req != template_ref.request_schema
                || new_res != template_ref.response_schema;
            Ok(DriftDetails {
                source_id: template_ref.source_id.clone(),
                operation_id: template_ref.operation_id.clone(),
                stored_path: request.path.clone(),
                current_path: Some(current_op.path.clone()),
                current_operation_id: None,
                path_changed: current_op.path != request.path,
                schema_changed,
                operation_removed: false,
            })
        }
    }
}

#[cfg(test)]
#[path = "tests/openapi.rs"]
mod tests;
