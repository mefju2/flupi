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

#[test]
fn test_disabled_headers_default_empty() {
    let json = r#"{"name":"r","method":"GET","path":"/","headers":{}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert!(req.disabled_headers.is_empty());
    assert!(req.disabled_collection_headers.is_empty());
}

#[test]
fn test_disabled_headers_round_trip() {
    let json = r#"{"name":"r","method":"GET","path":"/","headers":{},"disabledHeaders":["X-Foo"],"disabledCollectionHeaders":["Authorization"]}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.disabled_headers, vec!["X-Foo"]);
    assert_eq!(req.disabled_collection_headers, vec!["Authorization"]);
    let out = serde_json::to_string(&req).unwrap();
    assert!(out.contains("\"disabledHeaders\""));
    assert!(out.contains("\"disabledCollectionHeaders\""));
}

#[test]
fn test_disabled_headers_omitted_when_empty() {
    let json = r#"{"name":"r","method":"GET","path":"/","headers":{}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    let out = serde_json::to_string(&req).unwrap();
    assert!(!out.contains("disabledHeaders"));
    assert!(!out.contains("disabledCollectionHeaders"));
}

#[test]
fn test_body_form_disabled_fields_round_trip() {
    // Old format (legacy "form") — must still deserialize
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"form","content":{"key":"val"},"disabledFields":["key"]}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::FormUrlEncoded { content: _, disabled_fields } => {
            assert_eq!(disabled_fields, vec!["key"]);
        }
        _ => panic!("expected form-urlencoded body"),
    }
}

#[test]
fn test_body_form_disabled_fields_omitted_when_empty() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"form-urlencoded","content":{"key":"val"}}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    let out = serde_json::to_string(&req).unwrap();
    assert!(!out.contains("disabledFields"));
}

// --- New model: forward tests ---

#[test]
fn test_body_raw_json_round_trip() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"raw","format":"json","content":"{\"a\":1}"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::Raw { format: RawFormat::Json, content } => {
            assert_eq!(content, "{\"a\":1}");
        }
        _ => panic!("expected raw/json body"),
    }
}

#[test]
fn test_body_raw_xml_round_trip() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"raw","format":"xml","content":"<root/>"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::Raw { format: RawFormat::Xml, content } => {
            assert_eq!(content, "<root/>");
        }
        _ => panic!("expected raw/xml body"),
    }
}

#[test]
fn test_body_form_urlencoded_round_trip() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"form-urlencoded","content":{"key":"val"},"disabledFields":["key"]}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::FormUrlEncoded { content: _, disabled_fields } => {
            assert_eq!(disabled_fields, vec!["key"]);
        }
        _ => panic!("expected form-urlencoded body"),
    }
}

#[test]
fn test_legacy_json_body_migrates_to_raw_json() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"json","content":{"key":"value"}}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::Raw { format: RawFormat::Json, content } => {
            assert!(content.contains("key"));
            assert!(content.contains("value"));
        }
        _ => panic!("expected legacy json to become raw/json"),
    }
}

#[test]
fn test_legacy_json_body_string_content_preserved() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"json","content":"{\"x\":1}"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::Raw { format: RawFormat::Json, content } => {
            assert_eq!(content, "{\"x\":1}");
        }
        _ => panic!("expected legacy json to become raw/json"),
    }
}

#[test]
fn test_legacy_form_body_migrates_to_form_urlencoded() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"form","content":{"k":"v"},"disabledFields":["k"]}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::FormUrlEncoded { content, disabled_fields } => {
            assert_eq!(content.get("k").unwrap(), "v");
            assert_eq!(disabled_fields, vec!["k"]);
        }
        _ => panic!("expected legacy form to become form-urlencoded"),
    }
}

#[test]
fn test_legacy_raw_body_migrates_to_raw_text() {
    let json = r#"{"name":"r","method":"POST","path":"/","headers":{},"body":{"type":"raw","content":"hello world"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    match req.body.unwrap() {
        BodyConfig::Raw { format: RawFormat::Text, content } => {
            assert_eq!(content, "hello world");
        }
        _ => panic!("expected legacy raw to become raw/text"),
    }
}

#[test]
fn test_body_none_round_trip() {
    let json = r#"{"name":"r","method":"GET","path":"/","headers":{},"body":{"type":"none"}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.body.unwrap(), BodyConfig::None);
}

#[test]
fn test_new_raw_serializes_with_format_field() {
    let mut req: Request = serde_json::from_str(r#"{"name":"r","method":"POST","path":"/","headers":{}}"#).unwrap();
    req.body = Some(BodyConfig::Raw { format: RawFormat::Json, content: "{}".to_string() });
    let out = serde_json::to_string(&req).unwrap();
    assert!(out.contains(r#""type":"raw""#));
    assert!(out.contains(r#""format":"json""#));
}

#[test]
fn test_new_form_urlencoded_serializes_correctly() {
    let mut req: Request = serde_json::from_str(r#"{"name":"r","method":"POST","path":"/","headers":{}}"#).unwrap();
    let mut content = indexmap::IndexMap::new();
    content.insert("k".to_string(), "v".to_string());
    req.body = Some(BodyConfig::FormUrlEncoded { content, disabled_fields: vec![] });
    let out = serde_json::to_string(&req).unwrap();
    assert!(out.contains(r#""type":"form-urlencoded""#));
    assert!(!out.contains("disabledFields"));
}
