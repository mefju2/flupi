use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::error::{FlupiError, Result};

const GIT_NOT_FOUND: &str =
    "git binary not found. Please install git and ensure it is on your PATH.";

#[derive(Debug, Serialize, Clone)]
pub struct BranchInfo {
    pub name: String,
    #[serde(rename = "isCurrent")]
    pub is_current: bool,
    #[serde(rename = "isRemote")]
    pub is_remote: bool,
}

pub(crate) fn parse_branch_list(output: &str) -> Vec<BranchInfo> {
    output
        .lines()
        .filter_map(|line| {
            let is_current = line.starts_with("* ");
            let name = line
                .trim_start_matches("* ")
                .trim_start_matches("  ")
                .trim();
            if name.is_empty() || name.starts_with("(HEAD detached") {
                return None;
            }
            let is_remote = name.starts_with("remotes/");
            let clean_name = if is_remote {
                name.strip_prefix("remotes/").unwrap_or(name)
            } else {
                name
            };
            // Skip "origin/HEAD" auto-pointers and arrow entries
            if clean_name.ends_with("/HEAD") || clean_name.contains("->") {
                return None;
            }
            Some(BranchInfo {
                name: clean_name.to_string(),
                is_current,
                is_remote,
            })
        })
        .collect()
}

pub fn list_branches(path: &Path) -> Result<Vec<BranchInfo>> {
    let output = Command::new("git")
        .args(["branch", "-a"])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git branch failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_branch_list(&stdout))
}

pub fn checkout_branch(path: &Path, branch: &str) -> Result<()> {
    if branch.contains("..") || branch.starts_with('-') {
        return Err(FlupiError::Custom("Invalid branch name".to_string()));
    }

    let output = Command::new("git")
        .args(["checkout", branch])
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git checkout failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/git_branch.rs"]
mod tests;
