use super::*;
use std::collections::HashMap;
use crate::models::extraction::Extraction;

fn codes(patterns: &[&str]) -> Vec<String> {
    patterns.iter().map(|s| s.to_string()).collect()
}

fn make_extraction(variable: &str, from: &str, path: &str) -> Extraction {
    Extraction {
        variable: variable.to_string(),
        from: from.to_string(),
        path: path.to_string(),
    }
}

#[test]
fn test_apply_extraction_body_string_value() {
    let ext = make_extraction("token", "response.body", "$.data.token");
    let body = r#"{"data": {"token": "abc123"}}"#;
    let headers = HashMap::new();

    let result = apply_extraction(&ext, body, &headers).unwrap();
    assert_eq!(result, "abc123");
}

#[test]
fn test_apply_extraction_body_number_value() {
    let ext = make_extraction("id", "response.body", "$.id");
    let body = r#"{"id": 42}"#;
    let headers = HashMap::new();

    let result = apply_extraction(&ext, body, &headers).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_apply_extraction_body_no_match() {
    let ext = make_extraction("missing", "response.body", "$.does.not.exist");
    let body = r#"{"data": {}}"#;
    let headers = HashMap::new();

    let result = apply_extraction(&ext, body, &headers);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No match for path"));
}

#[test]
fn test_apply_extraction_body_invalid_json() {
    let ext = make_extraction("token", "response.body", "$.token");
    let body = "not valid json";
    let headers = HashMap::new();

    let result = apply_extraction(&ext, body, &headers);
    assert!(result.is_err());
}

#[test]
fn test_apply_extraction_header_found() {
    let ext = make_extraction("location", "response.headers", "Location");
    let body = "";
    let mut headers = HashMap::new();
    headers.insert("Location".to_string(), "/new/path".to_string());

    let result = apply_extraction(&ext, body, &headers).unwrap();
    assert_eq!(result, "/new/path");
}

#[test]
fn test_apply_extraction_header_not_found() {
    let ext = make_extraction("missing", "response.headers", "X-Missing-Header");
    let body = "";
    let headers = HashMap::new();

    let result = apply_extraction(&ext, body, &headers);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Header X-Missing-Header not found"));
}

#[test]
fn test_apply_overrides_adds_entries() {
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("base_url".to_string(), "http://localhost".to_string());

    let mut overrides = HashMap::new();
    overrides.insert("path".to_string(), "/api/v2/login".to_string());
    overrides.insert("method".to_string(), "POST".to_string());

    apply_overrides(&mut vars, &overrides);

    assert_eq!(vars.get("base_url").unwrap(), "http://localhost");
    assert_eq!(vars.get("path").unwrap(), "/api/v2/login");
    assert_eq!(vars.get("method").unwrap(), "POST");
}

#[test]
fn test_apply_overrides_overwrites_existing() {
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("token".to_string(), "old-token".to_string());

    let mut overrides = HashMap::new();
    overrides.insert("token".to_string(), "new-token".to_string());

    apply_overrides(&mut vars, &overrides);

    assert_eq!(vars.get("token").unwrap(), "new-token");
}

#[test]
fn test_status_is_expected_empty_list_defaults_to_2xx() {
    assert!(status_is_expected(200, &[]));
    assert!(status_is_expected(201, &[]));
    assert!(status_is_expected(299, &[]));
    assert!(!status_is_expected(400, &[]));
    assert!(!status_is_expected(500, &[]));
    assert!(!status_is_expected(301, &[]));
}

#[test]
fn test_status_is_expected_exact_match() {
    assert!(status_is_expected(400, &codes(&["400"])));
    assert!(!status_is_expected(200, &codes(&["400"])));
    assert!(status_is_expected(200, &codes(&["200", "400"])));
    assert!(status_is_expected(400, &codes(&["200", "400"])));
    assert!(!status_is_expected(500, &codes(&["200", "400"])));
}

#[test]
fn test_status_is_expected_wildcard_2xx() {
    assert!(status_is_expected(200, &codes(&["2**"])));
    assert!(status_is_expected(250, &codes(&["2**"])));
    assert!(status_is_expected(299, &codes(&["2**"])));
    assert!(!status_is_expected(300, &codes(&["2**"])));
    assert!(!status_is_expected(400, &codes(&["2**"])));
}

#[test]
fn test_status_is_expected_wildcard_4xx_partial() {
    assert!(status_is_expected(400, &codes(&["40*"])));
    assert!(status_is_expected(409, &codes(&["40*"])));
    assert!(!status_is_expected(410, &codes(&["40*"])));
    assert!(!status_is_expected(500, &codes(&["40*"])));
}

#[test]
fn test_status_is_expected_all_wildcard() {
    assert!(status_is_expected(200, &codes(&["***"])));
    assert!(status_is_expected(400, &codes(&["***"])));
    assert!(status_is_expected(500, &codes(&["***"])));
}

#[test]
fn test_apply_overrides_empty() {
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("key".to_string(), "value".to_string());

    apply_overrides(&mut vars, &HashMap::new());

    assert_eq!(vars.len(), 1);
    assert_eq!(vars.get("key").unwrap(), "value");
}
