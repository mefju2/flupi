use std::path::{Component, Path};

use serde::Serialize;

use crate::utils::git_command;

use crate::error::{FlupiError, Result};
use crate::services::GIT_NOT_FOUND;

#[derive(Debug, Serialize, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
    #[serde(rename = "isGitRepo")]
    pub is_git_repo: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType {
    Add,
    Remove,
    Same,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiffLine {
    #[serde(rename = "type")]
    pub line_type: DiffLineType,
    pub text: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GitFileDiff {
    pub lines: Vec<DiffLine>,
    #[serde(rename = "isNewFile")]
    pub is_new_file: bool,
}

pub fn get_status(path: &Path) -> Result<GitStatus> {
    let output = git_command()
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(path)
        .output();

    let output = match output {
        Err(_) => return Err(FlupiError::Custom(GIT_NOT_FOUND.to_string())),
        Ok(o) => o,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Ok(GitStatus {
                branch: String::new(),
                upstream: None,
                ahead: 0,
                behind: 0,
                staged: vec![],
                modified: vec![],
                deleted: vec![],
                untracked: vec![],
                is_git_repo: false,
            });
        }
        return Err(FlupiError::Custom(format!(
            "git status failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_porcelain_v2(&stdout)
}

fn parse_porcelain_v2(output: &str) -> Result<GitStatus> {
    let mut branch = String::new();
    let mut upstream: Option<String> = None;
    let mut ahead = 0u32;
    let mut behind = 0u32;
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut untracked = Vec::new();

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("# branch.head ") {
            branch = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("# branch.upstream ") {
            upstream = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("# branch.ab ") {
            // format: "+N -M"
            let parts: Vec<&str> = val.split_whitespace().collect();
            if parts.len() == 2 {
                ahead = parts[0].trim_start_matches('+').parse().unwrap_or(0);
                behind = parts[1].trim_start_matches('-').parse().unwrap_or(0);
            }
        } else if line.starts_with("1 ") {
            // ordinary changed entry: "1 XY sub mH mI mW hH hI path"
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() >= 9 {
                let xy = parts[1];
                let file_path = parts[8].trim().to_string();
                let x = xy.chars().next().unwrap_or('.');
                let y = xy.chars().nth(1).unwrap_or('.');

                // Index (staged) changes: x is not '.'
                if x != '.' {
                    staged.push(file_path.clone());
                }
                // Working tree changes
                if y == 'D' {
                    deleted.push(file_path);
                } else if y == 'M' {
                    modified.push(file_path);
                }
            }
        } else if line.starts_with("2 ") {
            // renamed/copied entry — the new path is staged
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if let Some(path_field) = parts.last() {
                let new_path = path_field
                    .split('\t')
                    .next()
                    .unwrap_or(path_field)
                    .trim()
                    .to_string();
                staged.push(new_path);
            }
        } else if let Some(file_path) = line.strip_prefix("? ") {
            untracked.push(file_path.trim().to_string());
        }
    }

    Ok(GitStatus {
        branch,
        upstream,
        ahead,
        behind,
        staged,
        modified,
        deleted,
        untracked,
        is_git_repo: true,
    })
}

pub fn fetch(path: &Path) -> Result<()> {
    let output = git_command()
        .arg("fetch")
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FlupiError::Custom(format!(
            "git fetch failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

pub fn pull(path: &Path) -> Result<()> {
    let output = git_command()
        .arg("pull")
        .current_dir(path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);
        if combined.contains("CONFLICT") {
            return Err(FlupiError::Custom(
                "CONFLICT: Merge conflicts were detected. Resolve them in your git client before pulling again.".to_string(),
            ));
        }
        return Err(FlupiError::Custom(format!(
            "git pull failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

pub fn get_file_diff(repo_path: &Path, file_path: &str) -> Result<GitFileDiff> {
    // Security: reject absolute paths and any path traversal attempts.
    let fp = Path::new(file_path);
    if fp.is_absolute() || fp.components().any(|c| c == Component::ParentDir) {
        return Err(FlupiError::Custom("Invalid file path".to_string()));
    }

    let old_output = git_command()
        .args(["show", &format!("HEAD:{}", file_path)])
        .current_dir(repo_path)
        .output()
        .map_err(|_| FlupiError::Custom(GIT_NOT_FOUND.to_string()))?;

    let old_content = if old_output.status.success() {
        String::from_utf8(old_output.stdout)
            .unwrap_or_else(|_| "<binary file — diff not available>".to_string())
    } else {
        // Untracked or new file — no HEAD version exists
        String::new()
    };

    let is_new_file = old_content.is_empty();
    let new_content = std::fs::read_to_string(repo_path.join(file_path)).unwrap_or_default();

    let diff = similar::TextDiff::from_lines(&old_content, &new_content);
    let lines = diff
        .iter_all_changes()
        .map(|change| DiffLine {
            line_type: match change.tag() {
                similar::ChangeTag::Insert => DiffLineType::Add,
                similar::ChangeTag::Delete => DiffLineType::Remove,
                similar::ChangeTag::Equal => DiffLineType::Same,
            },
            text: change.value().trim_end_matches('\n').to_string(),
        })
        .collect();

    Ok(GitFileDiff { lines, is_new_file })
}

#[cfg(test)]
#[path = "tests/git.rs"]
mod tests;
