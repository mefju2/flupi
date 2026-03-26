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
