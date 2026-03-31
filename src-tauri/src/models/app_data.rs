use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    #[serde(rename = "lastOpenedAt")]
    pub last_opened_at: String,
    #[serde(rename = "activeEnvironment", default)]
    pub active_environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RecentProjects {
    pub projects: Vec<RecentProject>,
}

impl RecentProjects {
    pub fn add(&mut self, name: &str, path: &str) {
        let existing_active_env = self.projects.iter()
            .find(|p| p.path == path)
            .and_then(|p| p.active_environment.clone());
        self.projects.retain(|p| p.path != path);
        self.projects.insert(0, RecentProject {
            name: name.to_string(),
            path: path.to_string(),
            last_opened_at: Utc::now().to_rfc3339(),
            active_environment: existing_active_env,
        });
    }

    pub fn remove(&mut self, path: &str) {
        self.projects.retain(|p| p.path != path);
    }

    pub fn update_active_environment(&mut self, path: &str, env_file_name: Option<&str>) {
        if let Some(project) = self.projects.iter_mut().find(|p| p.path == path) {
            project.active_environment = env_file_name.map(|s| s.to_string());
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub theme: String,
    #[serde(rename = "defaultTimeoutMs")]
    pub default_timeout_ms: u64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_timeout_ms: 30000,
        }
    }
}

#[cfg(test)]
#[path = "tests/app_data.rs"]
mod tests;
