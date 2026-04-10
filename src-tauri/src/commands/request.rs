use crate::error::FlupiError;
use crate::models::request::{derive_request_id, Request};
use crate::services::{file_io, request_path};
use crate::utils::name_to_slug;
use serde::Serialize;
use std::path::PathBuf;
use tauri::command;

/// `Request` enriched with a derived `collection` field for the frontend.
/// The `collection` field is never persisted to disk — it is computed from
/// the request ID at read time so the frontend doesn't have to parse paths.
#[derive(Serialize)]
pub struct RequestResponse {
    #[serde(flatten)]
    pub inner: Request,
    pub collection: Option<String>,
}

#[command]
pub fn get_request(
    project_path: PathBuf,
    request_id: String,
) -> Result<RequestResponse, FlupiError> {
    let path = request_path::resolve_request_path(&project_path, &request_id);
    let inner: Request = file_io::read_json(&path)?;
    let collection = request_path::collection_folder_for(&request_id, &project_path);
    Ok(RequestResponse { inner, collection })
}

#[command]
pub fn save_request(
    project_path: PathBuf,
    request_id: String,
    request: Request,
) -> Result<(), FlupiError> {
    let path = request_path::resolve_request_path(&project_path, &request_id);
    file_io::write_json(&path, &request)
}

#[command]
pub fn create_request(
    project_path: PathBuf,
    collection_folder: Option<String>,
    name: String,
) -> Result<String, FlupiError> {
    let slug = name_to_slug(&name);
    let (path, id) = match &collection_folder {
        Some(folder) => {
            let p = project_path
                .join("collections")
                .join(folder)
                .join("requests")
                .join(format!("{}.json", slug));
            let id = format!("{}/{}", folder, slug);
            (p, id)
        }
        None => {
            let p = project_path.join("requests").join(format!("{}.json", slug));
            (p, slug.clone())
        }
    };

    let request = Request {
        name,
        method: "GET".to_string(),
        path: "/".to_string(),
        auth: None,
        headers: indexmap::IndexMap::new(),
        path_params: indexmap::IndexMap::new(),
        body: None,
        template_ref: None,
        disabled_headers: vec![],
        disabled_collection_headers: vec![],
        extractions: vec![],
        pre_request_actions: vec![],
    };
    file_io::write_json(&path, &request)?;
    Ok(id)
}

#[command]
pub fn delete_request(project_path: PathBuf, request_id: String) -> Result<(), FlupiError> {
    let path = request_path::resolve_request_path(&project_path, &request_id);
    file_io::delete_file(&path)
}

#[command]
pub fn rename_request(
    project_path: PathBuf,
    request_id: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let old_path = request_path::resolve_request_path(&project_path, &request_id);
    let new_slug = name_to_slug(&new_name);

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;
    let new_path = parent.join(format!("{}.json", new_slug));

    // Read, update name, write to new path, remove old
    let mut request: Request = file_io::read_json(&old_path)?;
    request.name = new_name;
    file_io::write_json(&new_path, &request)?;
    if old_path != new_path {
        file_io::delete_file(&old_path)?;
    }

    // Derive new ID
    let old_id = request_id;
    let new_id = derive_request_id(&project_path, &new_path)?;
    crate::services::referential_integrity::update_references(&project_path, &old_id, &new_id)?;
    Ok(new_id)
}

#[command]
pub fn move_request(
    project_path: PathBuf,
    request_id: String,
    target_collection_folder: Option<String>,
) -> Result<String, FlupiError> {
    let old_path = request_path::resolve_request_path(&project_path, &request_id);
    let file_name = old_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FlupiError::Custom("Cannot determine file name".to_string()))?
        .to_string();

    let new_path = match &target_collection_folder {
        Some(folder) => project_path
            .join("collections")
            .join(folder)
            .join("requests")
            .join(&file_name),
        None => project_path.join("requests").join(&file_name),
    };

    let request: Request = file_io::read_json(&old_path)?;
    file_io::write_json(&new_path, &request)?;
    file_io::delete_file(&old_path)?;

    let old_id = request_id;
    let new_id = derive_request_id(&project_path, &new_path)?;
    crate::services::referential_integrity::update_references(&project_path, &old_id, &new_id)?;
    Ok(new_id)
}

#[command]
pub fn duplicate_request(project_path: PathBuf, request_id: String) -> Result<String, FlupiError> {
    let old_path = request_path::resolve_request_path(&project_path, &request_id);
    let mut request: Request = file_io::read_json(&old_path)?;

    let stem = old_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FlupiError::Custom("Cannot determine file stem".to_string()))?
        .to_string();

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;

    // Find a non-colliding copy name: try "{stem}-copy", then "{stem}-copy-2" … up to "-copy-10"
    let new_path = {
        let candidate = parent.join(format!("{}-copy.json", stem));
        if !candidate.exists() {
            candidate
        } else {
            let mut found: Option<PathBuf> = None;
            for n in 2..=10 {
                let c = parent.join(format!("{}-copy-{}.json", stem, n));
                if !c.exists() {
                    found = Some(c);
                    break;
                }
            }
            found.ok_or_else(|| FlupiError::Custom("duplicate already exists".to_string()))?
        }
    };

    // Derive a human-readable copy name to match the chosen file stem
    let fallback_stem = format!("{}-copy", stem);
    let copy_stem = new_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&fallback_stem);
    // Replace slug hyphens back to spaces for a friendlier display name,
    // preserving the original name logic: first copy → "{name} copy", subsequent → "{name} copy N"
    let base_name = request.name.clone();
    request.name = if copy_stem.ends_with("-copy") {
        format!("{} copy", base_name)
    } else {
        // extract the numeric suffix from "{stem}-copy-N"
        let suffix = copy_stem.rsplit('-').next().unwrap_or("2");
        format!("{} copy {}", base_name, suffix)
    };

    file_io::write_json(&new_path, &request)?;

    let new_id = derive_request_id(&project_path, &new_path)?;
    Ok(new_id)
}

#[tauri::command]
pub async fn get_request_references(
    project_path: String,
    request_id: String,
) -> Result<Vec<String>, FlupiError> {
    let project_path = std::path::Path::new(&project_path);
    let refs = crate::services::referential_integrity::find_references(project_path, &request_id)?;
    Ok(refs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
