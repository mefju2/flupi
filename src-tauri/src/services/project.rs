use std::path::Path;
use crate::error::Result;

const PROJECT_DIRS: &[&str] = &["environments", "collections", "requests", "scenarios"];
const GITIGNORE_CONTENT: &str = "*.secrets.json\n";

#[derive(Debug, PartialEq)]
pub enum ProjectState {
    Valid,
    Partial,
    Empty,
    NotFound,
}

pub fn init_project(path: &Path) -> Result<()> {
    for dir in PROJECT_DIRS {
        std::fs::create_dir_all(path.join(dir))?;
    }

    let gitignore_path = path.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, GITIGNORE_CONTENT)?;
    }

    Ok(())
}

pub fn validate_project(path: &Path) -> ProjectState {
    if !path.exists() {
        return ProjectState::NotFound;
    }

    let existing: Vec<bool> = PROJECT_DIRS
        .iter()
        .map(|d| path.join(d).is_dir())
        .collect();

    if existing.iter().all(|&e| e) {
        ProjectState::Valid
    } else if existing.iter().any(|&e| e) {
        ProjectState::Partial
    } else {
        ProjectState::Empty
    }
}

pub fn ensure_project_structure(path: &Path) -> Result<()> {
    for dir in PROJECT_DIRS {
        let dir_path = path.join(dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
