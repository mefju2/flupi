use std::path::Path;
use crate::error::Result;
use crate::models::openapi::ImportableOperation;
use crate::models::request::Request;
use crate::services::{file_io, openapi_import};

pub fn detect_drift(
    project_path: &Path,
    source_id: &str,
    operations: &[(ImportableOperation, serde_json::Value)],
    spec: &serde_json::Value,
) -> Result<Vec<String>> {
    let request_files = collect_request_files(project_path)?;
    let mut drifted = Vec::new();

    for file_path in &request_files {
        let request: Request = match file_io::read_json(file_path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let template_ref = match &request.template_ref {
            Some(t) => t,
            None => continue,
        };

        if template_ref.source_id != source_id {
            continue;
        }

        let found = operations
            .iter()
            .find(|(op, _)| op.operation_id == template_ref.operation_id);

        let is_drifted = match found {
            // Operation no longer exists in the spec → drift
            None => true,
            Some((current_op, op_json)) => {
                let current_hash = openapi_import::compute_sha256_hash(op_json);
                if current_hash != template_ref.schema_hash || current_op.path != request.path {
                    true
                } else {
                    // Secondary check: compare resolved schemas to catch $ref-based
                    // contract changes (e.g. a property added to a shared component).
                    let (new_req_schema, new_res_schema) =
                        openapi_import::extract_schemas(op_json, spec);
                    new_req_schema != template_ref.request_schema
                        || new_res_schema != template_ref.response_schema
                }
            }
        };

        if is_drifted {
            if let Ok(request_id) = crate::models::request::derive_request_id(project_path, file_path) {
                drifted.push(request_id);
            }
        }
    }

    Ok(drifted)
}

pub fn collect_request_files(project_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    // Collect from collections/{folder}/requests/
    let collections_dir = project_path.join("collections");
    if collections_dir.exists() {
        for entry in std::fs::read_dir(&collections_dir)? {
            let entry = entry?;
            let col_path = entry.path();
            if col_path.is_dir() {
                let requests_dir = col_path.join("requests");
                let mut col_files = file_io::list_json_files(&requests_dir)?;
                files.append(&mut col_files);
            }
        }
    }

    // Collect from root requests/
    let root_requests = project_path.join("requests");
    let mut root_files = file_io::list_json_files(&root_requests)?;
    files.append(&mut root_files);

    Ok(files)
}

#[cfg(test)]
#[path = "tests/drift_detection.rs"]
mod tests;
