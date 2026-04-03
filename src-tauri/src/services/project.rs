use std::path::Path;
use crate::error::Result;

const PROJECT_DIRS: &[&str] = &["environments", "collections", "requests", "scenarios", "functions"];
const GITIGNORE_CONTENT: &str = "*.secrets.json\n";

struct BuiltinFunction {
    name: &'static str,
    body: &'static str,
}

const BUILTIN_FUNCTIONS: &[BuiltinFunction] = &[
    BuiltinFunction {
        name: "randomGuid",
        body: "// Returns a random UUID v4\nreturn crypto.randomUUID();",
    },
    BuiltinFunction {
        name: "randomInt",
        body: "// Returns a random integer in [args[0], args[1])\nconst min = parseInt(args[0] ?? '0', 10);\nconst max = parseInt(args[1] ?? '100', 10);\nreturn String(Math.floor(Math.random() * (max - min)) + min);",
    },
    BuiltinFunction {
        name: "randomFloat",
        body: "// Returns a random float between args[0] and args[1]\nconst min = parseFloat(args[0] ?? '0');\nconst max = parseFloat(args[1] ?? '1');\nreturn String(Math.random() * (max - min) + min);",
    },
    BuiltinFunction {
        name: "now",
        body: "// Returns the current date-time as an ISO 8601 string\nreturn new Date().toISOString();",
    },
    BuiltinFunction {
        name: "timestamp",
        body: "// Returns the current Unix timestamp in milliseconds\nreturn String(Date.now());",
    },
];

#[derive(Debug, PartialEq)]
pub enum ProjectState {
    Valid,
    Partial,
    Empty,
    NotFound,
}

pub fn init_project(path: &Path) -> Result<()> {
    for dir in PROJECT_DIRS {
        std::fs::create_dir_all(path.join(dir))?;
    }

    let gitignore_path = path.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, GITIGNORE_CONTENT)?;
    }

    seed_builtin_functions(path)?;

    Ok(())
}

fn seed_builtin_functions(path: &Path) -> Result<()> {
    let functions_dir = path.join("functions");
    for f in BUILTIN_FUNCTIONS {
        let file_path = functions_dir.join(format!("{}.json", f.name));
        if !file_path.exists() {
            let json = serde_json::json!({ "name": f.name, "body": f.body });
            std::fs::write(&file_path, serde_json::to_string_pretty(&json)?)?;
        }
    }
    Ok(())
}

pub fn validate_project(path: &Path) -> ProjectState {
    if !path.exists() {
        return ProjectState::NotFound;
    }

    let existing: Vec<bool> = PROJECT_DIRS
        .iter()
        .map(|d| path.join(d).is_dir())
        .collect();

    if existing.iter().all(|&e| e) {
        ProjectState::Valid
    } else if existing.iter().any(|&e| e) {
        ProjectState::Partial
    } else {
        ProjectState::Empty
    }
}

pub fn ensure_project_structure(path: &Path) -> Result<()> {
    for dir in PROJECT_DIRS {
        let dir_path = path.join(dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path)?;
        }
    }
    seed_builtin_functions(path)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/project.rs"]
mod tests;
