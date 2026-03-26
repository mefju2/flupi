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
#[path = "tests/environment.rs"]
mod tests;
