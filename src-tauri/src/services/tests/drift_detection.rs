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

fn make_operation_at(operation_id: &str, path: &str, hash: &str) -> (ImportableOperation, serde_json::Value) {
    let op = ImportableOperation {
        tag: "test".to_string(),
        operation_id: operation_id.to_string(),
        method: "get".to_string(),
        path: path.to_string(),
        summary: None,
    };
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
    write_request_at_path(
        project_path,
        collection,
        request_name,
        source_id,
        operation_id,
        schema_hash,
        &format!("/{}", operation_id),
    );
}

fn write_request_at_path(
    project_path: &std::path::Path,
    collection: &str,
    request_name: &str,
    source_id: &str,
    operation_id: &str,
    schema_hash: &str,
    path: &str,
) {
    let req = Request {
        name: request_name.to_string(),
        method: "GET".to_string(),
        path: path.to_string(),
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
    let file_path = project_path
        .join("collections")
        .join(collection)
        .join("requests")
        .join(format!("{}.json", request_name));
    file_io::write_json(&file_path, &req).unwrap();
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
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
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
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
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
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
    assert!(drifted.is_empty(), "Should not report drift for other sources");
}

#[test]
fn test_detect_drift_handles_empty_project() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    let (op, op_json) = make_operation("getHealth", "m");
    let ops = vec![(op, op_json)];

    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
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
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
    assert_eq!(drifted.len(), 1);
}

#[test]
fn test_detect_drift_when_operation_removed() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    write_request_with_template_ref(
        project_path,
        "my-col",
        "listPets",
        "src-1",
        "listPets",
        "some-hash",
    );

    // Spec no longer contains this operation
    let ops: Vec<(ImportableOperation, serde_json::Value)> = vec![];
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
    assert_eq!(drifted.len(), 1, "Removed operation should be marked as drifted");
    assert!(drifted[0].contains("listPets"));
}

#[test]
fn test_detect_drift_when_path_changed() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    // Spec now has the operation at the NEW path
    let (op_new, op_json) = make_operation_at("listPets", "/api/Pets", "marker1");
    let hash = crate::services::openapi_import::compute_operation_hash(&op_json);

    // Request was imported from the OLD path — same hash, same operationId, but stale path
    write_request_at_path(
        project_path,
        "my-col",
        "listPets",
        "src-1",
        "listPets",
        &hash,
        "/api/Pet",
    );

    let ops = vec![(op_new, op_json)];
    let spec = serde_json::json!({"paths": {}});
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
    assert_eq!(drifted.len(), 1, "Path change should trigger drift");
    assert!(drifted[0].contains("listPets"));
}

#[test]
fn test_detect_drift_when_resolved_schema_changes() {
    // Simulates a $ref-based schema change: the raw op_json hash stays the same
    // (same $ref string) but the resolved schema differs — e.g. a new property was
    // added to a shared component. The stored templateRef.request_schema holds the
    // previously-resolved copy, so comparing it to the freshly-resolved schema
    // must trigger drift.
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();

    // Build a spec with an inline requestBody schema (no $ref needed for the test;
    // we just verify the resolved-schema comparison path by giving the request a
    // stale stored requestSchema that differs from the live one).
    let op_json = serde_json::json!({
        "operationId": "createRole",
        "requestBody": {
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "description": { "type": "string" }
                        }
                    }
                }
            }
        }
    });
    let spec = serde_json::json!({"paths": {}});
    let current_hash = crate::services::openapi_import::compute_operation_hash(&op_json);

    let op = ImportableOperation {
        tag: "test".to_string(),
        operation_id: "createRole".to_string(),
        method: "post".to_string(),
        path: "/api/Role".to_string(),
        summary: None,
    };

    // Stored requestSchema is the OLD schema (only "name", no "description").
    // The hash is current (matches live op), so the hash check alone would NOT drift.
    let old_request_schema = serde_json::json!({
        "type": "object",
        "properties": { "name": { "type": "string" } }
    });

    let req = crate::models::request::Request {
        name: "createRole".to_string(),
        method: "POST".to_string(),
        path: "/api/Role".to_string(),
        auth: None,
        headers: indexmap::IndexMap::new(),
        path_params: indexmap::IndexMap::new(),
        body: None,
        template_ref: Some(crate::models::request::TemplateRef {
            source_id: "src-1".to_string(),
            operation_id: "createRole".to_string(),
            schema_hash: current_hash, // hash matches — diff only in resolved schema
            request_schema: old_request_schema,
            response_schema: serde_json::Value::Null,
        }),
        disabled_headers: vec![],
        disabled_collection_headers: vec![],
        extractions: vec![],
    };
    let file_path = project_path
        .join("collections")
        .join("roles")
        .join("requests")
        .join("createRole.json");
    crate::services::file_io::write_json(&file_path, &req).unwrap();

    let ops = vec![(op, op_json)];
    let drifted = detect_drift(project_path, "src-1", &ops, &spec).unwrap();
    assert_eq!(drifted.len(), 1, "Contract change (new property) must trigger drift");
    assert!(drifted[0].contains("createRole"));
}
