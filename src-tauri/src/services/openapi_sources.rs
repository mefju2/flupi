use std::path::Path;
use crate::error::Result;
use crate::models::openapi::{OpenApiSource, OpenApiSources};
use crate::services::file_io;

const SOURCES_FILE: &str = "openapi-sources.json";

pub fn load(project_path: &Path) -> Result<OpenApiSources> {
    let path = project_path.join(SOURCES_FILE);
    if !path.exists() {
        return Ok(OpenApiSources::default());
    }
    file_io::read_json(&path)
}

pub fn save(project_path: &Path, sources: &OpenApiSources) -> Result<()> {
    let path = project_path.join(SOURCES_FILE);
    file_io::write_json(&path, sources)
}

pub fn add(project_path: &Path, source: OpenApiSource) -> Result<()> {
    let mut sources = load(project_path)?;
    sources.sources.push(source);
    save(project_path, &sources)
}

pub fn remove(project_path: &Path, source_id: &str) -> Result<()> {
    let mut sources = load(project_path)?;
    sources.sources.retain(|s| s.id() != source_id);
    save(project_path, &sources)
}
