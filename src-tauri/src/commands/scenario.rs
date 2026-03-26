use std::path::{Path, PathBuf};
use tauri::command;
use serde::Serialize;
use crate::error::FlupiError;
use crate::models::scenario::Scenario;
use crate::services::file_io;
use crate::utils::name_to_slug;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ScenarioTreeNode {
    Scenario {
        id: String,
        name: String,
    },
    Group {
        name: String,
        children: Vec<ScenarioTreeNode>,
    },
}

fn scan_scenarios_dir(
    scenarios_root: &Path,
    dir: &Path,
) -> Result<Vec<ScenarioTreeNode>, FlupiError> {
    let mut nodes: Vec<ScenarioTreeNode> = Vec::new();

    if !dir.exists() {
        return Ok(nodes);
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let group_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let children = scan_scenarios_dir(scenarios_root, &path)?;
            nodes.push(ScenarioTreeNode::Group {
                name: group_name,
                children,
            });
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let scenario: Scenario = file_io::read_json(&path)?;
            let rel = path.strip_prefix(scenarios_root)
                .map_err(|_| FlupiError::Custom("path prefix error".to_string()))?;
            let id = rel.with_extension("").to_string_lossy().replace('\\', "/");
            nodes.push(ScenarioTreeNode::Scenario {
                id,
                name: scenario.name,
            });
        }
    }

    Ok(nodes)
}

#[command]
pub fn load_scenario_tree(project_path: PathBuf) -> Result<Vec<ScenarioTreeNode>, FlupiError> {
    let scenarios_dir = project_path.join("scenarios");
    scan_scenarios_dir(&scenarios_dir, &scenarios_dir)
}

fn scenario_path(project_path: &Path, scenario_id: &str) -> PathBuf {
    let rel = scenario_id.replace('/', std::path::MAIN_SEPARATOR_STR);
    project_path.join("scenarios").join(format!("{}.json", rel))
}

#[command]
pub fn get_scenario(project_path: PathBuf, scenario_id: String) -> Result<Scenario, FlupiError> {
    let path = scenario_path(&project_path, &scenario_id);
    file_io::read_json(&path)
}

#[command]
pub fn save_scenario(
    project_path: PathBuf,
    scenario_id: String,
    scenario: Scenario,
) -> Result<(), FlupiError> {
    let path = scenario_path(&project_path, &scenario_id);
    file_io::write_json(&path, &scenario)
}

#[command]
pub fn create_scenario(
    project_path: PathBuf,
    group: Option<String>,
    name: String,
) -> Result<String, FlupiError> {
    let slug = name_to_slug(&name);
    let (path, id) = match &group {
        Some(g) => {
            let rel = format!("{}/{}", g, slug);
            let p = scenario_path(&project_path, &rel);
            (p, rel)
        }
        None => {
            let p = scenario_path(&project_path, &slug);
            (p, slug.clone())
        }
    };

    let scenario = Scenario {
        name,
        inputs: Vec::new(),
        steps: Vec::new(),
    };
    file_io::write_json(&path, &scenario)?;
    Ok(id)
}

#[command]
pub fn delete_scenario(project_path: PathBuf, scenario_id: String) -> Result<(), FlupiError> {
    let path = scenario_path(&project_path, &scenario_id);
    file_io::delete_file(&path)
}

#[command]
pub fn rename_scenario(
    project_path: PathBuf,
    scenario_id: String,
    new_name: String,
) -> Result<String, FlupiError> {
    let old_path = scenario_path(&project_path, &scenario_id);
    let new_slug = name_to_slug(&new_name);

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;
    let new_path = parent.join(format!("{}.json", new_slug));

    let mut scenario: Scenario = file_io::read_json(&old_path)?;
    scenario.name = new_name;
    file_io::write_json(&new_path, &scenario)?;
    if old_path != new_path {
        file_io::delete_file(&old_path)?;
    }

    let scenarios_root = project_path.join("scenarios");
    let rel = new_path.strip_prefix(&scenarios_root)
        .map_err(|_| FlupiError::Custom("path prefix error".to_string()))?;
    let new_id = rel.with_extension("").to_string_lossy().replace('\\', "/");
    Ok(new_id)
}

#[command]
pub fn duplicate_scenario(
    project_path: PathBuf,
    scenario_id: String,
) -> Result<String, FlupiError> {
    let old_path = scenario_path(&project_path, &scenario_id);
    let mut scenario: Scenario = file_io::read_json(&old_path)?;

    let stem = old_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FlupiError::Custom("Cannot determine file stem".to_string()))?
        .to_string();

    let parent = old_path
        .parent()
        .ok_or_else(|| FlupiError::Custom("Cannot determine parent directory".to_string()))?;

    let new_path = {
        let candidate = parent.join(format!("{}-copy.json", stem));
        if !candidate.exists() {
            candidate
        } else {
            let mut found: Option<PathBuf> = None;
            for n in 2..=10 {
                let c = parent.join(format!("{}-copy-{}.json", stem, n));
                if !c.exists() {
                    found = Some(c);
                    break;
                }
            }
            found.ok_or_else(|| FlupiError::Custom("duplicate already exists".to_string()))?
        }
    };

    let fallback_stem = format!("{}-copy", stem);
    let copy_stem = new_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&fallback_stem);
    let base_name = scenario.name.clone();
    scenario.name = if copy_stem.ends_with("-copy") {
        format!("{} copy", base_name)
    } else {
        let suffix = copy_stem.rsplit('-').next().unwrap_or("2");
        format!("{} copy {}", base_name, suffix)
    };

    file_io::write_json(&new_path, &scenario)?;

    let scenarios_root = project_path.join("scenarios");
    let rel = new_path.strip_prefix(&scenarios_root)
        .map_err(|_| FlupiError::Custom("path prefix error".to_string()))?;
    let new_id = rel.with_extension("").to_string_lossy().replace('\\', "/");
    Ok(new_id)
}

#[cfg(test)]
#[path = "tests/scenario.rs"]
mod tests;
