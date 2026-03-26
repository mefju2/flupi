use super::*;
use tempfile::TempDir;

#[test]
fn test_read_json_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.json");
    std::fs::write(&path, r#"{"name": "test"}"#).unwrap();

    let result: serde_json::Value = read_json(&path).unwrap();
    assert_eq!(result["name"], "test");
}

#[test]
fn test_read_json_file_not_found() {
    let result: Result<serde_json::Value> = read_json(Path::new("/nonexistent.json"));
    assert!(result.is_err());
}

#[test]
fn test_write_json_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("out.json");
    let data = serde_json::json!({"key": "value"});

    write_json(&path, &data).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["key"], "value");
}

#[test]
fn test_write_json_creates_parent_dirs() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sub/dir/file.json");
    let data = serde_json::json!({"nested": true});

    write_json(&path, &data).unwrap();
    assert!(path.exists());
}
