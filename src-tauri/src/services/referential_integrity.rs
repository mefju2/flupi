use std::path::Path;
use crate::error::Result;
use crate::models::scenario::{Scenario, ScenarioStep};
use crate::services::file_io;

pub fn find_references(project_path: &Path, request_id: &str) -> Result<Vec<std::path::PathBuf>> {
    let scenarios_dir = project_path.join("scenarios");
    let scenario_files = file_io::list_json_files(&scenarios_dir)?;
    let mut referencing = Vec::new();

    for file in scenario_files {
        let scenario: Scenario = file_io::read_json(&file)?;
        if scenario.steps.iter().any(|s| matches!(s, ScenarioStep::Request(r) if r.request_id == request_id)) {
            referencing.push(file);
        }
    }

    Ok(referencing)
}

pub fn update_references(project_path: &Path, old_id: &str, new_id: &str) -> Result<()> {
    let scenarios_dir = project_path.join("scenarios");
    let scenario_files = file_io::list_json_files(&scenarios_dir)?;

    for file in scenario_files {
        let mut scenario: Scenario = file_io::read_json(&file)?;
        let mut modified = false;

        for step in &mut scenario.steps {
            if let ScenarioStep::Request(ref mut r) = step {
                if r.request_id == old_id {
                    r.request_id = new_id.to_string();
                    modified = true;
                }
            }
        }

        if modified {
            file_io::write_json(&file, &scenario)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/referential_integrity.rs"]
mod tests;
