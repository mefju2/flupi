use std::path::{Path, PathBuf};

/// Resolves a request ID to an absolute file path.
///
/// Disambiguation strategy: if the ID contains a `/`, the first path segment is
/// checked against `collections/{first_segment}/` on disk. If that directory
/// exists, the request is treated as a collection request and maps to
/// `collections/{first_segment}/requests/{rest}.json`. Otherwise (the collection
/// directory does not exist), the entire ID is treated as a root request under
/// `requests/{id}.json`. This assumes collection folder names never collide with
/// root request sub-paths.
pub fn resolve_request_path(project_path: &Path, request_id: &str) -> PathBuf {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        let rest = parts[1];
        let collection_dir = project_path.join("collections").join(first);
        if collection_dir.exists() {
            let rest_path = rest.replace('/', std::path::MAIN_SEPARATOR_STR);
            return project_path
                .join("collections")
                .join(first)
                .join("requests")
                .join(format!("{}.json", rest_path));
        }
    }
    let rest_path = request_id.replace('/', std::path::MAIN_SEPARATOR_STR);
    project_path.join("requests").join(format!("{}.json", rest_path))
}

/// Returns the collection folder name for a request ID, or `None` if the
/// request lives at the project root.
pub fn collection_folder_for(request_id: &str, project_path: &Path) -> Option<String> {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        if project_path.join("collections").join(first).exists() {
            return Some(first.to_string());
        }
    }
    None
}
