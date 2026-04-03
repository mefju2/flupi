use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::models::script_function::ScriptFunction;
use crate::services::file_io;

fn functions_dir(project_path: &std::path::Path) -> std::path::PathBuf {
    project_path.join("functions")
}

fn validate_fn_name(name: &str) -> Result<(), FlupiError> {
    if name.is_empty() {
        return Err(FlupiError::Custom("Function name cannot be empty".to_string()));
    }
    let mut chars = name.chars();
    let first_ok = chars.next().map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$').unwrap_or(false);
    if !first_ok {
        return Err(FlupiError::Custom(format!(
            "Invalid function name \"{name}\": must start with a letter, _ or $"
        )));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') {
        return Err(FlupiError::Custom(format!(
            "Invalid function name \"{name}\": may only contain letters, digits, _ or $"
        )));
    }
    Ok(())
}

#[command]
pub fn list_functions(project_path: PathBuf) -> Result<Vec<ScriptFunction>, FlupiError> {
    let dir = functions_dir(&project_path);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut functions = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match file_io::read_json::<ScriptFunction>(&path) {
                Ok(f) => functions.push(f),
                Err(e) => eprintln!("[flupi] failed to read function {:?}: {e}", path),
            }
        }
    }

    functions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(functions)
}

#[command]
pub fn save_function(project_path: PathBuf, function: ScriptFunction) -> Result<(), FlupiError> {
    validate_fn_name(&function.name)?;
    let dir = functions_dir(&project_path);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", function.name));
    file_io::write_json(&path, &function)
}

#[command]
pub fn delete_function(project_path: PathBuf, name: String) -> Result<(), FlupiError> {
    validate_fn_name(&name)?;
    let path = functions_dir(&project_path).join(format!("{}.json", name));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/functions.rs"]
mod tests;
