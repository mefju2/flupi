use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};
use crate::error::FlupiError;
use crate::models::app_data::{RecentProjects, Preferences};
use crate::services::file_io;

fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("failed to get app data dir")
}

#[command]
pub fn get_recent_projects(app: AppHandle) -> Result<RecentProjects, FlupiError> {
    let path = app_data_dir(&app).join("recent-projects.json");
    if path.exists() {
        file_io::read_json(&path)
    } else {
        Ok(RecentProjects::default())
    }
}

#[command]
pub fn add_recent_project(app: AppHandle, name: String, path: String) -> Result<(), FlupiError> {
    let file_path = app_data_dir(&app).join("recent-projects.json");
    let mut projects = if file_path.exists() {
        file_io::read_json(&file_path)?
    } else {
        RecentProjects::default()
    };
    projects.add(&name, &path);
    file_io::write_json(&file_path, &projects)
}

#[command]
pub fn get_preferences(app: AppHandle) -> Result<Preferences, FlupiError> {
    let path = app_data_dir(&app).join("preferences.json");
    if path.exists() {
        file_io::read_json(&path)
    } else {
        Ok(Preferences::default())
    }
}

#[command]
pub fn save_preferences(app: AppHandle, preferences: Preferences) -> Result<(), FlupiError> {
    let path = app_data_dir(&app).join("preferences.json");
    file_io::write_json(&path, &preferences)
}
