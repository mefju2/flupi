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
#[path = "tests/project.rs"]
mod tests;
