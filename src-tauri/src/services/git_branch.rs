use std::path::Path;

use serde::Serialize;

use crate::utils::git_command;

use crate::error::{FlupiError, Result};
use crate::services::GIT_NOT_FOUND;

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
    let output = git_command()
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

pub fn checkout_branch(path: &Path, branch: &str, is_remote: bool) -> Result<()> {
    if branch.contains("..")
        || branch.starts_with('-')
        || branch.chars().any(|c| matches!(c, '\0' | '\n' | '\r'))
    {
        return Err(FlupiError::Custom("Invalid branch name".to_string()));
    }

    if is_remote {
        // branch is e.g. "origin/feature" — derive the local tracking name
        let local = branch.splitn(2, '/').nth(1).unwrap_or(branch);

        // Try to create a local tracking branch
        let out = git_command()
            .args(["checkout", "-b", local, "--track", branch])
            .current_dir(path)
            .output()
            .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

        if out.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&out.stderr);
        // Local branch already exists — just switch to it
        if stderr.contains("already exists") {
            let out2 = git_command()
                .args(["checkout", local])
                .current_dir(path)
                .output()
                .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;
            if !out2.status.success() {
                let e = String::from_utf8_lossy(&out2.stderr);
                return Err(FlupiError::Custom(format!(
                    "git checkout failed: {}",
                    e.trim()
                )));
            }
            return Ok(());
        }

        return Err(FlupiError::Custom(format!(
            "git checkout failed: {}",
            stderr.trim()
        )));
    }

    let output = git_command()
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
