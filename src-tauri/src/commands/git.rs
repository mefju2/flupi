use std::path::PathBuf;

use tauri::command;

use crate::error::FlupiError;
use crate::services::git;
use crate::services::git::{GitFileDiff, GitStatus};

#[command]
pub fn get_git_status(project_path: String) -> Result<GitStatus, FlupiError> {
    git::get_status(&PathBuf::from(project_path))
}

#[command]
pub fn git_fetch(project_path: String) -> Result<(), FlupiError> {
    git::fetch(&PathBuf::from(project_path))
}

#[command]
pub fn git_pull(project_path: String) -> Result<(), FlupiError> {
    git::pull(&PathBuf::from(project_path))
}

#[command]
pub fn git_file_diff(project_path: String, file_path: String) -> Result<GitFileDiff, FlupiError> {
    git::get_file_diff(&PathBuf::from(project_path), &file_path)
}
