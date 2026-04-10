use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::services::file_io;

const FETCH_TIMES_FILE: &str = "openapi-fetch-times.json";

/// Outer key: canonical project path string.
/// Inner key: source ID.
/// Value: RFC 3339 timestamp of last successful fetch.
type FetchTimesMap = HashMap<String, HashMap<String, String>>;

fn load(app_data_dir: &Path) -> Result<FetchTimesMap> {
    let path = app_data_dir.join(FETCH_TIMES_FILE);
    if path.exists() {
        file_io::read_json(&path)
    } else {
        Ok(FetchTimesMap::default())
    }
}

fn save(app_data_dir: &Path, map: &FetchTimesMap) -> Result<()> {
    let path = app_data_dir.join(FETCH_TIMES_FILE);
    file_io::write_json(&path, map)
}

pub fn get(app_data_dir: &Path, project_path: &Path, source_id: &str) -> Result<Option<String>> {
    let map = load(app_data_dir)?;
    let key = project_path.to_string_lossy().to_string();
    Ok(map
        .get(&key)
        .and_then(|inner| inner.get(source_id))
        .cloned())
}

pub fn set(
    app_data_dir: &Path,
    project_path: &Path,
    source_id: &str,
    timestamp: &str,
) -> Result<()> {
    let mut map = load(app_data_dir)?;
    let key = project_path.to_string_lossy().to_string();
    map.entry(key)
        .or_default()
        .insert(source_id.to_string(), timestamp.to_string());
    save(app_data_dir, &map)
}
