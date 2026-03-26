use std::path::{Path, PathBuf};
use tauri::command;
use serde::Serialize;
use crate::error::FlupiError;
use crate::models::request::{Request, derive_request_id};
use crate::models::collection::Collection;
use crate::services::file_io;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum RequestTreeNode {
    Collection {
        name: String,
        folder_name: String,
        children: Vec<RequestTreeNode>,
    },
    Folder {
        name: String,
        children: Vec<RequestTreeNode>,
    },
    Request {
        id: String,
        name: String,
        method: String,
    },
}

fn scan_requests_dir(
    project_root: &Path,
    dir: &Path,
) -> Result<Vec<RequestTreeNode>, FlupiError> {
    let mut nodes: Vec<RequestTreeNode> = Vec::new();

    if !dir.exists() {
        return Ok(nodes);
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let folder_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let children = scan_requests_dir(project_root, &path)?;
            nodes.push(RequestTreeNode::Folder {
                name: folder_name,
                children,
            });
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let request: Request = file_io::read_json(&path)?;
            let id = derive_request_id(project_root, &path)?;
            nodes.push(RequestTreeNode::Request {
                id,
                name: request.name,
                method: request.method,
            });
        }
    }

    Ok(nodes)
}

pub fn build_request_tree(project_path: &Path) -> Result<Vec<RequestTreeNode>, FlupiError> {
    let mut tree: Vec<RequestTreeNode> = Vec::new();

    // Scan collections/
    let collections_dir = project_path.join("collections");
    if collections_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&collections_dir)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let collection_json = path.join("collection.json");
                let collection_name = if collection_json.exists() {
                    let col: Collection = file_io::read_json(&collection_json)?;
                    col.name
                } else {
                    folder_name.clone()
                };
                let requests_dir = path.join("requests");
                let children = scan_requests_dir(project_path, &requests_dir)?;
                tree.push(RequestTreeNode::Collection {
                    name: collection_name,
                    folder_name,
                    children,
                });
            }
        }
    }

    // Scan root requests/
    let requests_dir = project_path.join("requests");
    let root_nodes = scan_requests_dir(project_path, &requests_dir)?;
    tree.extend(root_nodes);

    Ok(tree)
}

#[command]
pub fn load_request_tree(project_path: PathBuf) -> Result<Vec<RequestTreeNode>, FlupiError> {
    build_request_tree(&project_path)
}

#[cfg(test)]
#[path = "tests/request_tree.rs"]
mod tests;
