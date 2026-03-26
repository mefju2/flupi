use super::*;
use std::path::Path;
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
                        RequestTreeNode::Request { id, name, method } => {
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
                RequestTreeNode::Request { id, name, method } => {
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
        RequestTreeNode::Request { id, name, method } => {
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
