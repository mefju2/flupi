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
        file_path: String,
    },
}

fn name_to_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
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
                file_path: path.to_string_lossy().to_string(),
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

fn resolve_request_path(project_path: &Path, request_id: &str) -> PathBuf {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        let rest = parts[1];
        let collection_dir = project_path.join("collections").join(first);
        if collection_dir.exists() {
            // It's a collection request
            let rest_path = rest.replace('/', std::path::MAIN_SEPARATOR_STR);
            return project_path
                .join("collections")
                .join(first)
                .join("requests")
                .join(format!("{}.json", rest_path));
        }
    }
    // Root request
    let rest_path = request_id.replace('/', std::path::MAIN_SEPARATOR_STR);
    project_path.join("requests").join(format!("{}.json", rest_path))
}

#[command]
pub fn load_request_tree(project_path: PathBuf) -> Result<Vec<RequestTreeNode>, FlupiError> {
    build_request_tree(&project_path)
}

#[command]
pub fn get_request(project_path: PathBuf, request_id: String) -> Result<Request, FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
    file_io::read_json(&path)
}

#[command]
pub fn save_request(
    project_path: PathBuf,
    request_id: String,
    request: Request,
) -> Result<(), FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
    file_io::write_json(&path, &request)
}

#[command]
pub fn create_request(
    project_path: PathBuf,
    collection_folder: Option<String>,
    name: String,
) -> Result<String, FlupiError> {
    let slug = name_to_slug(&name);
    let (path, id) = match &collection_folder {
        Some(folder) => {
            let p = project_path
                .join("collections")
                .join(folder)
                .join("requests")
                .join(format!("{}.json", slug));
            let id = format!("{}/{}", folder, slug);
            (p, id)
        }
        None => {
            let p = project_path.join("requests").join(format!("{}.json", slug));
            (p, slug.clone())
        }
    };

    let request = Request {
        name,
        method: "GET".to_string(),
        path: "/".to_string(),
        auth: None,
        headers: indexmap::IndexMap::new(),
        body: None,
        template_ref: None,
    };
    file_io::write_json(&path, &request)?;
    Ok(id)
}

#[command]
pub fn delete_request(project_path: PathBuf, request_id: String) -> Result<(), FlupiError> {
    let path = resolve_request_path(&project_path, &request_id);
    file_io::delete_file(&path)
}

#[command]
pub fn rename_request(
    project_path: PathBuf,
    request_id: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let old_path = resolve_request_path(&project_path, &request_id);
    let new_slug = name_to_slug(&new_name);

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;
    let new_path = parent.join(format!("{}.json", new_slug));

    // Read, update name, write to new path, remove old
    let mut request: Request = file_io::read_json(&old_path)?;
    request.name = new_name;
    file_io::write_json(&new_path, &request)?;
    if old_path != new_path {
        file_io::delete_file(&old_path)?;
    }

    // Derive new ID
    let new_id = derive_request_id(&project_path, &new_path)?;
    Ok(new_id)
}

#[command]
pub fn move_request(
    project_path: PathBuf,
    request_id: String,
    target_collection_folder: Option<String>,
) -> Result<String, FlupiError> {
    let old_path = resolve_request_path(&project_path, &request_id);
    let file_name = old_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FlupiError::Custom("Cannot determine file name".to_string()))?
        .to_string();

    let new_path = match &target_collection_folder {
        Some(folder) => project_path
            .join("collections")
            .join(folder)
            .join("requests")
            .join(&file_name),
        None => project_path.join("requests").join(&file_name),
    };

    let request: Request = file_io::read_json(&old_path)?;
    file_io::write_json(&new_path, &request)?;
    file_io::delete_file(&old_path)?;

    let new_id = derive_request_id(&project_path, &new_path)?;
    Ok(new_id)
}

