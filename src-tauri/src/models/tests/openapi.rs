use super::*;

#[test]
fn test_openapi_source_url_serializes_with_type_url() {
    let source = OpenApiSource::Url {
        id: "src-1".to_string(),
        name: "My API".to_string(),
        url: "https://example.com/openapi.json".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    let json = serde_json::to_value(&source).unwrap();
    assert_eq!(json["type"], "url");
    assert_eq!(json["id"], "src-1");
    assert_eq!(json["name"], "My API");
    assert_eq!(json["url"], "https://example.com/openapi.json");
    assert!(json.get("lastFetchedAt").is_none() || json["lastFetchedAt"].is_null());
}

#[test]
fn test_openapi_source_file_serializes_with_type_file() {
    let source = OpenApiSource::File {
        id: "src-2".to_string(),
        name: "Local API".to_string(),
        path: "/path/to/spec.json".to_string(),
        last_fetched_at: Some("2026-03-27T00:00:00Z".to_string()),
        last_hash: Some("abc123".to_string()),
    };
    let json = serde_json::to_value(&source).unwrap();
    assert_eq!(json["type"], "file");
    assert_eq!(json["id"], "src-2");
    assert_eq!(json["path"], "/path/to/spec.json");
    // lastFetchedAt is skipped in serialization — it lives in app_data_dir instead
    // lastFetchedAt is excluded from the project file (skip_deserializing, skip_serializing_if = None)
    // but IS included when Some — used for the IPC response after being injected from app_data_dir.
    assert_eq!(json["lastFetchedAt"], "2026-03-27T00:00:00Z");
    assert_eq!(json["lastHash"], "abc123");
}

#[test]
fn test_openapi_sources_round_trips() {
    let sources = OpenApiSources {
        sources: vec![
            OpenApiSource::Url {
                id: "u1".to_string(),
                name: "Remote".to_string(),
                url: "https://api.example.com/spec".to_string(),
                last_fetched_at: None,
                last_hash: None,
            },
            OpenApiSource::File {
                id: "f1".to_string(),
                name: "Local".to_string(),
                path: "/tmp/spec.json".to_string(),
                last_fetched_at: None,
                last_hash: None,
            },
        ],
    };

    let json = serde_json::to_string(&sources).unwrap();
    let restored: OpenApiSources = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.sources.len(), 2);
    assert_eq!(restored.sources[0].id(), "u1");
    assert_eq!(restored.sources[1].id(), "f1");
}

#[test]
fn test_openapi_source_id_getter() {
    let url_src = OpenApiSource::Url {
        id: "url-id".to_string(),
        name: "".to_string(),
        url: "".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    let file_src = OpenApiSource::File {
        id: "file-id".to_string(),
        name: "".to_string(),
        path: "".to_string(),
        last_fetched_at: None,
        last_hash: None,
    };
    assert_eq!(url_src.id(), "url-id");
    assert_eq!(file_src.id(), "file-id");
}

#[test]
fn test_importable_operation_serializes_camel_case() {
    let op = ImportableOperation {
        tag: "pets".to_string(),
        operation_id: "listPets".to_string(),
        method: "get".to_string(),
        path: "/pets".to_string(),
        summary: Some("List all pets".to_string()),
    };
    let json = serde_json::to_value(&op).unwrap();
    assert_eq!(json["tag"], "pets");
    assert_eq!(json["operationId"], "listPets");
    assert_eq!(json["method"], "get");
    assert_eq!(json["path"], "/pets");
    assert_eq!(json["summary"], "List all pets");
}
