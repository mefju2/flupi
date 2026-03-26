use super::*;
use tempfile::TempDir;
use crate::services::file_io;

#[test]
fn test_load_environment() {
    let dir = TempDir::new().unwrap();
    let env_dir = dir.path().join("environments");
    std::fs::create_dir(&env_dir).unwrap();

    let env = Environment {
        name: "Dev".to_string(),
        variables: [("baseUrl".to_string(), "https://dev.api".to_string())]
            .into_iter().collect(),
        secrets: vec![],
    };
    file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();

    let loaded: Environment = file_io::read_json(&env_dir.join("dev.json")).unwrap();
    assert_eq!(loaded.name, "Dev");
    assert_eq!(loaded.variables["baseUrl"], "https://dev.api");
}

#[test]
fn test_load_environment_with_secrets() {
    let dir = TempDir::new().unwrap();
    let env_dir = dir.path().join("environments");
    std::fs::create_dir(&env_dir).unwrap();

    let env = Environment {
        name: "Dev".to_string(),
        variables: [
            ("baseUrl".to_string(), "https://dev.api".to_string()),
            ("client_secret".to_string(), String::new()),
        ].into_iter().collect(),
        secrets: vec!["client_secret".to_string()],
    };
    file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();

    let secrets: std::collections::HashMap<String, String> =
        [("client_secret".to_string(), "supersecret".to_string())]
            .into_iter().collect();
    file_io::write_json(&env_dir.join("dev.secrets.json"), &secrets).unwrap();

    let resolved = resolve_env_variables(&env_dir.join("dev.json")).unwrap();
    assert_eq!(resolved["baseUrl"], "https://dev.api");
    assert_eq!(resolved["client_secret"], "supersecret");
}

#[test]
fn test_secret_key_excluded_from_variables() {
    let dir = TempDir::new().unwrap();
    let env_dir = dir.path().join("environments");
    std::fs::create_dir(&env_dir).unwrap();

    let env = Environment {
        name: "Dev".to_string(),
        variables: [
            ("client_secret".to_string(), "placeholder-should-not-load".to_string()),
        ].into_iter().collect(),
        secrets: vec!["client_secret".to_string()],
    };
    file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();
    // No secrets file exists

    let resolved = resolve_env_variables(&env_dir.join("dev.json")).unwrap();
    // Secret key should NOT be in the resolved map (no secrets file)
    assert!(!resolved.contains_key("client_secret"));
}
