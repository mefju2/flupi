use super::*;
use tempfile::TempDir;

fn write_scenario(dir: &std::path::Path, filename: &str, request_id: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let scenario = serde_json::json!({
        "name": "Test",
        "steps": [
            {
                "id": "s1",
                "name": "Step 1",
                "requestId": request_id,
                "overrides": {},
                "extract": []
            }
        ]
    });
    let path = dir.join(filename);
    std::fs::write(&path, serde_json::to_string_pretty(&scenario).unwrap()).unwrap();
}

#[test]
fn test_find_scenarios_referencing_request() {
    let tmp = TempDir::new().unwrap();
    let scenarios_dir = tmp.path().join("scenarios");
    write_scenario(&scenarios_dir, "auth.json", "auth/get-token");

    let results = find_references(tmp.path(), "auth/get-token").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_find_scenarios_no_match() {
    let tmp = TempDir::new().unwrap();
    let scenarios_dir = tmp.path().join("scenarios");
    write_scenario(&scenarios_dir, "auth.json", "auth/get-token");

    let results = find_references(tmp.path(), "other/request").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_update_references() {
    let tmp = TempDir::new().unwrap();
    let scenarios_dir = tmp.path().join("scenarios");
    write_scenario(&scenarios_dir, "auth.json", "auth/get-token");

    update_references(tmp.path(), "auth/get-token", "auth/new-token").unwrap();

    let path = scenarios_dir.join("auth.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["steps"][0]["requestId"], "auth/new-token");
}

#[test]
fn test_update_references_no_match() {
    let tmp = TempDir::new().unwrap();
    let scenarios_dir = tmp.path().join("scenarios");
    write_scenario(&scenarios_dir, "auth.json", "auth/get-token");

    update_references(tmp.path(), "nonexistent/request", "auth/new-token").unwrap();

    let path = scenarios_dir.join("auth.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["steps"][0]["requestId"], "auth/get-token");
}
