use super::*;

#[test]
fn test_collection_round_trip() {
    let json = r#"{
        "name": "Auth Service",
        "baseUrl": "https://api.example.com",
        "auth": {"type": "bearer", "token": "{{token}}"},
        "headers": {"X-Request-Id": "{{requestId}}"}
    }"#;

    let col: Collection = serde_json::from_str(json).unwrap();
    assert_eq!(col.name, "Auth Service");
    assert_eq!(col.base_url.as_deref(), Some("https://api.example.com"));
    assert!(col.auth.is_some());
    assert_eq!(col.headers.len(), 1);

    let serialized = serde_json::to_string(&col).unwrap();
    let re_parsed: Collection = serde_json::from_str(&serialized).unwrap();
    assert_eq!(re_parsed.name, col.name);
    assert_eq!(re_parsed.base_url, col.base_url);
}

#[test]
fn test_collection_defaults() {
    let json = r#"{"name": "Minimal"}"#;
    let col: Collection = serde_json::from_str(json).unwrap();
    assert_eq!(col.name, "Minimal");
    assert!(col.base_url.is_none());
    assert!(col.auth.is_none());
    assert!(col.headers.is_empty());
}

#[test]
fn test_collection_optional_fields_not_serialized_when_absent() {
    let col = Collection {
        name: "Test".to_string(),
        base_url: None,
        auth: None,
        headers: indexmap::IndexMap::new(),
    };
    let json = serde_json::to_string(&col).unwrap();
    assert!(!json.contains("baseUrl"));
    assert!(!json.contains("auth"));
}
