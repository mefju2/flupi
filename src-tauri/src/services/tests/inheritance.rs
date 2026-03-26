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
        body: None,
        template_ref: None,
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
