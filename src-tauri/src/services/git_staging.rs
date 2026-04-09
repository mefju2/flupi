use std::path::{Component, Path};
use std::process::Command;

use crate::error::{FlupiError, Result};

const GIT_NOT_FOUND: &str =
    "git binary not found. Please install git and ensure it is on your PATH.";

fn validate_path(file_path: &str) -> Result<()> {
    let fp = Path::new(file_path);
    if fp.is_absolute() || fp.components().any(|c| c == Component::ParentDir) {
        return Err(FlupiError::Custom("Invalid file path".to_string()));
    }
    Ok(())
}

pub fn stage_file(path: &Path, file_path: &str) -> Result<()> {
    validate_path(file_path)?;
    let output = Command::new("git")
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
    let output = Command::new("git")
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
    let output = Command::new("git")
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
    let output = Command::new("git")
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

    let output = Command::new("git")
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

pub fn push(path: &Path) -> Result<()> {
    let output = Command::new("git")
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
