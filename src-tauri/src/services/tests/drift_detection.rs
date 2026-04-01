use super::*;
use tempfile::TempDir;
use crate::models::openapi::ImportableOperation;
use crate::models::request::{Request, TemplateRef};
use crate::services::file_io;
use indexmap::IndexMap;

fn make_operation(operation_id: &str, hash: &str) -> (ImportableOperation, serde_json::Value) {
    let op = ImportableOperation {
        tag: "test".to_string(),
        operation_id: operation_id.to_string(),
        method: "get".to_string(),
        path: format!("/{}", operation_id),
        summary: None,
    };
    // Create a JSON value whose hash will match `hash` by using it directly
    // We'll store the hash separately; compute_operation_hash will give a real hash
    let json = serde_json::json!({ "operationId": operation_id, "_marker": hash });
    (op, json)
}

fn write_request_with_template_ref(
    project_path: &std::path::Path,
    collection: &str,
    request_name: &str,
    source_id: &str,
    operation_id: &str,
    schema_hash: &str,
) {
    let req = Request {
        name: request_name.to_string(),
        method: "GET".to_string(),
        path: format!("/{}", operation_id),
        auth: None,
        headers: IndexMap::new(),
        path_params: IndexMap::new(),
        body: None,
        template_ref: Some(TemplateRef {
            source_id: source_id.to_string(),
            operation_id: operation_id.to_string(),
            schema_hash: schema_hash.to_string(),
            request_schema: serde_json::Value::Null,
            response_schema: serde_json::Value::Null,
        }),
        disabled_headers: vec![],
        disabled_collection_headers: vec![],
        extractions: vec![],
    };
    let path = project_path
        .join("collections")
        .join(collection)
        .join("requests")
        .join(format!("{}.json", request_name));
    file_io::write_json(&path, &req).unwrap();
}

#[test]
fn test_detect_drift_no_drift_when_hash_matches() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("listPets", "marker1");
    let correct_hash = crate::services::openapi_import::compute_operation_hash(&op_json);

    write_request_with_template_ref(
        project_path,
        "my-col",
        "listPets",
        "src-1",
        "listPets",
        &correct_hash,
    );

    let ops = vec![(op, op_json)];
    let drifted = detect_drift(project_path, "src-1", &ops).unwrap();
    assert!(drifted.is_empty(), "Expected no drift, got: {:?}", drifted);
}

#[test]
fn test_detect_drift_returns_id_when_hash_differs() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("listPets", "marker1");

    // Store a wrong hash in the request
    write_request_with_template_ref(
        project_path,
        "my-col",
        "listPets",
        "src-1",
        "listPets",
        "outdated-hash-value",
    );

    let ops = vec![(op, op_json)];
    let drifted = detect_drift(project_path, "src-1", &ops).unwrap();
    assert_eq!(drifted.len(), 1);
    assert!(drifted[0].contains("listPets"));
}

#[test]
fn test_detect_drift_ignores_other_sources() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("listPets", "marker1");
    let correct_hash = crate::services::openapi_import::compute_operation_hash(&op_json);

    // Write a request that belongs to a different source
    write_request_with_template_ref(
        project_path,
        "my-col",
        "listPets",
        "src-other",
        "listPets",
        "whatever-hash",
    );

    // Also write a request for src-1 with correct hash
    write_request_with_template_ref(
        project_path,
        "my-col",
        "myOp",
        "src-1",
        "listPets",
        &correct_hash,
    );

    let ops = vec![(op, op_json)];
    let drifted = detect_drift(project_path, "src-1", &ops).unwrap();
    assert!(drifted.is_empty(), "Should not report drift for other sources");
}

#[test]
fn test_detect_drift_handles_empty_project() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("getHealth", "m");
    let ops = vec![(op, op_json)];

    let drifted = detect_drift(project_path, "src-1", &ops).unwrap();
    assert!(drifted.is_empty());
}

#[test]
fn test_detect_drift_multiple_collections() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("listPets", "marker1");
    let correct_hash = crate::services::openapi_import::compute_operation_hash(&op_json);

    // One request with correct hash, one with wrong hash, in different collections
    write_request_with_template_ref(
        project_path,
        "col-a",
        "listPets",
        "src-1",
        "listPets",
        &correct_hash,
    );
    write_request_with_template_ref(
        project_path,
        "col-b",
        "listPets",
        "src-1",
        "listPets",
        "stale-hash",
    );

    let ops = vec![(op, op_json)];
    let drifted = detect_drift(project_path, "src-1", &ops).unwrap();
    assert_eq!(drifted.len(), 1);
}
