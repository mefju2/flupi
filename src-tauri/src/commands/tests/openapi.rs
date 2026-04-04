use super::*;
use tempfile::TempDir;
use crate::services::openapi_sources;

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

    openapi_sources::add(&project_path,source).unwrap();

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

    openapi_sources::add(&project_path,source1).unwrap();
    openapi_sources::add(&project_path,source2).unwrap();

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

    openapi_sources::add(&project_path,source1).unwrap();
    openapi_sources::add(&project_path,source2).unwrap();
    openapi_sources::remove(&project_path, "src-1").unwrap();

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
    openapi_sources::add(&project_path,source).unwrap();

    let ops = fetch_operations(project_path, "src-file".to_string()).await.unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].operation_id, "getHealth");
    assert_eq!(ops[0].method, "get");
}

// ── find_candidate_operations ────────────────────────────────────────────────

fn make_ops(paths_methods: &[(&str, &str, &str)]) -> Vec<(crate::models::openapi::ImportableOperation, serde_json::Value)> {
    paths_methods
        .iter()
        .map(|(method, path, op_id)| {
            let op = crate::models::openapi::ImportableOperation {
                tag: "test".to_string(),
                operation_id: op_id.to_string(),
                method: method.to_string(),
                path: path.to_string(),
                summary: None,
            };
            (op, serde_json::json!({"operationId": op_id}))
        })
        .collect()
}

#[test]
fn test_candidates_ranked_by_normalized_score() {
    // /api/Roles (len=10) vs /api/Role (len=9): score = 2*9/19 ≈ 0.947
    // /api/Roles (len=10) vs /api/RoleIntentAssignment/role-with-intents (len=42): score = 2*9/52 ≈ 0.346
    let ops = make_ops(&[
        ("get", "/api/RoleIntentAssignment/role-with-intents", "getApiRoleRoleId"),
        ("get", "/api/Role", "getApiRole"),
    ]);
    let excluded = std::collections::HashSet::new();
    let candidates = find_candidate_operations("get", "/api/Roles", &ops, &excluded);
    assert!(!candidates.is_empty(), "Expected at least one candidate");
    // /api/Role must be ranked first (highest score)
    assert_eq!(candidates[0].path, "/api/Role",
        "Expected /api/Role as top candidate, got: {:?}", candidates.iter().map(|c| &c.path).collect::<Vec<_>>());
}

#[test]
fn test_candidates_excludes_claimed_operations() {
    let ops = make_ops(&[
        ("get", "/api/Role", "getApiRole"),
        ("get", "/api/Roles", "getApiRoles"),
    ]);
    let mut excluded = std::collections::HashSet::new();
    excluded.insert("getApiRole".to_string()); // already claimed by another request
    let candidates = find_candidate_operations("get", "/api/Roles", &ops, &excluded);
    // getApiRole is excluded; getApiRoles has different operationId but same path — should still appear
    assert!(
        candidates.iter().all(|c| c.operation_id != "getApiRole"),
        "Claimed operation must not appear in candidates"
    );
}

#[test]
fn test_candidates_filters_wrong_method() {
    let ops = make_ops(&[
        ("post", "/api/Role", "postApiRole"), // wrong method
        ("get", "/api/Role", "getApiRole"),
    ]);
    let excluded = std::collections::HashSet::new();
    let candidates = find_candidate_operations("get", "/api/Roles", &ops, &excluded);
    assert!(
        candidates.iter().all(|c| c.method == "get"),
        "Only same-method candidates should appear"
    );
}

#[test]
fn test_candidates_below_threshold_excluded() {
    // /api/Roles vs /v2/completely/different — prefix = 0, score = 0 → below 0.20 threshold
    let ops = make_ops(&[("get", "/v2/completely/different", "someOp")]);
    let excluded = std::collections::HashSet::new();
    let candidates = find_candidate_operations("get", "/api/Roles", &ops, &excluded);
    assert!(candidates.is_empty(), "Dissimilar paths must not be candidates");
}

#[test]
fn test_candidates_capped_at_eight() {
    let many_ops: Vec<(&str, String, String)> = (0..20)
        .map(|i| ("get", format!("/api/Role{}", i), format!("getRole{}", i)))
        .collect();
    let ops_ref: Vec<(&str, &str, &str)> = many_ops
        .iter()
        .map(|(m, p, id)| (*m, p.as_str(), id.as_str()))
        .collect();
    let ops = make_ops(&ops_ref);
    let excluded = std::collections::HashSet::new();
    let candidates = find_candidate_operations("get", "/api/Role", &ops, &excluded);
    assert!(candidates.len() <= 8, "Must not return more than 8 candidates");
}
