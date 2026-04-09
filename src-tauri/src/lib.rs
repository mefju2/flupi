pub mod commands;
pub mod error;
pub mod models;
pub mod services;
pub mod utils;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state injected via Tauri's managed state system.
pub struct AppState {
    /// Serialises all reads/writes of `openapi-sources.json` so concurrent
    /// `refresh_source` calls (startup scan, "Sync All") cannot corrupt the file.
    pub sources_lock: Arc<Mutex<()>>,
    /// Ensures only one request or scenario executes at a time. Using a Mutex
    /// (rather than AtomicBool) guarantees the lock is always released when the
    /// guard is dropped, even on panic.
    pub execution_lock: Arc<Mutex<()>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sources_lock: Arc::new(Mutex::new(())),
            execution_lock: Arc::new(Mutex::new(())),
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::project::create_project,
            commands::project::open_project,
            commands::app_data::get_recent_projects,
            commands::app_data::add_recent_project,
            commands::app_data::remove_recent_project,
            commands::app_data::get_preferences,
            commands::app_data::save_preferences,
            commands::app_data::set_project_active_environment,
            commands::environment::list_environments,
            commands::environment::save_environment,
            commands::environment::save_secrets,
            commands::environment::get_resolved_variables,
            commands::environment::delete_environment,
            commands::environment::duplicate_environment,
            commands::environment::rename_environment,
            commands::request_tree::load_request_tree,
            commands::request::get_request,
            commands::request::save_request,
            commands::request::create_request,
            commands::request::delete_request,
            commands::request::rename_request,
            commands::request::move_request,
            commands::request::duplicate_request,
            commands::request::get_request_references,
            commands::collection::get_collection,
            commands::collection::create_collection,
            commands::collection::save_collection,
            commands::collection::delete_collection,
            commands::collection::rename_collection,
            commands::execution::send_request,
            commands::execution_runner::run_scenario,
            commands::scenario::load_scenario_tree,
            commands::scenario::get_scenario,
            commands::scenario::save_scenario,
            commands::scenario::create_scenario,
            commands::scenario::delete_scenario,
            commands::scenario::rename_scenario,
            commands::scenario::duplicate_scenario,
            commands::openapi::add_openapi_source,
            commands::openapi::remove_openapi_source,
            commands::openapi::list_openapi_sources,
            commands::openapi::fetch_operations,
            commands::openapi::import_operations,
            commands::openapi::refresh_source,
            commands::openapi::resolve_drift,
            commands::openapi::get_drift_details,
            commands::openapi::generate_body_from_schema,
            commands::functions::list_functions,
            commands::functions::save_function,
            commands::functions::delete_function,
            commands::functions::rename_function,
            commands::environment::rename_variable_key,
            commands::openapi::rename_openapi_source,
            commands::openapi::list_requests_by_source,
            commands::git::get_git_status,
            commands::git::git_fetch,
            commands::git::git_pull,
            commands::git::git_file_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
