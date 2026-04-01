use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Emitter};
use serde::Serialize;
use crate::error::FlupiError;
use crate::models::scenario::Scenario;
use crate::models::extraction::Extraction;
use crate::services::{file_io, http_client};
use super::execution::{acquire_lock, release_lock, execute_single_request};

#[derive(Debug, Serialize, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: String,
    pub response: Option<http_client::HttpResponse>,
    pub error: Option<String>,
    pub extracted: HashMap<String, String>,
}

pub fn apply_extraction(
    extraction: &Extraction,
    body: &str,
    headers: &HashMap<String, String>,
) -> Result<String, String> {
    if extraction.from == "response.body" {
        let json: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let path = serde_json_path::JsonPath::parse(&extraction.path)
            .map_err(|e| e.to_string())?;
        let nodes = path.query(&json);
        nodes.first()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .ok_or_else(|| format!("No match for path {}", extraction.path))
    } else {
        headers
            .get(&extraction.path)
            .cloned()
            .ok_or_else(|| format!("Header {} not found", extraction.path))
    }
}

pub fn apply_overrides(
    extra_vars: &mut HashMap<String, String>,
    overrides: &HashMap<String, String>,
) {
    for (k, v) in overrides {
        extra_vars.insert(k.clone(), v.clone());
    }
}

#[command]
pub async fn run_scenario(
    app: AppHandle,
    project_path: PathBuf,
    scenario_id: String,
    env_file_name: String,
    inputs: HashMap<String, String>,
    timeout_ms: u64,
) -> Result<(), FlupiError> {
    acquire_lock()?;

    let result = run_scenario_inner(
        &app,
        &project_path,
        &scenario_id,
        &env_file_name,
        inputs,
        timeout_ms,
    )
    .await;

    release_lock();
    result
}

async fn run_scenario_inner(
    app: &AppHandle,
    project_path: &Path,
    scenario_id: &str,
    env_file_name: &str,
    inputs: HashMap<String, String>,
    timeout_ms: u64,
) -> Result<(), FlupiError> {
    let scenario_path = project_path
        .join("scenarios")
        .join(format!("{}.json", scenario_id.replace('/', std::path::MAIN_SEPARATOR_STR)));

    let scenario: Scenario = file_io::read_json(&scenario_path)?;

    let mut extracted: HashMap<String, String> = inputs.clone();

    for step in &scenario.steps {
        let mut path_param_overrides: HashMap<String, String> = HashMap::new();
        let mut regular_overrides: HashMap<String, String> = HashMap::new();
        for (k, v) in &step.overrides {
            if let Some(param_name) = k.strip_prefix("path.") {
                path_param_overrides.insert(param_name.to_string(), v.clone());
            } else {
                regular_overrides.insert(k.clone(), v.clone());
            }
        }
        let mut extra_vars = extracted.clone();
        apply_overrides(&mut extra_vars, &regular_overrides);

        let response = execute_single_request(
            project_path,
            &step.request_id,
            env_file_name,
            timeout_ms,
            &extra_vars,
            &path_param_overrides,
        )
        .await;

        match response {
            Err(e) => {
                let result = StepResult {
                    step_id: step.id.clone(),
                    status: "error".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                    extracted: HashMap::new(),
                };
                let _ = app.emit("scenario-step-result", &result);
                return Err(e);
            }
            Ok(resp) => {
                let is_success = resp.status >= 200 && resp.status < 300;
                if !is_success {
                    let error_msg = format!("HTTP {} {}", resp.status, resp.status_text);
                    let result = StepResult {
                        step_id: step.id.clone(),
                        status: "error".to_string(),
                        response: Some(resp),
                        error: Some(error_msg.clone()),
                        extracted: HashMap::new(),
                    };
                    let _ = app.emit("scenario-step-result", &result);
                    return Err(FlupiError::Custom(error_msg));
                }

                let mut step_extracted: HashMap<String, String> = HashMap::new();
                for ext in &step.extract {
                    match apply_extraction(ext, &resp.body, &resp.headers) {
                        Ok(val) => {
                            step_extracted.insert(ext.variable.clone(), val.clone());
                            extracted.insert(ext.variable.clone(), val);
                        }
                        Err(e) => {
                            let result = StepResult {
                                step_id: step.id.clone(),
                                status: "error".to_string(),
                                response: Some(resp),
                                error: Some(e.clone()),
                                extracted: HashMap::new(),
                            };
                            let _ = app.emit("scenario-step-result", &result);
                            return Err(FlupiError::Custom(e));
                        }
                    }
                }

                let result = StepResult {
                    step_id: step.id.clone(),
                    status: "success".to_string(),
                    response: Some(resp),
                    error: None,
                    extracted: step_extracted,
                };
                let _ = app.emit("scenario-step-result", &result);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/execution.rs"]
mod tests;
