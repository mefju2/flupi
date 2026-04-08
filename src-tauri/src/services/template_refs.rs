use std::path::Path;
use crate::error::Result;

/// Performs a raw substring find-and-replace across all request and scenario files
/// in a project. Returns the number of files that were modified.
///
/// `search` and `replace` are literal strings (no regex). Each file is read as UTF-8,
/// all occurrences are replaced, and the file is written back only if the content changed.
pub fn update_template_references(project_path: &Path, search: &str, replace: &str) -> Result<usize> {
    if search == replace {
        return Ok(0);
    }

    let mut modified = 0;

    // Walk request files: requests/**/*.json and collections/*/requests/**/*.json
    let request_files = collect_all_request_files(project_path)?;
    for path in request_files {
        if apply_replacement(&path, search, replace)? {
            modified += 1;
        }
    }

    // Walk scenario files: scenarios/**/*.json
    let scenarios_dir = project_path.join("scenarios");
    if scenarios_dir.exists() {
        let scenario_files = crate::services::file_io::list_json_files(&scenarios_dir)?;
        for path in scenario_files {
            if apply_replacement(&path, search, replace)? {
                modified += 1;
            }
        }
    }

    Ok(modified)
}

fn apply_replacement(path: &Path, search: &str, replace: &str) -> Result<bool> {
    let original = std::fs::read_to_string(path)?;
    if !original.contains(search) {
        return Ok(false);
    }
    let updated = original.replace(search, replace);
    std::fs::write(path, updated.as_bytes())?;
    Ok(true)
}

fn collect_all_request_files(project_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    // Delegate to drift_detection for request files (collections + root requests).
    crate::services::drift_detection::collect_request_files(project_path)
}
