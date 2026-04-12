use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Emitter};
use serde::Serialize;
use crate::AppState;
use crate::error::FlupiError;
use crate::models::scenario::{Scenario, ScenarioStep};
use crate::services::{file_io, http_client, variable_resolver, request_executor};
use super::execution::execute_single_request;
pub use request_executor::apply_extraction;

#[derive(Debug, Serialize, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: String,
    pub response: Option<http_client::HttpResponse>,
    pub error: Option<String>,
    pub extracted: HashMap<String, String>,
    pub sent_request: Option<http_client::ExecutableRequest>,
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
    state: tauri::State<'_, AppState>,
    app: AppHandle,
    project_path: PathBuf,
    scenario_id: String,
    env_file_name: String,
    inputs: HashMap<String, String>,
    timeout_ms: u64,
    injected_vars: Option<HashMap<String, String>>,
) -> Result<(), FlupiError> {
    let _guard = state.execution_lock.lock().await;
    run_scenario_inner(
        &app,
        &project_path,
        &scenario_id,
        &env_file_name,
        inputs,
        timeout_ms,
        injected_vars.unwrap_or_default(),
    )
    .await
}

fn status_is_expected(status: u16, expected: &[String]) -> bool {
    if expected.is_empty() {
        return status >= 200 && status < 300;
    }
    let s = status.to_string();
    expected.iter().any(|pattern| {
        pattern.len() == s.len()
            && pattern.chars().zip(s.chars()).all(|(p, c)| p == '*' || p == c)
    })
}

async fn run_scenario_inner(
    app: &AppHandle,
    project_path: &Path,
    scenario_id: &str,
    env_file_name: &str,
    inputs: HashMap<String, String>,
    timeout_ms: u64,
    injected_vars: HashMap<String, String>,
) -> Result<(), FlupiError> {
    let scenario_path = project_path
        .join("scenarios")
        .join(format!("{}.json", scenario_id.replace('/', std::path::MAIN_SEPARATOR_STR)));

    let scenario: Scenario = file_io::read_json(&scenario_path)?;

    // Seed injected vars (pre-evaluated function results) into the initial
    // extracted map so they are available throughout the entire scenario run.
    let mut extracted: HashMap<String, String> = injected_vars;
    for (k, v) in inputs {
        extracted.insert(k, v);
    }

    for step in &scenario.steps {
        match step {
            ScenarioStep::Delay(delay_step) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_step.duration)).await;
                let result = StepResult {
                    step_id: delay_step.id.clone(),
                    status: "success".to_string(),
                    response: None,
                    error: None,
                    extracted: HashMap::new(),
                    sent_request: None,
                };
                let _ = app.emit("scenario-step-result", &result);
            }
            ScenarioStep::Request(step) => {
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
                // Resolve override values against the current context so function-call
                // tokens like {{$randomInt(12)}} are expanded before being stored as vars.
                let pre_ctx = variable_resolver::build_context(
                    HashMap::new(),
                    &[],
                    None,
                    Some(&extracted),
                );
                let resolved_overrides: HashMap<String, String> = regular_overrides
                    .iter()
                    .map(|(k, v)| (k.clone(), variable_resolver::resolve_string(v, &pre_ctx)))
                    .collect();
                apply_overrides(&mut extra_vars, &resolved_overrides);

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
                            sent_request: None,
                        };
                        let _ = app.emit("scenario-step-result", &result);
                        return Err(e);
                    }
                    Ok((sent_req, resp)) => {
                        let is_success = status_is_expected(resp.status, &step.expected_status);
                        if !is_success {
                            let error_msg = format!("HTTP {} {}", resp.status, resp.status_text);
                            let result = StepResult {
                                step_id: step.id.clone(),
                                status: "error".to_string(),
                                response: Some(resp),
                                error: Some(error_msg.clone()),
                                extracted: HashMap::new(),
                                sent_request: Some(sent_req),
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
                                        sent_request: Some(sent_req),
                                    };
                                    let _ = app.emit("scenario-step-result", &result);
                                    return Err(FlupiError::Custom(e));
                                }
                            }
                        }

                        // Persist env-scoped extractions to the environment file
                        let env_exts: Vec<_> = step.extract.iter()
                            .filter(|e| e.scope != "scenario")
                            .cloned()
                            .collect();
                        if !env_exts.is_empty() {
                            if let Err(e) = request_executor::apply_extractions_to_env(
                                project_path, env_file_name, &env_exts, &resp,
                            ) {
                                eprintln!("[flupi] scenario extraction write failed: {e}");
                            }
                        }

                        let result = StepResult {
                            step_id: step.id.clone(),
                            status: "success".to_string(),
                            response: Some(resp),
                            error: None,
                            extracted: step_extracted,
                            sent_request: Some(sent_req),
                        };
                        let _ = app.emit("scenario-step-result", &result);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/execution.rs"]
mod tests;
