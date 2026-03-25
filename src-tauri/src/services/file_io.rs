use std::path::Path;
use serde::{de::DeserializeOwned, Serialize};
use crate::error::Result;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn delete_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn list_json_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if dir.exists() {
        collect_json_files(dir, &mut files)?;
    }
    Ok(files)
}

fn collect_json_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
