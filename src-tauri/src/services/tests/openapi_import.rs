use super::*;
use std::path::Path;
use tempfile::TempDir;

fn minimal_openapi_spec() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.0.0",
        "info": { "title": "Test API", "version": "1.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "operationId": "listPets",
                    "summary": "List all pets",
                    "tags": ["pets"],
                    "responses": {
                        "200": {
                            "content": {
                                "application/json": {
                                    "schema": { "type": "array" }
                                }
                            }
                        }
                    }
                }
            },
            "/pets/{id}": {
                "post": {
                    "operationId": "createPet",
                    "summary": "Create a pet",
                    "tags": ["pets"],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": { "type": "object", "properties": { "name": { "type": "string" } } }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "content": {
                                "application/json": {
                                    "schema": { "type": "object" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

#[test]
fn test_parse_operations_returns_two_operations() {
    let spec = minimal_openapi_spec();
    let ops = parse_operations(&spec).unwrap();
    assert_eq!(ops.len(), 2);
}

#[test]
fn test_parse_operations_correct_methods_and_paths() {
    let spec = minimal_openapi_spec();
    let mut ops = parse_operations(&spec).unwrap();
    ops.sort_by(|a, b| a.0.operation_id.cmp(&b.0.operation_id));

    let create_pet = ops.iter().find(|(op, _)| op.operation_id == "createPet").unwrap();
    assert_eq!(create_pet.0.method, "post");
    assert_eq!(create_pet.0.path, "/pets/{id}");
    assert_eq!(create_pet.0.tag, "pets");

    let list_pets = ops.iter().find(|(op, _)| op.operation_id == "listPets").unwrap();
    assert_eq!(list_pets.0.method, "get");
    assert_eq!(list_pets.0.path, "/pets");
    assert_eq!(list_pets.0.tag, "pets");
}

#[test]
fn test_parse_operations_extracts_summary() {
    let spec = minimal_openapi_spec();
    let ops = parse_operations(&spec).unwrap();
    let list_pets = ops.iter().find(|(op, _)| op.operation_id == "listPets").unwrap();
    assert_eq!(list_pets.0.summary.as_deref(), Some("List all pets"));
}

#[test]
fn test_compute_operation_hash_is_consistent() {
    let op_json = serde_json::json!({
        "operationId": "listPets",
        "summary": "List all pets"
    });
    let hash1 = compute_operation_hash(&op_json);
    let hash2 = compute_operation_hash(&op_json);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars
}

#[test]
fn test_compute_operation_hash_differs_for_different_ops() {
    let op1 = serde_json::json!({ "operationId": "op1" });
    let op2 = serde_json::json!({ "operationId": "op2" });
    assert_ne!(compute_operation_hash(&op1), compute_operation_hash(&op2));
}

#[test]
fn test_import_operations_creates_request_files() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();
    let spec = minimal_openapi_spec();
    let ops = parse_operations(&spec).unwrap();

    let created = import_operations(project_path, "src-1", &ops, "my-collection").unwrap();
    assert_eq!(created.len(), 2);

    // Verify files were created
    let list_pets_path = project_path
        .join("collections/my-collection/requests/listPets.json");
    assert!(list_pets_path.exists());

    let create_pet_path = project_path
        .join("collections/my-collection/requests/createPet.json");
    assert!(create_pet_path.exists());
}

#[test]
fn test_import_operations_sets_template_ref() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();
    let spec = minimal_openapi_spec();
    let ops = parse_operations(&spec).unwrap();

    import_operations(project_path, "src-1", &ops, "col").unwrap();

    let req_path = project_path.join("collections/col/requests/listPets.json");
    let content = std::fs::read_to_string(&req_path).unwrap();
    let req: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(req["templateRef"]["sourceId"], "src-1");
    assert_eq!(req["templateRef"]["operationId"], "listPets");
    assert!(!req["templateRef"]["schemaHash"].as_str().unwrap().is_empty());
}

#[test]
fn test_import_operations_sets_method_and_path() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path();
    let spec = minimal_openapi_spec();
    let ops = parse_operations(&spec).unwrap();

    import_operations(project_path, "src-1", &ops, "col").unwrap();

    let req_path = project_path.join("collections/col/requests/listPets.json");
    let content = std::fs::read_to_string(&req_path).unwrap();
    let req: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(req["method"].as_str().unwrap().to_uppercase(), "GET");
    assert_eq!(req["path"], "/pets");
}

#[test]
fn test_read_spec_from_file() {
    let dir = TempDir::new().unwrap();
    let spec_path = dir.path().join("spec.json");
    let spec = minimal_openapi_spec();
    std::fs::write(&spec_path, serde_json::to_string(&spec).unwrap()).unwrap();

    let loaded = read_spec_from_file(&spec_path).unwrap();
    assert_eq!(loaded["openapi"], "3.0.0");
}
