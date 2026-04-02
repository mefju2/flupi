use super::*;
use tempfile::TempDir;

fn minimal_spec_json() -> String {
    serde_json::json!({
        "openapi": "3.0.0",
        "info": { "title": "Test API", "version": "1.0.0" },
        "paths": {
            "/health": {
                "get": {
                    "operationId": "getHealth",
                    "summary": "Health check",
                    "tags": ["health"],
                    "responses": { "200": {} }
                }
            }
        }
    })
    .to_string()
}

#[test]
fn test_add_openapi_source_creates_file() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().to_path_buf();

    let source = crate::models::openapi::OpenApiSource::Url {
        id: "src-1".to_string(),
        name: "My API".to_string(),
        url: "https://example.com/api.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };

    add_source_to_disk(&project_path, source).unwrap();

    let sources_path = project_path.join("openapi-sources.json");
    assert!(sources_path.exists());

    let content = std::fs::read_to_string(&sources_path).unwrap();
    let sources: crate::models::openapi::OpenApiSources = serde_json::from_str(&content).unwrap();
    assert_eq!(sources.sources.len(), 1);
    assert_eq!(sources.sources[0].id(), "src-1");
}

#[test]
fn test_add_openapi_source_appends_to_existing() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().to_path_buf();

    let source1 = crate::models::openapi::OpenApiSource::File {
        id: "src-1".to_string(),
        name: "First".to_string(),
        path: "/tmp/spec1.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    let source2 = crate::models::openapi::OpenApiSource::File {
        id: "src-2".to_string(),
        name: "Second".to_string(),
        path: "/tmp/spec2.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };

    add_source_to_disk(&project_path, source1).unwrap();
    add_source_to_disk(&project_path, source2).unwrap();

    let sources = list_openapi_sources(project_path).unwrap();
    assert_eq!(sources.len(), 2);
}

#[test]
fn test_remove_openapi_source() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().to_path_buf();

    let source1 = crate::models::openapi::OpenApiSource::File {
        id: "src-1".to_string(),
        name: "First".to_string(),
        path: "/tmp/spec1.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    let source2 = crate::models::openapi::OpenApiSource::File {
        id: "src-2".to_string(),
        name: "Second".to_string(),
        path: "/tmp/spec2.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };

    add_source_to_disk(&project_path, source1).unwrap();
    add_source_to_disk(&project_path, source2).unwrap();
    remove_source_from_disk(&project_path, "src-1").unwrap();

    let sources = list_openapi_sources(project_path).unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].id(), "src-2");
}

#[test]
fn test_list_openapi_sources_empty_when_no_file() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().to_path_buf();

    let sources = list_openapi_sources(project_path).unwrap();
    assert!(sources.is_empty());
}

#[tokio::test]
async fn test_fetch_operations_from_file_source() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().to_path_buf();

    let spec_path = dir.path().join("spec.json");
    std::fs::write(&spec_path, minimal_spec_json()).unwrap();

    let source = crate::models::openapi::OpenApiSource::File {
        id: "src-file".to_string(),
        name: "Local Spec".to_string(),
        path: spec_path.to_str().unwrap().to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    add_source_to_disk(&project_path, source).unwrap();

    let ops = fetch_operations(project_path, "src-file".to_string()).await.unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].operation_id, "getHealth");
    assert_eq!(ops[0].method, "get");
}
