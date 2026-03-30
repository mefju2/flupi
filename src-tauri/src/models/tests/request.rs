use super::*;

#[test]
fn test_path_params_default_empty() {
    let json = r#"{"name":"r","method":"GET","path":"/users","headers":{}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert!(req.path_params.is_empty());
}

#[test]
fn test_path_params_round_trip() {
    let json = r#"{"name":"r","method":"GET","path":"/users/{id}","headers":{},"pathParams":{"id":"{{userId}}"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.path_params.get("id").unwrap(), "{{userId}}");
    let out = serde_json::to_string(&req).unwrap();
    assert!(out.contains("\"pathParams\""));
}

#[test]
fn test_path_params_omitted_when_empty() {
    let json = r#"{"name":"r","method":"GET","path":"/users","headers":{}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    let out = serde_json::to_string(&req).unwrap();
    assert!(!out.contains("pathParams"));
}

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
