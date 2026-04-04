use super::*;

#[test]
fn test_extraction_round_trip() {
    let ext = Extraction {
        variable: "token".to_string(),
        from: "response.body".to_string(),
        path: "$.auth.token".to_string(),
    };

    let json = serde_json::to_string(&ext).unwrap();
    let parsed: Extraction = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.variable, "token");
    assert_eq!(parsed.from, "response.body");
    assert_eq!(parsed.path, "$.auth.token");
}

#[test]
fn test_extraction_equality() {
    let a = Extraction {
        variable: "x".to_string(),
        from: "response.body".to_string(),
        path: "$.x".to_string(),
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_extraction_inequality() {
    let a = Extraction {
        variable: "x".to_string(),
        from: "response.body".to_string(),
        path: "$.x".to_string(),
    };
    let b = Extraction {
        variable: "y".to_string(),
        from: "response.body".to_string(),
        path: "$.y".to_string(),
    };
    assert_ne!(a, b);
}

#[test]
fn test_extraction_header_source() {
    let json = r#"{"variable": "loc", "from": "response.headers", "path": "Location"}"#;
    let ext: Extraction = serde_json::from_str(json).unwrap();
    assert_eq!(ext.from, "response.headers");
    assert_eq!(ext.path, "Location");
}
