use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use crate::error::Result;
use crate::services::file_io;

/// Uses IndexMap instead of HashMap to preserve insertion order,
/// producing stable JSON output and clean Git diffs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: IndexMap<String, String>,
    #[serde(default)]
    pub secrets: Vec<String>,
}

pub fn resolve_env_variables(env_path: &Path) -> Result<HashMap<String, String>> {
    let env: Environment = file_io::read_json(env_path)?;
    let mut vars: HashMap<String, String> = env.variables
        .into_iter()
        .filter(|(key, _)| !env.secrets.contains(key))
        .collect();

    // Load secrets file if it exists
    // Use string replace instead of with_extension to handle filenames with dots
    let env_name = env_path.file_name().unwrap().to_string_lossy();
    let secrets_name = env_name.replace(".json", ".secrets.json");
    let secrets_path = env_path.with_file_name(secrets_name);
    if secrets_path.exists() {
        let secrets: HashMap<String, String> = file_io::read_json(&secrets_path)?;
        vars.extend(secrets);
    }

    Ok(vars)
}

pub fn list_environments(project_path: &Path) -> Result<Vec<(String, Environment)>> {
    let env_dir = project_path.join("environments");
    let mut envs = Vec::new();

    if env_dir.exists() {
        for entry in std::fs::read_dir(&env_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && !path.file_name().unwrap().to_string_lossy().ends_with(".secrets.json")
            {
                let env: Environment = file_io::read_json(&path)?;
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                envs.push((file_name, env));
            }
        }
    }

    Ok(envs)
}

#[cfg(test)]
mod tests {
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
}
