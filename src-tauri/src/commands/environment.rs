use crate::error::FlupiError;
use crate::models::environment::{self, Environment};
use crate::services::file_io;
use crate::utils::name_to_slug;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;

#[command]
pub fn list_environments(project_path: PathBuf) -> Result<Vec<(String, Environment)>, FlupiError> {
    environment::list_environments(&project_path)
}

#[command]
pub fn save_environment(
    project_path: PathBuf,
    file_name: String,
    env: Environment,
) -> Result<(), FlupiError> {
    let path = project_path
        .join("environments")
        .join(format!("{}.json", file_name));
    file_io::write_json(&path, &env)
}

#[command]
pub fn save_secrets(
    project_path: PathBuf,
    file_name: String,
    secrets: HashMap<String, String>,
) -> Result<(), FlupiError> {
    let path = project_path
        .join("environments")
        .join(format!("{}.secrets.json", file_name));
    file_io::write_json(&path, &secrets)
}

#[command]
pub fn get_resolved_variables(
    project_path: PathBuf,
    file_name: String,
) -> Result<HashMap<String, String>, FlupiError> {
    let path = project_path
        .join("environments")
        .join(format!("{}.json", file_name));
    environment::resolve_env_variables(&path)
}

#[command]
pub fn delete_environment(project_path: PathBuf, file_name: String) -> Result<(), FlupiError> {
    let env_path = project_path
        .join("environments")
        .join(format!("{}.json", file_name));
    let secrets_path = project_path
        .join("environments")
        .join(format!("{}.secrets.json", file_name));
    file_io::delete_file(&env_path)?;
    if secrets_path.exists() {
        file_io::delete_file(&secrets_path)?;
    }
    Ok(())
}

#[command]
pub fn duplicate_environment(
    project_path: PathBuf,
    file_name: String,
) -> Result<String, FlupiError> {
    let env_dir = project_path.join("environments");
    let old_env_path = env_dir.join(format!("{}.json", file_name));
    let mut env: Environment = file_io::read_json(&old_env_path)?;

    // Find a non-colliding file_name: "{file_name}-copy", then "{file_name}-copy-2" … up to "-copy-10"
    let new_file_name = {
        let candidate = format!("{}-copy", file_name);
        if !env_dir.join(format!("{}.json", candidate)).exists() {
            candidate
        } else {
            let mut found: Option<String> = None;
            for n in 2..=10 {
                let c = format!("{}-copy-{}", file_name, n);
                if !env_dir.join(format!("{}.json", c)).exists() {
                    found = Some(c);
                    break;
                }
            }
            found
                .ok_or_else(|| FlupiError::Custom("Too many duplicate environments".to_string()))?
        }
    };

    // Human-readable copy name
    let base_name = env.name.clone();
    env.name = if new_file_name.ends_with("-copy") {
        format!("{} copy", base_name)
    } else {
        let suffix = new_file_name.rsplit('-').next().unwrap_or("2");
        format!("{} copy {}", base_name, suffix)
    };

    file_io::write_json(&env_dir.join(format!("{}.json", new_file_name)), &env)?;

    // Copy secrets sidecar if it exists
    let old_secrets_path = env_dir.join(format!("{}.secrets.json", file_name));
    if old_secrets_path.exists() {
        let new_secrets_path = env_dir.join(format!("{}.secrets.json", new_file_name));
        std::fs::copy(&old_secrets_path, &new_secrets_path)
            .map_err(|e| FlupiError::Custom(format!("Failed to copy secrets: {}", e)))?;
    }

    Ok(new_file_name)
}

#[command]
pub fn rename_environment(
    project_path: PathBuf,
    file_name: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let env_dir = project_path.join("environments");
    let old_env_path = env_dir.join(format!("{}.json", file_name));

    let new_slug = name_to_slug(&new_name);
    let new_file_name = format!("{}.env.json", new_slug);
    let new_env_path = env_dir.join(format!("{}.json", new_file_name));

    let mut env: Environment = file_io::read_json(&old_env_path)?;
    env.name = new_name;
    file_io::write_json(&new_env_path, &env)?;
    if old_env_path != new_env_path {
        file_io::delete_file(&old_env_path)?;
    }

    // Rename secrets sidecar if it exists
    let old_secrets_path = env_dir.join(format!("{}.secrets.json", file_name));
    if old_secrets_path.exists() {
        let new_secrets_path = env_dir.join(format!("{}.secrets.json", new_file_name));
        if old_secrets_path != new_secrets_path {
            std::fs::rename(&old_secrets_path, &new_secrets_path)
                .map_err(|e| FlupiError::Custom(format!("Failed to rename secrets: {}", e)))?;
        }
    }

    Ok(new_file_name)
}
