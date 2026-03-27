use std::path::PathBuf;
use tauri::command;
use crate::error::{FlupiError, Result};
use crate::models::openapi::{ImportableOperation, OpenApiSource, OpenApiSources};
use crate::services::{file_io, openapi_import};

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

#[command]
pub fn add_openapi_source(project_path: PathBuf, source: OpenApiSource) -> Result<()> {
    let mut sources = load_sources(&project_path)?;
    sources.sources.push(source);
    save_sources(&project_path, &sources)
}

#[command]
pub fn remove_openapi_source(project_path: PathBuf, source_id: String) -> Result<()> {
    let mut sources = load_sources(&project_path)?;
    sources.sources.retain(|s| s.id() != source_id);
    save_sources(&project_path, &sources)
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

    openapi_import::import_operations(&project_path, &source_id, &filtered, &collection_folder)
}

#[command]
pub async fn refresh_source(project_path: PathBuf, source_id: String) -> Result<Vec<String>> {
    let mut sources = load_sources(&project_path)?;
    let source = sources
        .sources
        .iter()
        .find(|s| s.id() == source_id)
        .ok_or_else(|| FlupiError::Custom(format!("Source '{}' not found", source_id)))?
        .clone();

    let spec = get_spec_for_source(&source).await?;
    let ops = openapi_import::parse_operations(&spec)?;

    let now = chrono::Utc::now().to_rfc3339();
    let new_hash = crate::services::openapi_import::compute_operation_hash(&spec);

    // Update last_fetched_at and last_hash on source
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

    let drifted = crate::services::drift_detection::detect_drift(&project_path, &source_id, &ops)?;
    Ok(drifted)
}

#[cfg(test)]
#[path = "tests/openapi.rs"]
mod tests;
