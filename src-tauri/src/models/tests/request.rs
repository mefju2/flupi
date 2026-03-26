use super::*;

#[test]
fn test_derive_request_id_collection() {
    let project = Path::new("/project");
    let path = Path::new("/project/collections/auth-service/requests/get-token.json");
    assert_eq!(derive_request_id(project, path).unwrap(), "auth-service/get-token");
}

#[test]
fn test_derive_request_id_collection_nested() {
    let project = Path::new("/project");
    let path = Path::new("/project/collections/auth-service/requests/admin/create-user.json");
    assert_eq!(derive_request_id(project, path).unwrap(), "auth-service/admin/create-user");
}

#[test]
fn test_derive_request_id_root() {
    let project = Path::new("/project");
    let path = Path::new("/project/requests/health-check.json");
    assert_eq!(derive_request_id(project, path).unwrap(), "health-check");
}

#[test]
fn test_derive_request_id_root_nested() {
    let project = Path::new("/project");
    let path = Path::new("/project/requests/monitoring/status.json");
    assert_eq!(derive_request_id(project, path).unwrap(), "monitoring/status");
}
