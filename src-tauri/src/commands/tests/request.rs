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

#[test]
fn test_duplicate_request_no_collision() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    write_json_file(
        &root.join("requests/ping.json"),
        &make_request_json("Ping", "GET"),
    );

    let new_id = duplicate_request(root.clone(), "ping".to_string()).unwrap();
    assert_eq!(new_id, "ping-copy");
    assert!(root.join("requests/ping-copy.json").exists());
}

#[test]
fn test_duplicate_request_collision_increments() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    write_json_file(
        &root.join("requests/ping.json"),
        &make_request_json("Ping", "GET"),
    );
    write_json_file(
        &root.join("requests/ping-copy.json"),
        &make_request_json("Ping copy", "GET"),
    );

    let new_id = duplicate_request(root.clone(), "ping".to_string()).unwrap();
    assert_eq!(new_id, "ping-copy-2");
    assert!(root.join("requests/ping-copy-2.json").exists());
}

#[test]
fn test_duplicate_request_all_collide_returns_error() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    write_json_file(
        &root.join("requests/ping.json"),
        &make_request_json("Ping", "GET"),
    );
    // Occupy ping-copy and ping-copy-2 through ping-copy-10
    write_json_file(
        &root.join("requests/ping-copy.json"),
        &make_request_json("Ping copy", "GET"),
    );
    for n in 2..=10 {
        write_json_file(
            &root.join(format!("requests/ping-copy-{}.json", n)),
            &make_request_json(&format!("Ping copy {}", n), "GET"),
        );
    }

    let result = duplicate_request(root.clone(), "ping".to_string());
    assert!(result.is_err());
}
