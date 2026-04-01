use super::*;
use crate::models::request::{Request, AuthConfig};
use crate::models::collection::Collection;
use indexmap::IndexMap;

fn make_request(
    path: &str,
    auth: Option<AuthConfig>,
    headers: IndexMap<String, String>,
) -> Request {
    Request {
        name: "Test".to_string(),
        method: "GET".to_string(),
        path: path.to_string(),
        auth,
        headers,
        path_params: IndexMap::new(),
        body: None,
        template_ref: None,
        disabled_headers: vec![],
        disabled_collection_headers: vec![],
        extractions: vec![],
    }
}

fn make_collection(
    base_url: Option<&str>,
    auth: Option<AuthConfig>,
    headers: IndexMap<String, String>,
) -> Collection {
    Collection {
        name: "Test".to_string(),
        base_url: base_url.map(|s| s.to_string()),
        auth,
        headers,
    }
}

#[test]
fn test_inherit_auth_from_collection() {
    let collection = make_collection(
        None,
        Some(AuthConfig::Bearer {
            token: "{{token}}".to_string(),
        }),
        IndexMap::new(),
    );
    let request = make_request("/resource", Some(AuthConfig::Inherit), IndexMap::new());

    let effective = resolve_inheritance(&request, Some(&collection));
    match effective.auth.unwrap() {
        AuthConfig::Bearer { token } => assert_eq!(token, "{{token}}"),
        _ => panic!("expected bearer auth"),
    }
}

#[test]
fn test_merge_headers_request_wins() {
    let mut col_headers = IndexMap::new();
    col_headers.insert("Content-Type".to_string(), "application/json".to_string());
    let collection = make_collection(None, None, col_headers);

    let mut req_headers = IndexMap::new();
    req_headers.insert("Content-Type".to_string(), "text/plain".to_string());
    let request = make_request("/resource", None, req_headers);

    let effective = resolve_inheritance(&request, Some(&collection));
    assert_eq!(effective.headers["Content-Type"], "text/plain");
}

#[test]
fn test_prepend_base_url() {
    let collection = make_collection(Some("https://api.dev"), None, IndexMap::new());
    let request = make_request("/resource", None, IndexMap::new());

    let effective = resolve_inheritance(&request, Some(&collection));
    assert_eq!(effective.path, "https://api.dev/resource");
}

#[test]
fn test_absolute_url_not_prepended() {
    let collection = make_collection(Some("https://api.dev"), None, IndexMap::new());
    let request = make_request("https://other.api/resource", None, IndexMap::new());

    let effective = resolve_inheritance(&request, Some(&collection));
    assert_eq!(effective.path, "https://other.api/resource");
}

#[test]
fn test_template_base_url_not_prepended() {
    // {{BaseUrl}} likely resolves to a full URL; collection base_url must not be prepended
    let collection = make_collection(Some("https://api.dev"), None, IndexMap::new());
    let request = make_request("{{BaseUrl}}/resource", None, IndexMap::new());

    let effective = resolve_inheritance(&request, Some(&collection));
    assert_eq!(effective.path, "{{BaseUrl}}/resource");
}

#[test]
fn test_disabled_collection_header_excluded() {
    let mut col_headers = IndexMap::new();
    col_headers.insert("X-Tenant".to_string(), "acme".to_string());
    col_headers.insert("X-Version".to_string(), "2".to_string());
    let collection = make_collection(None, None, col_headers);

    let mut req = make_request("/resource", None, IndexMap::new());
    req.disabled_collection_headers = vec!["X-Tenant".to_string()];

    let effective = resolve_inheritance(&req, Some(&collection));
    assert!(!effective.headers.contains_key("X-Tenant"), "disabled collection header must be excluded");
    assert_eq!(effective.headers["X-Version"], "2");
}

#[test]
fn test_disabled_request_header_excluded() {
    let collection = make_collection(None, None, IndexMap::new());

    let mut req_headers = IndexMap::new();
    req_headers.insert("X-Debug".to_string(), "true".to_string());
    req_headers.insert("Accept".to_string(), "application/json".to_string());
    let mut req = make_request("/resource", None, req_headers);
    req.disabled_headers = vec!["X-Debug".to_string()];

    let effective = resolve_inheritance(&req, Some(&collection));
    assert!(!effective.headers.contains_key("X-Debug"), "disabled request header must be excluded");
    assert_eq!(effective.headers["Accept"], "application/json");
}
