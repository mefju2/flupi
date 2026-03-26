use super::*;
use tempfile::TempDir;

#[test]
fn test_init_project_creates_structure() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path()).unwrap();

    assert!(dir.path().join(".gitignore").exists());
    assert!(dir.path().join("environments").is_dir());
    assert!(dir.path().join("collections").is_dir());
    assert!(dir.path().join("requests").is_dir());
    assert!(dir.path().join("scenarios").is_dir());
}

#[test]
fn test_gitignore_contains_secrets_pattern() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path()).unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains("*.secrets.json"));
}

#[test]
fn test_gitignore_not_overwritten_if_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "custom\n").unwrap();
    init_project(dir.path()).unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(gitignore, "custom\n");
}

#[test]
fn test_validate_project_full_structure() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path()).unwrap();

    let result = validate_project(dir.path());
    assert_eq!(result, ProjectState::Valid);
}

#[test]
fn test_validate_project_empty_dir() {
    let dir = TempDir::new().unwrap();
    let result = validate_project(dir.path());
    assert_eq!(result, ProjectState::Empty);
}

#[test]
fn test_validate_project_partial() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("environments")).unwrap();
    let result = validate_project(dir.path());
    assert_eq!(result, ProjectState::Partial);
}
