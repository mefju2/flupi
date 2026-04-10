use std::path::PathBuf;

use tauri::command;

use crate::error::FlupiError;
use crate::services::git;
use crate::services::git::{GitFileDiff, GitStatus};
use crate::services::git_branch::{self, BranchInfo};
use crate::services::git_staging;

#[command]
pub fn get_git_status(project_path: String) -> Result<GitStatus, FlupiError> {
    git::get_status(&PathBuf::from(project_path))
}

#[command]
pub async fn git_fetch(project_path: String) -> Result<(), FlupiError> {
    let path = PathBuf::from(project_path);
    tokio::task::spawn_blocking(move || git::fetch(&path))
        .await
        .map_err(|e| FlupiError::Custom(e.to_string()))?
}

#[command]
pub async fn git_pull(project_path: String) -> Result<(), FlupiError> {
    let path = PathBuf::from(project_path);
    tokio::task::spawn_blocking(move || git::pull(&path))
        .await
        .map_err(|e| FlupiError::Custom(e.to_string()))?
}

#[command]
pub fn git_file_diff(project_path: String, file_path: String) -> Result<GitFileDiff, FlupiError> {
    git::get_file_diff(&PathBuf::from(project_path), &file_path)
}

#[command]
pub fn git_stage_file(project_path: String, file_path: String) -> Result<(), FlupiError> {
    git_staging::stage_file(&PathBuf::from(project_path), &file_path)
}

#[command]
pub fn git_unstage_file(project_path: String, file_path: String) -> Result<(), FlupiError> {
    git_staging::unstage_file(&PathBuf::from(project_path), &file_path)
}

#[command]
pub fn git_stage_all(project_path: String) -> Result<(), FlupiError> {
    git_staging::stage_all(&PathBuf::from(project_path))
}

#[command]
pub fn git_unstage_all(project_path: String) -> Result<(), FlupiError> {
    git_staging::unstage_all(&PathBuf::from(project_path))
}

#[command]
pub fn git_commit(project_path: String, message: String) -> Result<(), FlupiError> {
    git_staging::commit(&PathBuf::from(project_path), &message)
}

#[command]
pub async fn git_push(project_path: String) -> Result<(), FlupiError> {
    let path = PathBuf::from(project_path);
    tokio::task::spawn_blocking(move || git_staging::push(&path))
        .await
        .map_err(|e| FlupiError::Custom(e.to_string()))?
}

#[command]
pub fn git_list_branches(project_path: String) -> Result<Vec<BranchInfo>, FlupiError> {
    git_branch::list_branches(&PathBuf::from(project_path))
}

#[command]
pub fn git_checkout_branch(project_path: String, branch: String, is_remote: bool) -> Result<(), FlupiError> {
    git_branch::checkout_branch(&PathBuf::from(project_path), &branch, is_remote)
}
