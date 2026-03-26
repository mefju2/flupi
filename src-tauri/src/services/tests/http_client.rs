use super::*;

#[tokio::test]
async fn test_build_reqwest_request() {
    let req = ExecutableRequest {
        method: "GET".to_string(),
        url: "https://httpbin.org/get".to_string(),
        headers: [("Accept".to_string(), "application/json".to_string())].into(),
        body: None,
        timeout_ms: 30000,
    };

    let built = build_request(&req).unwrap();
    assert_eq!(built.method().as_str(), "GET");
}

#[tokio::test]
async fn test_build_post_request_with_json_body() {
    let req = ExecutableRequest {
        method: "POST".to_string(),
        url: "https://httpbin.org/post".to_string(),
        headers: HashMap::new(),
        body: Some(RequestBody::Json {
            content: serde_json::json!({"key": "value"}),
        }),
        timeout_ms: 30000,
    };

    let built = build_request(&req).unwrap();
    assert_eq!(built.method().as_str(), "POST");
}
