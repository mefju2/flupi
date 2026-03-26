pub mod error;
pub mod services;
pub mod commands;
pub mod models;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::project::create_project,
            commands::project::open_project,
            commands::app_data::get_recent_projects,
            commands::app_data::add_recent_project,
            commands::app_data::get_preferences,
            commands::app_data::save_preferences,
            commands::environment::list_environments,
            commands::environment::save_environment,
            commands::environment::save_secrets,
            commands::environment::get_resolved_variables,
            commands::environment::delete_environment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
