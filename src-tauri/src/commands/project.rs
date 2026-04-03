use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::services::project::{self, ProjectState};

#[command]
pub fn create_project(path: PathBuf) -> Result<(), FlupiError> {
    project::init_project(&path)
}

#[command]
pub fn open_project(path: PathBuf) -> Result<String, FlupiError> {
    match project::validate_project(&path) {
        ProjectState::Valid => Ok("valid".to_string()),
        ProjectState::Partial => {
            project::ensure_project_structure(&path)?;
            Ok("partial_fixed".to_string())
        }
        ProjectState::Empty => {
            project::ensure_project_structure(&path)?;
            Ok("empty_fixed".to_string())
        }
        ProjectState::NotFound => Err(FlupiError::Custom(
            "Project folder not found".to_string(),
        )),
    }
}
