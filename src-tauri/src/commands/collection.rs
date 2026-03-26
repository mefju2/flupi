use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::models::collection::Collection;
use crate::services::file_io;
use crate::utils::name_to_slug;

#[command]
pub fn create_collection(project_path: PathBuf, name: String) -> Result<String, FlupiError> {
    let slug = name_to_slug(&name);
    let collection_dir = project_path.join("collections").join(&slug);
    let collection_json = collection_dir.join("collection.json");
    let requests_dir = collection_dir.join("requests");

    let collection = Collection {
        name,
        base_url: None,
        auth: None,
        headers: indexmap::IndexMap::new(),
    };

    file_io::write_json(&collection_json, &collection)?;
    std::fs::create_dir_all(&requests_dir)?;

    Ok(slug)
}

#[command]
pub fn save_collection(
    project_path: PathBuf,
    folder_name: String,
    collection: Collection,
) -> Result<(), FlupiError> {
    let path = project_path
        .join("collections")
        .join(&folder_name)
        .join("collection.json");
    file_io::write_json(&path, &collection)
}

#[command]
pub fn delete_collection(project_path: PathBuf, folder_name: String) -> Result<(), FlupiError> {
    let collection_dir = project_path.join("collections").join(&folder_name);
    if !collection_dir.exists() {
        return Err(FlupiError::Custom(format!(
            "Collection '{}' not found",
            folder_name
        )));
    }
    std::fs::remove_dir_all(&collection_dir)?;
    Ok(())
}

#[command]
pub fn rename_collection(
    project_path: PathBuf,
    folder_name: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let new_slug = name_to_slug(&new_name);
    let old_dir = project_path.join("collections").join(&folder_name);
    let new_dir = project_path.join("collections").join(&new_slug);

    if !old_dir.exists() {
        return Err(FlupiError::Custom(format!(
            "Collection '{}' not found",
            folder_name
        )));
    }

    // Rename the directory
    std::fs::rename(&old_dir, &new_dir)?;

    // Update collection.json name field
    let collection_json = new_dir.join("collection.json");
    if collection_json.exists() {
        let mut collection: Collection = file_io::read_json(&collection_json)?;
        collection.name = new_name;
        file_io::write_json(&collection_json, &collection)?;
    }

    Ok(new_slug)
}

#[cfg(test)]
#[path = "tests/collection.rs"]
mod tests;
