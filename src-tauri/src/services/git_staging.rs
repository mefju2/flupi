use std::path::{Component, Path};

use crate::error::{FlupiError, Result};
use crate::services::GIT_NOT_FOUND;
use crate::utils::git_command;

fn validate_path(file_path: &str) -> Result<()> {
    let fp = Path::new(file_path);
    if fp.is_absolute() || fp.components().any(|c| c == Component::ParentDir) {
        return Err(FlupiError::Custom("Invalid file path".to_string()));
    }
    Ok(())
}

pub fn stage_file(path: &Path, file_path: &str) -> Result<()> {
    validate_path(file_path)?;
    let output = git_command()
        .args(["add", file_path])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git add failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn unstage_file(path: &Path, file_path: &str) -> Result<()> {
    validate_path(file_path)?;
    let output = git_command()
        .args(["restore", "--staged", file_path])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git restore --staged failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn stage_all(path: &Path) -> Result<()> {
    let output = git_command()
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git add -A failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn unstage_all(path: &Path) -> Result<()> {
    let output = git_command()
        .args(["restore", "--staged", "."])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git restore --staged failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn commit(path: &Path, message: &str) -> Result<()> {
    let message = message.trim();
    if message.is_empty() {
        return Err(FlupiError::Custom(
            "Commit message cannot be empty".to_string(),
        ));
    }

    let output = git_command()
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git commit failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn discard_file(project_path: &Path, file_path: &str) -> Result<()> {
    validate_path(file_path)?;
    let output = git_command()
        .args(["restore", file_path])
        .current_dir(project_path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git restore failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

pub fn delete_file(project_path: &Path, file_path: &str) -> Result<()> {
    validate_path(file_path)?;
    let full_path = project_path.join(file_path);
    std::fs::remove_file(&full_path)
        .map_err(|e| FlupiError::Custom(format!("Failed to delete file: {}", e)))
}

pub fn push(path: &Path) -> Result<()> {
    let output = git_command()
        .arg("push")
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git push failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/git_staging.rs"]
mod tests;
