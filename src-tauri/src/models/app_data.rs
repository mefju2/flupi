use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    #[serde(rename = "lastOpenedAt")]
    pub last_opened_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RecentProjects {
    pub projects: Vec<RecentProject>,
}

impl RecentProjects {
    pub fn add(&mut self, name: &str, path: &str) {
        self.projects.retain(|p| p.path != path);
        self.projects.insert(0, RecentProject {
            name: name.to_string(),
            path: path.to_string(),
            last_opened_at: Utc::now().to_rfc3339(),
        });
    }

    pub fn remove(&mut self, path: &str) {
        self.projects.retain(|p| p.path != path);
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
