use std::path::{Path, PathBuf};
use tauri::command;
use crate::error::FlupiError;
use crate::models::request::{Request, derive_request_id};
use crate::services::file_io;
use crate::utils::name_to_slug;

/// Resolves a request ID to an absolute file path.
///
/// Disambiguation strategy: if the ID contains a `/`, the first path segment is
/// checked against `collections/{first_segment}/` on disk. If that directory
/// exists, the request is treated as a collection request and maps to
/// `collections/{first_segment}/requests/{rest}.json`. Otherwise (the collection
/// directory does not exist), the entire ID is treated as a root request under
/// `requests/{id}.json`. This assumes collection folder names never collide with
/// root request sub-paths.
fn resolve_request_path(project_path: &Path, request_id: &str) -> PathBuf {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        let rest = parts[1];
        let collection_dir = project_path.join("collections").join(first);
        if collection_dir.exists() {
            // It's a collection request
            let rest_path = rest.replace('/', std::path::MAIN_SEPARATOR_STR);
            return project_path
                .join("collections")
                .join(first)
                .join("requests")
                .join(format!("{}.json", rest_path));
        }
    }
    // Root request
    let rest_path = request_id.replace('/', std::path::MAIN_SEPARATOR_STR);
    project_path.join("requests").join(format!("{}.json", rest_path))
}

#[command]
pub fn get_request(project_path: PathBuf, request_id: String) -> Result<Request, FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
    file_io::read_json(&path)
}

#[command]
pub fn save_request(
    project_path: PathBuf,
    request_id: String,
    request: Request,
) -> Result<(), FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
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
    };
    file_io::write_json(&path, &request)?;
    Ok(id)
}

#[command]
pub fn delete_request(project_path: PathBuf, request_id: String) -> Result<(), FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
    file_io::delete_file(&path)
}

#[command]
pub fn rename_request(
    project_path: PathBuf,
    request_id: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let old_path = resolve_request_path(&project_path, &request_id);
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
    let old_path = resolve_request_path(&project_path, &request_id);
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
pub fn duplicate_request(
    project_path: PathBuf,
    request_id: String,
) -> Result<String, FlupiError> {
    let old_path = resolve_request_path(&project_path, &request_id);
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
            found.ok_or_else(|| {
                FlupiError::Custom("duplicate already exists".to_string())
            })?
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
        let suffix = copy_stem
            .rsplit('-')
            .next()
            .unwrap_or("2");
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
    Ok(refs.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
