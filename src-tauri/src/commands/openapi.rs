use std::path::PathBuf;
use serde::Serialize;
use tauri::command;
use crate::error::{FlupiError, Result};
use crate::models::openapi::{ImportableOperation, OpenApiSource};
use crate::models::request::derive_request_id;
use crate::services::{file_io, openapi_import, openapi_sources};
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
    /// For an exact-operationId match where only the path changed: the new path.
    /// None when the operationId no longer exists (user picks from `candidates`).
    #[serde(rename = "currentPath")]
    pub current_path: Option<String>,
    #[serde(rename = "pathChanged")]
    pub path_changed: bool,
    #[serde(rename = "schemaChanged")]
    pub schema_changed: bool,
    /// True only when the operationId is gone and no plausible rename candidates exist.
    #[serde(rename = "operationRemoved")]
    pub operation_removed: bool,
    /// Rename candidates sorted by similarity score (descending). Non-empty only when
    /// the stored operationId no longer exists in the spec. Empty means exact-match or
    /// truly removed.
    pub candidates: Vec<PathCandidate>,
    /// Schema snapshots — only populated when `schema_changed` is true (exact-match case).
    #[serde(rename = "storedRequestSchema", skip_serializing_if = "Option::is_none")]
    pub stored_request_schema: Option<serde_json::Value>,
    #[serde(rename = "storedResponseSchema", skip_serializing_if = "Option::is_none")]
    pub stored_response_schema: Option<serde_json::Value>,
    #[serde(rename = "newRequestSchema", skip_serializing_if = "Option::is_none")]
    pub new_request_schema: Option<serde_json::Value>,
    #[serde(rename = "newResponseSchema", skip_serializing_if = "Option::is_none")]
    pub new_response_schema: Option<serde_json::Value>,
}