#[command]
pub fn duplicate_request(
    project_path: PathBuf,
    request_id: String,
) -> Result<String, FlupiError> {
    let old_path = resolve_request_path(&project_path, &request_id);
    let mut request: Request = file_io::read_json(&old_path)?;

    let stem = old_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FlupiError::Custom("Cannot determine file stem".to_string()))?;
    let new_stem = format!("{}-copy", stem);
    request.name = format!("{} copy", request.name);

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;
    let new_path = parent.join(format!("{}.json", new_stem));

    file_io::write_json(&new_path, &request)?;

    let new_id = derive_request_id(&project_path, &new_path)?;
    Ok(new_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_json_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn make_request_json(name: &str, method: &str) -> String {
        format!(
            r#"{{"name": "{}", "method": "{}", "path": "/"}}"#,
            name, method
        )
    }

    #[test]
    fn test_load_request_tree_with_collection_and_root_requests() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create collection
        write_json_file(
            &root.join("collections/auth-service/collection.json"),
            r#"{"name": "Auth Service"}"#,
        );
        write_json_file(
            &root.join("collections/auth-service/requests/get-token.json"),
            &make_request_json("Get Token", "POST"),
        );
        write_json_file(
            &root.join("collections/auth-service/requests/admin/create-user.json"),
            &make_request_json("Create User", "POST"),
        );

        // Create root requests
        write_json_file(
            &root.join("requests/health-check.json"),
            &make_request_json("Health Check", "GET"),
        );
        write_json_file(
            &root.join("requests/monitoring/status.json"),
            &make_request_json("Status", "GET"),
        );

        let tree = build_request_tree(root).unwrap();

        // Should have 1 collection + 2 root nodes (health-check request + monitoring folder)
        assert_eq!(tree.len(), 3);

        // First node is the collection
        match &tree[0] {
            RequestTreeNode::Collection { name, folder_name, children } => {
                assert_eq!(name, "Auth Service");
                assert_eq!(folder_name, "auth-service");
                // Children: admin folder + get-token request (sorted alphabetically)
                assert_eq!(children.len(), 2);
                // First child should be "admin" folder (a < g)
                match &children[0] {
                    RequestTreeNode::Folder { name, children } => {
                        assert_eq!(name, "admin");
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            RequestTreeNode::Request { id, name, method, .. } => {
                                assert_eq!(id, "auth-service/admin/create-user");
                                assert_eq!(name, "Create User");
                                assert_eq!(method, "POST");
                            }
                            _ => panic!("Expected Request node"),
                        }
                    }
                    _ => panic!("Expected Folder node"),
                }
                // Second child should be get-token request
                match &children[1] {
                    RequestTreeNode::Request { id, name, method, .. } => {
                        assert_eq!(id, "auth-service/get-token");
                        assert_eq!(name, "Get Token");
                        assert_eq!(method, "POST");
                    }
                    _ => panic!("Expected Request node"),
                }
            }
            _ => panic!("Expected Collection node"),
        }

        // Second node: health-check root request (h < m alphabetically)
        match &tree[1] {
            RequestTreeNode::Request { id, name, method, .. } => {
                assert_eq!(id, "health-check");
                assert_eq!(name, "Health Check");
                assert_eq!(method, "GET");
            }
            _ => panic!("Expected Request node"),
        }

        // Third node: monitoring folder
        match &tree[2] {
            RequestTreeNode::Folder { name, children } => {
                assert_eq!(name, "monitoring");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RequestTreeNode::Request { id, name, .. } => {
                        assert_eq!(id, "monitoring/status");
                        assert_eq!(name, "Status");
                    }
                    _ => panic!("Expected Request node"),
                }
            }
            _ => panic!("Expected Folder node"),
        }
    }

    #[test]
    fn test_load_request_tree_empty_project() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let tree = build_request_tree(root).unwrap();
        assert!(tree.is_empty());
    }

    #[test]
    fn test_load_request_tree_collection_without_collection_json() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Collection dir with no collection.json — folder_name used as name
        write_json_file(
            &root.join("collections/my-service/requests/ping.json"),
            &make_request_json("Ping", "GET"),
        );

        let tree = build_request_tree(root).unwrap();
        assert_eq!(tree.len(), 1);
        match &tree[0] {
            RequestTreeNode::Collection { name, folder_name, .. } => {
                assert_eq!(name, "my-service");
                assert_eq!(folder_name, "my-service");
            }
            _ => panic!("Expected Collection node"),
        }
    }

    #[test]
    fn test_resolve_request_path_collection() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        // Create the collection directory so resolve_request_path detects it
        std::fs::create_dir_all(root.join("collections/auth-service")).unwrap();

        let path = resolve_request_path(root, "auth-service/get-token");
        assert_eq!(
            path,
            root.join("collections/auth-service/requests/get-token.json")
        );
    }

    #[test]
    fn test_resolve_request_path_collection_nested() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("collections/auth-service")).unwrap();

        let path = resolve_request_path(root, "auth-service/admin/create-user");
        assert_eq!(
            path,
            root.join("collections/auth-service/requests/admin/create-user.json")
        );
    }

    #[test]
    fn test_resolve_request_path_root() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let path = resolve_request_path(root, "health-check");
        assert_eq!(path, root.join("requests/health-check.json"));
    }

    #[test]
    fn test_resolve_request_path_root_nested() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let path = resolve_request_path(root, "monitoring/status");
        assert_eq!(path, root.join("requests/monitoring/status.json"));
    }
}
