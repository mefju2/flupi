use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::models::environment::{self, Environment};
use crate::services::file_io;

#[command]
pub fn list_environments(project_path: PathBuf) -> Result<Vec<(String, Environment)>, FlupiError> {
    environment::list_environments(&project_path)
}

#[command]
pub fn save_environment(project_path: PathBuf, file_name: String, env: Environment) -> Result<(), FlupiError> {
    let path = project_path.join("environments").join(format!("{}.json", file_name));
    file_io::write_json(&path, &env)
}

#[command]
pub fn save_secrets(project_path: PathBuf, file_name: String, secrets: HashMap<String, String>) -> Result<(), FlupiError> {
    let path = project_path.join("environments").join(format!("{}.secrets.json", file_name));
    file_io::write_json(&path, &secrets)
}

#[command]
pub fn get_resolved_variables(project_path: PathBuf, file_name: String) -> Result<HashMap<String, String>, FlupiError> {
    let path = project_path.join("environments").join(format!("{}.json", file_name));
    environment::resolve_env_variables(&path)
}

#[command]
pub fn delete_environment(project_path: PathBuf, file_name: String) -> Result<(), FlupiError> {
    let env_path = project_path.join("environments").join(format!("{}.json", file_name));
    let secrets_path = project_path.join("environments").join(format!("{}.secrets.json", file_name));
    file_io::delete_file(&env_path)?;
    if secrets_path.exists() {
        file_io::delete_file(&secrets_path)?;
    }
    Ok(())
}
