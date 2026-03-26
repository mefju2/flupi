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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_collection() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();

        let slug = create_collection(root.clone(), "Auth Service".to_string()).unwrap();
        assert_eq!(slug, "auth-service");
        assert!(root.join("collections/auth-service/collection.json").exists());
        assert!(root.join("collections/auth-service/requests").exists());

        let col: Collection =
            file_io::read_json(&root.join("collections/auth-service/collection.json")).unwrap();
        assert_eq!(col.name, "Auth Service");
    }

    #[test]
    fn test_save_collection() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("collections/my-service")).unwrap();

        let collection = Collection {
            name: "My Service".to_string(),
            base_url: Some("https://example.com".to_string()),
            auth: None,
            headers: indexmap::IndexMap::new(),
        };
        save_collection(root.clone(), "my-service".to_string(), collection).unwrap();

        let col: Collection =
            file_io::read_json(&root.join("collections/my-service/collection.json")).unwrap();
        assert_eq!(col.name, "My Service");
        assert_eq!(col.base_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_delete_collection() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("collections/to-delete/requests")).unwrap();
        std::fs::write(
            root.join("collections/to-delete/collection.json"),
            r#"{"name":"To Delete"}"#,
        )
        .unwrap();

        delete_collection(root.clone(), "to-delete".to_string()).unwrap();
        assert!(!root.join("collections/to-delete").exists());
    }

    #[test]
    fn test_delete_collection_not_found() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        let result = delete_collection(root, "nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_collection() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("collections/old-name/requests")).unwrap();
        std::fs::write(
            root.join("collections/old-name/collection.json"),
            r#"{"name":"Old Name"}"#,
        )
        .unwrap();

        let new_slug = rename_collection(root.clone(), "old-name".to_string(), "New Name".to_string()).unwrap();
        assert_eq!(new_slug, "new-name");
        assert!(!root.join("collections/old-name").exists());
        assert!(root.join("collections/new-name/collection.json").exists());

        let col: Collection =
            file_io::read_json(&root.join("collections/new-name/collection.json")).unwrap();
        assert_eq!(col.name, "New Name");
    }
}
