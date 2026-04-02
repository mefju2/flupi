use super::*;
use tempfile::TempDir;

fn spec_with_body() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.0.0",
        "info": { "title": "Test", "version": "1.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "operationId": "listPets",
                    "tags": ["pets"],
                    "responses": { "200": { "description": "ok" } }
                }
            },
            "/pets/{id}": {
                "post": {
                    "operationId": "createPet",
                    "tags": ["pets"],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": { "name": { "type": "string" } }
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "created" } }
                }
            }
        }
    })
}

#[test]
fn test_import_operations_sets_json_body_for_post_with_schema() {
    let dir = TempDir::new().unwrap();
    let spec = spec_with_body();
    let ops = parse_operations(&spec).unwrap();

    import_operations(dir.path(), "src-1", &ops, "col", &spec).unwrap();

    let content = std::fs::read_to_string(
        dir.path().join("collections/col/requests/createPet.json")
    ).unwrap();
    let req: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(req["body"]["type"], "json");
    assert_eq!(req["body"]["content"]["name"], "string");
}

#[test]
fn test_import_operations_no_body_for_get_without_schema() {
    let dir = TempDir::new().unwrap();
    let spec = spec_with_body();
    let ops = parse_operations(&spec).unwrap();

    import_operations(dir.path(), "src-1", &ops, "col", &spec).unwrap();

    let content = std::fs::read_to_string(
        dir.path().join("collections/col/requests/listPets.json")
    ).unwrap();
    let req: serde_json::Value = serde_json::from_str(&content).unwrap();

    // GET with no requestBody — body field must be absent
    assert!(req.get("body").is_none());
}
