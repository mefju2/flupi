use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;
use crate::AppState;
use crate::error::FlupiError;
use crate::services::{http_client, file_io, request_path, request_executor};

pub use request_executor::execute_single_request;

#[command]
pub async fn send_request(
    state: tauri::State<'_, AppState>,
    project_path: PathBuf,
    request_id: String,
    env_file_name: String,
    timeout_ms: u64,
    injected_vars: Option<HashMap<String, String>>,
) -> Result<http_client::HttpResponse, FlupiError> {
    // Read extractions once before the request is sent to avoid a race where
    // the user saves new extractions while a slow request is in flight.
    let req_path = request_path::resolve_request_path(&project_path, &request_id);
    let extractions = file_io::read_json::<crate::models::request::Request>(&req_path)
        .map(|r| r.extractions)
        .unwrap_or_default();

    let extra_vars: HashMap<String, String> = injected_vars.unwrap_or_default();

    let _guard = state.execution_lock.lock().await;
    let result = execute_single_request(&project_path, &request_id, &env_file_name, timeout_ms, &extra_vars, &HashMap::new()).await;

    if let Ok(ref response) = result {
        if !extractions.is_empty() {
            if let Err(e) = request_executor::apply_extractions_to_env(&project_path, &env_file_name, &extractions, response) {
                eprintln!("[flupi] extraction write failed: {e}");
            }
        }
    }

    result
}