/// A candidate operation the user can remap this drifted request to.
#[derive(Debug, Serialize, Clone)]
pub struct PathCandidate {
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub path: String,
    pub method: String,
    pub summary: Option<String>,
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Returns plausible rename candidates sorted by normalised Dice-prefix score (descending).
/// Threshold: 0.20. Cap: 8 results. Excludes `excluded_ids` (operations already mapped
/// by other requests in the same source, so they are not a rename target for this one).
fn find_candidate_operations(
    stored_method: &str,
    stored_path: &str,
    operations: &[(ImportableOperation, serde_json::Value)],
    excluded_ids: &std::collections::HashSet<String>,
) -> Vec<PathCandidate> {
    const MIN_SCORE: f64 = 0.20;
    const MAX_CANDIDATES: usize = 8;

    let method_up = stored_method.to_uppercase();
    let a_len = stored_path.chars().count();

    let mut scored: Vec<(u32, PathCandidate)> = operations
        .iter()
        .filter(|(op, _)| {
            op.method.to_uppercase() == method_up
                && !excluded_ids.contains(&op.operation_id)
        })
        .filter_map(|(op, _)| {
            let b_len = op.path.chars().count();
            let denom = a_len + b_len;
            let score = if denom == 0 {
                0.0
            } else {
                2.0 * common_prefix_len(stored_path, &op.path) as f64 / denom as f64
            };
            if score >= MIN_SCORE {
                Some(((score * 1_000_000.0) as u32, PathCandidate {
                    operation_id: op.operation_id.clone(),
                    path: op.path.clone(),
                    method: op.method.clone(),
                    summary: op.summary.clone(),
                }))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(MAX_CANDIDATES);
    scored.into_iter().map(|(_, c)| c).collect()
}

/// Collects operationIds already claimed by other requests in this project that belong
/// to `source_id`, excluding `exclude_request_id` itself (the drifted request we're
/// computing candidates for).
fn collect_claimed_operation_ids(
    project_path: &std::path::Path,
    source_id: &str,
    exclude_request_id: &str,
) -> std::collections::HashSet<String> {
    let mut claimed = std::collections::HashSet::new();
    let files = match crate::services::drift_detection::collect_request_files(project_path) {
        Ok(f) => f,
        Err(_) => return claimed,
    };
    for file_path in &files {
        let rid = match crate::models::request::derive_request_id(project_path, file_path) {
            Ok(id) => id,
            Err(_) => continue,
        };
        if rid == exclude_request_id {
            continue;
        }
        let req: crate::models::request::Request = match file_io::read_json(file_path) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Some(tr) = &req.template_ref {
            if tr.source_id == source_id {
                claimed.insert(tr.operation_id.clone());
            }
        }
    }
    claimed
}

#[command]
pub async fn add_openapi_source(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    source: OpenApiSource,
) -> Result<()> {
    let _guard = state.sources_lock.lock().await;
    openapi_sources::add(&project_path, source)
}

#[command]
pub async fn remove_openapi_source(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    source_id: String,
) -> Result<()> {
    let _guard = state.sources_lock.lock().await;
    openapi_sources::remove(&project_path, &source_id)
}

#[command]
pub fn list_openapi_sources(project_path: PathBuf) -> Result<Vec<OpenApiSource>> {
    let sources = openapi_sources::load(&project_path)?;
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
    let sources = openapi_sources::load(&project_path)?;
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
    let sources = openapi_sources::load(&project_path)?;
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
        let sources = openapi_sources::load(&project_path)?;
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
        let mut sources = openapi_sources::load(&project_path)?;
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
        openapi_sources::save(&project_path, &sources)?;
    } // lock released here

    let drifted = crate::services::drift_detection::detect_drift(&project_path, &source_id, &ops, &spec)?;
    Ok(drifted)
}

#[command]
pub async fn resolve_drift(
    project_path: PathBuf,
    request_id: String,
    source_id: String,
    // The operationId the user chose from the candidates list.
    // Required when the stored operationId no longer exists (rename case).
    // Pass None for exact-match path-change (stored operationId is still valid).
    chosen_operation_id: Option<String>,
) -> Result<()> {
    let sources = openapi_sources::load(&project_path)?;
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

    // When the user chose a rename candidate, look up by that ID directly.
    // Otherwise fall back to the stored operationId (exact-match path-change case).
    let lookup_id = chosen_operation_id
        .as_deref()
        .unwrap_or(&template_ref.operation_id);
    let (current_op, op_json) = ops
        .iter()
        .find(|(op, _)| op.operation_id == lookup_id)
        .ok_or_else(|| FlupiError::Custom(format!(
            "Operation '{}' not found in current spec",
            lookup_id
        )))?;

    // Compute all new values before any mutation (avoids borrow conflict).
    let new_path = current_op.path.clone();
    let new_operation_id = current_op.operation_id.clone();
    let new_schema_hash = openapi_import::compute_sha256_hash(op_json);
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

    let sources = openapi_sources::load(&project_path)?;
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
            // Exact operationId not found — build a ranked list of rename candidates,
            // filtered to exclude operations already claimed by other requests.
            let claimed = collect_claimed_operation_ids(
                &project_path, &template_ref.source_id, &request_id,
            );
            let candidates = find_candidate_operations(
                &request.method, &request.path, &ops, &claimed,
            );
            Ok(DriftDetails {
                source_id: template_ref.source_id.clone(),
                operation_id: template_ref.operation_id.clone(),
                stored_path: request.path,
                current_path: None,
                path_changed: !candidates.is_empty(),
                schema_changed: false,
                operation_removed: candidates.is_empty(),
                candidates,
                stored_request_schema: None,
                stored_response_schema: None,
                new_request_schema: None,
                new_response_schema: None,
            })
        }
        Some((current_op, op_json)) => {
            let current_hash = openapi_import::compute_sha256_hash(op_json);
            let (new_req, new_res) = openapi_import::extract_schemas(op_json, &spec);
            let schema_changed = current_hash != template_ref.schema_hash
                || new_req != template_ref.request_schema
                || new_res != template_ref.response_schema;
            let (stored_req, stored_res, new_req_out, new_res_out) = if schema_changed {
                (
                    Some(template_ref.request_schema.clone()),
                    Some(template_ref.response_schema.clone()),
                    Some(new_req),
                    Some(new_res),
                )
            } else {
                (None, None, None, None)
            };
            Ok(DriftDetails {
                source_id: template_ref.source_id.clone(),
                operation_id: template_ref.operation_id.clone(),
                stored_path: request.path.clone(),
                current_path: Some(current_op.path.clone()),
                path_changed: current_op.path != request.path,
                schema_changed,
                operation_removed: false,
                candidates: vec![],
                stored_request_schema: stored_req,
                stored_response_schema: stored_res,
                new_request_schema: new_req_out,
                new_response_schema: new_res_out,
            })
        }
    }
}

#[cfg(test)]
#[path = "tests/openapi.rs"]
mod tests;
