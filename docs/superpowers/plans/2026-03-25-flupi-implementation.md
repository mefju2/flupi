# Flupi Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Flupi — a Tauri desktop app for creating and running API test scenarios with Git-native project storage.

**Architecture:** Tauri v2 (Rust backend) + Svelte 5 + TypeScript frontend. All project data stored as JSON files on disk. Rust handles file I/O, HTTP execution, and project management via Tauri commands. Frontend handles UI state, variable resolution preview, and intellisense.

**Tech Stack:** Tauri v2, Rust, reqwest, serde, SvelteKit (static adapter, SSR disabled), Svelte 5, TypeScript, Tailwind CSS, shadcn-svelte, jsonpath-plus, svelte-dnd-action

**Coding conventions:** See `CLAUDE.md` — Coding Conventions section.

**Specs:**
- Main spec: `flupi-spec.md`
- Addendum: `docs/superpowers/specs/2026-03-25-flupi-spec-review-design.md`

---

## File Structure

### Rust Backend (`src-tauri/src/`)

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs                          ← Tauri entry, registers all commands
│   ├── lib.rs                           ← Module declarations
│   ├── error.rs                         ← Unified error type for all commands
│   ├── models/
│   │   ├── mod.rs
│   │   ├── environment.rs               ← Environment, Secrets structs
│   │   ├── request.rs                   ← Request, AuthConfig, BodyConfig
│   │   ├── collection.rs                ← Collection struct
│   │   ├── scenario.rs                  ← Scenario, Step, Extraction
│   │   ├── openapi.rs                   ← OpenApiSource, ImportedOperation
│   │   ├── variable.rs                  ← VariableContext, resolution types
│   │   └── app_data.rs                  ← RecentProjects, Preferences
│   ├── services/
│   │   ├── mod.rs
│   │   ├── file_io.rs                   ← Generic JSON read/write helpers
│   │   ├── project.rs                   ← Project init, validate, structure
│   │   ├── variable_resolver.rs         ← Variable resolution engine
│   │   ├── inheritance.rs               ← Collection → request inheritance
│   │   ├── http_client.rs               ← reqwest wrapper, timeout handling
│   │   ├── referential_integrity.rs     ← Update requestIds on rename/move
│   │   ├── openapi_import.rs            ← Fetch, parse, generate requests
│   │   └── drift_detection.rs           ← Hash comparison, drift state
│   └── commands/
│       ├── mod.rs
│       ├── project.rs                   ← open, create, validate, recent projects
│       ├── environment.rs               ← CRUD, secrets, switch active
│       ├── request.rs                   ← CRUD, rename, move, duplicate
│       ├── collection.rs                ← CRUD, rename
│       ├── scenario.rs                  ← CRUD, rename, move, duplicate
│       ├── execution.rs                 ← send_request, run_scenario (with events)
│       ├── openapi.rs                   ← add/remove source, import, refresh
│       └── app_data.rs                  ← preferences, recent projects
```

### SvelteKit Frontend (`src/`)

SvelteKit with `@sveltejs/adapter-static` and `ssr: false`. File-based routing replaces the manual `activeSection` store. Navigation uses SvelteKit's `goto()`. The `$lib` alias is provided natively — no Vite config needed.

```
src/
├── app.css                              ← Tailwind + global styles, dark mode
├── lib/
│   ├── stores/
│   │   ├── project.ts                   ← Active project state
│   │   ├── environment.ts               ← Environments, active env, secrets
│   │   ├── requests.ts                  ← Request tree, active request
│   │   ├── collections.ts               ← Collection data
│   │   ├── scenarios.ts                 ← Scenario tree, active scenario
│   │   ├── execution.ts                 ← Execution state, lock flag
│   │   ├── openapi.ts                   ← Sources, drift state
│   │   └── ui.ts                        ← Theme only (dark/light/system)
│   ├── services/
│   │   ├── tauri-commands.ts            ← Typed wrappers for all invoke() calls
│   │   ├── variable-resolver.ts         ← Frontend variable resolution for preview
│   │   ├── debounced-save.ts            ← 500ms debounce auto-save utility
│   │   └── keyboard-shortcuts.ts        ← Global shortcut registration
│   ├── components/
│   │   ├── layout/
│   │   │   ├── TopBar.svelte            ← Logo, project name, env switcher
│   │   │   └── Sidebar.svelte           ← Section navigation icons (uses $page for active)
│   │   ├── environments/
│   │   │   ├── EnvironmentList.svelte   ← Env list with active indicator
│   │   │   └── EnvironmentEditor.svelte ← Key-value editor with secret toggle
│   │   ├── requests/
│   │   │   ├── RequestTree.svelte       ← Sidebar tree with DnD
│   │   │   ├── RequestEditor.svelte     ← Tab container (≤150 lines — delegates to tabs)
│   │   │   ├── ParamsTab.svelte         ← URL params key-value table
│   │   │   ├── HeadersTab.svelte        ← Headers with inherited badges
│   │   │   ├── AuthTab.svelte           ← Auth type selector + fields
│   │   │   ├── BodyTab.svelte           ← Body type selector + editor
│   │   │   ├── SchemaTab.svelte         ← Schema tree + drift panel
│   │   │   ├── EffectiveRequestTab.svelte ← Read-only resolved preview
│   │   │   ├── ResponsePanel.svelte     ← Status, headers, body display
│   │   │   └── CollectionEditor.svelte  ← Collection-level settings
│   │   ├── scenarios/
│   │   │   ├── ScenarioTree.svelte      ← Sidebar tree with groups
│   │   │   ├── ScenarioEditor.svelte    ← Inputs + steps (delegates to sub-components)
│   │   │   ├── InputsList.svelte        ← Scenario-level input definitions
│   │   │   ├── StepCard.svelte          ← Collapsible step with DnD handle
│   │   │   ├── OverridesPanel.svelte    ← Key-value overrides for a step
│   │   │   ├── ExtractionsPanel.svelte  ← Extraction definitions for a step
│   │   │   ├── ScenarioRunner.svelte    ← Wrapper: pre-run form OR execution view
│   │   │   ├── PreRunForm.svelte        ← Input form shown before run
│   │   │   ├── RunnerStepper.svelte     ← Vertical step progress cards
│   │   │   ├── StepResultCard.svelte    ← Single step result (request + response detail)
│   │   │   └── VariableStatePanel.svelte ← Live variable context sidebar
│   │   ├── openapi/
│   │   │   ├── SourceList.svelte        ← Registered sources
│   │   │   ├── AddSourceForm.svelte     ← URL/file/DnD input
│   │   │   ├── ImportWizard.svelte      ← Operation selection grouped by tag
│   │   │   └── DriftPanel.svelte        ← Side-by-side schema diff
│   │   ├── settings/
│   │   │   └── SettingsPanel.svelte     ← Project + app settings
│   │   └── shared/
│   │       ├── KeyValueTable.svelte     ← Reusable key-value editor row list
│   │       ├── ContextMenu.svelte       ← Right-click menu wrapper
│   │       ├── VariableAutocomplete.svelte ← {{variable}} token autocomplete
│   │       ├── JsonEditor.svelte        ← Code editor for JSON body
│   │       └── TreeNode.svelte          ← Recursive tree node (used by Request/ScenarioTree)
│   └── utils/
│       ├── request-id.ts                ← Derive request ID from path
│       ├── jsonpath.ts                  ← jsonpath-plus wrapper
│       └── format.ts                    ← Response formatting, truncation
└── routes/
    ├── +layout.ts                       ← export const ssr = false; export const prerender = true;
    ├── +layout.svelte                   ← Root layout — applies theme class to <html>
    ├── +page.svelte                     ← Project picker screen
    └── (app)/
        ├── +layout.svelte               ← App shell: TopBar + Sidebar + <slot>
        ├── environments/
        │   └── +page.svelte             ← EnvironmentList + EnvironmentEditor
        ├── requests/
        │   └── +page.svelte             ← RequestTree + RequestEditor
        ├── scenarios/
        │   └── +page.svelte             ← ScenarioTree + ScenarioEditor/Runner
        ├── openapi/
        │   └── +page.svelte             ← SourceList + ImportWizard
        └── settings/
            └── +page.svelte             ← SettingsPanel
```

---

## Phase 1: Project Scaffolding

### Task 1.1: Initialize Tauri + Svelte Project

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src/app.css`, `src/routes/+layout.ts`, `src/routes/+layout.svelte`
- Create: `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`

- [ ] **Step 1: Create Tauri v2 project with SvelteKit template**

```bash
npm create tauri-app@latest flupi-scaffold -- --template sveltekit-ts
```

Copy the generated files into the project root. This gives us the base Tauri + SvelteKit + TypeScript + Vite setup.

- [ ] **Step 2: Install frontend dependencies**

```bash
npm install tailwindcss @tailwindcss/vite
npm install -D @tailwindcss/typography
npx shadcn-svelte@latest init
npm install svelte-dnd-action jsonpath-plus
npm install -D @types/jsonpath-plus
```

- [ ] **Step 3: Configure SvelteKit with static adapter and Tailwind**

Update `svelte.config.js`:
```js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
  },
};

export default config;
```

Create `src/routes/+layout.ts` to disable SSR globally:
```ts
export const ssr = false;
export const prerender = true;
```

In `src/app.css`:
```css
@import "tailwindcss";
```

In `vite.config.ts` (SvelteKit handles `$lib` alias automatically — no manual alias needed):
```ts
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

export default {
  plugins: [sveltekit(), tailwindcss()],
};
```

- [ ] **Step 4: Configure dark mode default**

Create `src/routes/+layout.svelte` — the root layout applies the theme class:
```svelte
<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';

  onMount(() => {
    // Default to dark mode (overridden by saved preference later)
    document.documentElement.classList.add('dark');
  });
</script>

<slot />
```

- [ ] **Step 5: Add Rust dependencies to Cargo.toml**

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "stream"] }
futures-util = "0.3"
tokio = { version = "1", features = ["full"] }
sha2 = "0.10"
hex = "0.4"
thiserror = "2"
regex = "1"
chrono = { version = "0.4", features = ["serde"] }
indexmap = { version = "2", features = ["serde"] }
serde_json_path = "0.7"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 6: Verify the app builds and launches**

```bash
npm run tauri dev
```

Expected: A blank Tauri window opens with the Svelte app loaded.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: initialize Tauri v2 + Svelte + TypeScript project scaffold"
```

---

### Task 1.2: Rust Error Type and File I/O Service

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/services/mod.rs`
- Create: `src-tauri/src/services/file_io.rs`

- [ ] **Step 1: Write tests for file I/O helpers**

Create `src-tauri/src/services/file_io.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_json_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        std::fs::write(&path, r#"{"name": "test"}"#).unwrap();

        let result: serde_json::Value = read_json(&path).unwrap();
        assert_eq!(result["name"], "test");
    }

    #[test]
    fn test_read_json_file_not_found() {
        let result: Result<serde_json::Value, _> = read_json(Path::new("/nonexistent.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_json_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.json");
        let data = serde_json::json!({"key": "value"});

        write_json(&path, &data).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_write_json_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sub/dir/file.json");
        let data = serde_json::json!({"nested": true});

        write_json(&path, &data).unwrap();
        assert!(path.exists());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test services::file_io
```

Expected: Compilation errors — functions don't exist yet.

- [ ] **Step 3: Implement error type**

Create `src-tauri/src/error.rs`:

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum FlupiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Custom(String),
}

impl Serialize for FlupiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, FlupiError>;
```

- [ ] **Step 4: Implement file I/O helpers**

In `src-tauri/src/services/file_io.rs`:

```rust
use std::path::Path;
use serde::{de::DeserializeOwned, Serialize};
use crate::error::Result;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn delete_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn list_json_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if dir.exists() {
        collect_json_files(dir, &mut files)?;
    }
    Ok(files)
}

fn collect_json_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    Ok(())
}
```

- [ ] **Step 5: Wire up modules in lib.rs**

```rust
pub mod error;
pub mod services;
```

`src-tauri/src/services/mod.rs`:
```rust
pub mod file_io;
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cd src-tauri && cargo test services::file_io
```

Expected: All 4 tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/services/ src-tauri/src/lib.rs
git commit -m "feat: add error type and file I/O service with tests"
```

---

### Task 1.3: Project Initialization and Validation

**Files:**
- Create: `src-tauri/src/services/project.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/project.rs`

- [ ] **Step 1: Write tests for project service**

In `src-tauri/src/services/project.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_project_creates_structure() {
        let dir = TempDir::new().unwrap();
        init_project(dir.path()).unwrap();

        assert!(dir.path().join(".gitignore").exists());
        assert!(dir.path().join("environments").is_dir());
        assert!(dir.path().join("collections").is_dir());
        assert!(dir.path().join("requests").is_dir());
        assert!(dir.path().join("scenarios").is_dir());
    }

    #[test]
    fn test_gitignore_contains_secrets_pattern() {
        let dir = TempDir::new().unwrap();
        init_project(dir.path()).unwrap();

        let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(gitignore.contains("*.secrets.json"));
    }

    #[test]
    fn test_gitignore_not_overwritten_if_exists() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "custom\n").unwrap();
        init_project(dir.path()).unwrap();

        let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(gitignore, "custom\n");
    }

    #[test]
    fn test_validate_project_full_structure() {
        let dir = TempDir::new().unwrap();
        init_project(dir.path()).unwrap();

        let result = validate_project(dir.path());
        assert_eq!(result, ProjectState::Valid);
    }

    #[test]
    fn test_validate_project_empty_dir() {
        let dir = TempDir::new().unwrap();
        let result = validate_project(dir.path());
        assert_eq!(result, ProjectState::Empty);
    }

    #[test]
    fn test_validate_project_partial() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("environments")).unwrap();
        let result = validate_project(dir.path());
        assert_eq!(result, ProjectState::Partial);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test services::project
```

- [ ] **Step 3: Implement project service**

```rust
use std::path::Path;
use crate::error::Result;

const PROJECT_DIRS: &[&str] = &["environments", "collections", "requests", "scenarios"];

const GITIGNORE_CONTENT: &str = "*.secrets.json\n";

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
    Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test services::project
```

Expected: All 6 tests pass.

- [ ] **Step 5: Create Tauri commands for project operations**

In `src-tauri/src/commands/project.rs`:

```rust
use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::services::project::{self, ProjectState};

#[command]
pub fn create_project(path: PathBuf) -> Result<(), FlupiError> {
    project::init_project(&path)
}

#[command]
pub fn open_project(path: PathBuf) -> Result<String, FlupiError> {
    match project::validate_project(&path) {
        ProjectState::Valid => Ok("valid".to_string()),
        ProjectState::Partial => {
            project::ensure_project_structure(&path)?;
            Ok("partial_fixed".to_string())
        }
        ProjectState::Empty => Ok("empty".to_string()),
        ProjectState::NotFound => Err(FlupiError::Custom(
            "Project folder not found".to_string(),
        )),
    }
}
```

- [ ] **Step 6: Register commands in main.rs**

```rust
mod commands;
mod error;
mod services;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::project::create_project,
            commands::project::open_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/project.rs src-tauri/src/commands/
git commit -m "feat: add project init, validation, and Tauri commands"
```

---

### Task 1.4: App Data — Recent Projects and Preferences

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/app_data.rs`
- Create: `src-tauri/src/commands/app_data.rs`

- [ ] **Step 1: Write tests for app data models**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_projects_add_and_sort() {
        let mut rp = RecentProjects { projects: vec![] };
        rp.add("Test", "/path/to/test");
        rp.add("Other", "/path/to/other");

        assert_eq!(rp.projects.len(), 2);
        // Most recent first
        assert_eq!(rp.projects[0].name, "Other");
    }

    #[test]
    fn test_recent_projects_dedup_by_path() {
        let mut rp = RecentProjects { projects: vec![] };
        rp.add("Test", "/path/to/test");
        rp.add("Test Updated", "/path/to/test");

        assert_eq!(rp.projects.len(), 1);
        assert_eq!(rp.projects[0].name, "Test Updated");
    }

    #[test]
    fn test_recent_projects_remove() {
        let mut rp = RecentProjects { projects: vec![] };
        rp.add("Test", "/path/to/test");
        rp.remove("/path/to/test");

        assert_eq!(rp.projects.len(), 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement app data models**

```rust
use chrono::{DateTime, Utc};
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
    pub theme: String,           // "dark" | "light" | "system"
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
```

Note: `chrono` is already in Cargo.toml from Task 1.1.

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Implement Tauri commands for app data**

In `src-tauri/src/commands/app_data.rs`:

```rust
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};
use crate::error::FlupiError;
use crate::models::app_data::{RecentProjects, Preferences};
use crate::services::file_io;

fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("failed to get app data dir")
}

#[command]
pub fn get_recent_projects(app: AppHandle) -> Result<RecentProjects, FlupiError> {
    let path = app_data_dir(&app).join("recent-projects.json");
    if path.exists() {
        file_io::read_json(&path)
    } else {
        Ok(RecentProjects::default())
    }
}

#[command]
pub fn add_recent_project(app: AppHandle, name: String, path: String) -> Result<(), FlupiError> {
    let file_path = app_data_dir(&app).join("recent-projects.json");
    let mut projects = if file_path.exists() {
        file_io::read_json(&file_path)?
    } else {
        RecentProjects::default()
    };
    projects.add(&name, &path);
    file_io::write_json(&file_path, &projects)
}

#[command]
pub fn get_preferences(app: AppHandle) -> Result<Preferences, FlupiError> {
    let path = app_data_dir(&app).join("preferences.json");
    if path.exists() {
        file_io::read_json(&path)
    } else {
        Ok(Preferences::default())
    }
}

#[command]
pub fn save_preferences(app: AppHandle, preferences: Preferences) -> Result<(), FlupiError> {
    let path = app_data_dir(&app).join("preferences.json");
    file_io::write_json(&path, &preferences)
}
```

- [ ] **Step 6: Register new commands in main.rs and verify compilation**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add app data models and commands for recent projects and preferences"
```

---

### Task 1.5: Project Picker UI

**Files:**
- Create: `src/lib/services/tauri-commands.ts`
- Create: `src/lib/stores/project.ts`
- Create: `src/lib/stores/ui.ts`
- Create: `src/lib/components/project-picker/RecentProjectList.svelte`
- Create: `src/routes/+page.svelte`  ← Project picker page

- [ ] **Step 1: Create typed Tauri command wrappers**

In `src/lib/services/tauri-commands.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface RecentProject {
  name: string;
  path: string;
  lastOpenedAt: string;
}

export interface RecentProjects {
  projects: RecentProject[];
}

export interface Preferences {
  theme: string;
  defaultTimeoutMs: number;
}

export async function getRecentProjects(): Promise<RecentProjects> {
  return invoke('get_recent_projects');
}

export async function addRecentProject(name: string, path: string): Promise<void> {
  return invoke('add_recent_project', { name, path });
}

export async function createProject(path: string): Promise<void> {
  return invoke('create_project', { path });
}

export async function openProject(path: string): Promise<string> {
  return invoke('open_project', { path });
}

export async function getPreferences(): Promise<Preferences> {
  return invoke('get_preferences');
}

export async function savePreferences(preferences: Preferences): Promise<void> {
  return invoke('save_preferences', { preferences });
}

export async function pickFolder(): Promise<string | null> {
  return open({ directory: true });
}
```

- [ ] **Step 2: Create project store**

In `src/lib/stores/project.ts`:

```typescript
import { writable } from 'svelte/store';

export interface ProjectState {
  isOpen: boolean;
  path: string | null;
  name: string | null;
}

export const project = writable<ProjectState>({
  isOpen: false,
  path: null,
  name: null,
});
```

- [ ] **Step 3: Create UI store for theme**

`activeSection` is removed — SvelteKit routing replaces it. The store holds theme only.

In `src/lib/stores/ui.ts`:

```typescript
import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light' | 'system';

export const theme = writable<Theme>('dark');

theme.subscribe((value) => {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  if (value === 'dark' || (value === 'system' && prefersDark)) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
});
```

- [ ] **Step 4: Build project picker route page**

In `src/routes/+page.svelte` (the project picker is the root page):

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import RecentProjectList from '$lib/components/project-picker/RecentProjectList.svelte';
  import { getRecentProjects, createProject, openProject, addRecentProject, pickFolder } from '$lib/services/tauri-commands';
  import { project } from '$lib/stores/project';
  import type { RecentProject } from '$lib/services/tauri-commands';

  let recentProjects: RecentProject[] = [];
  let error = '';

  onMount(async () => {
    const data = await getRecentProjects();
    recentProjects = data.projects;
  });

  async function openAndNavigate(path: string, name: string) {
    project.set({ isOpen: true, path, name });
    await addRecentProject(name, path);
    goto('/requests');
  }

  async function handleNewProject() {
    const path = await pickFolder();
    if (!path) return;
    try {
      await createProject(path);
      await openAndNavigate(path, path.split('/').pop() || path);
    } catch (e) { error = String(e); }
  }

  async function handleOpenFolder() {
    const path = await pickFolder();
    if (!path) return;
    try {
      const state = await openProject(path);
      if (state === 'empty') {
        if (!confirm("This folder doesn't appear to be a Flupi project. Initialize it?")) return;
        await createProject(path);
      }
      await openAndNavigate(path, path.split('/').pop() || path);
    } catch (e) { error = String(e); }
  }

  async function handleSelectRecent(p: RecentProject) {
    try {
      const state = await openProject(p.path);
      if (state === 'empty') {
        if (!confirm("This folder doesn't appear to be a Flupi project. Initialize it?")) return;
        await createProject(p.path);
      }
      await openAndNavigate(p.path, p.name);
    } catch (e) {
      error = String(e);
      recentProjects = recentProjects.filter(rp => rp.path !== p.path);
    }
  }
</script>

<div class="flex flex-col items-center justify-center h-screen bg-background text-foreground">
  <h1 class="text-3xl font-bold mb-2">Flupi</h1>
  <p class="text-muted-foreground mb-8">API test scenarios, powered by Git</p>

  {#if error}
    <p class="text-destructive mb-4">{error}</p>
  {/if}

  <div class="flex gap-4 mb-8">
    <button class="btn btn-primary" onclick={handleNewProject}>New Project</button>
    <button class="btn btn-secondary" onclick={handleOpenFolder}>Open Folder</button>
  </div>

  {#if recentProjects.length > 0}
    <RecentProjectList projects={recentProjects} onSelect={handleSelectRecent} />
  {:else}
    <p class="text-muted-foreground text-sm">No recent projects</p>
  {/if}
</div>
```

- [ ] **Step 5: Build RecentProjectList component**

In `src/lib/components/project-picker/RecentProjectList.svelte`:

```svelte
<script lang="ts">
  import type { RecentProject } from '$lib/services/tauri-commands';

  interface Props {
    projects: RecentProject[];
    onSelect: (project: RecentProject) => void;
  }

  let { projects, onSelect }: Props = $props();
</script>

<div class="w-full max-w-md">
  <h2 class="text-sm font-medium text-muted-foreground mb-2">Recent Projects</h2>
  <ul class="space-y-1">
    {#each projects as p}
      <li>
        <button
          class="w-full text-left p-3 rounded hover:bg-accent transition-colors"
          onclick={() => onSelect(p)}
        >
          <div class="font-medium">{p.name}</div>
          <div class="text-xs text-muted-foreground truncate">{p.path}</div>
        </button>
      </li>
    {/each}
  </ul>
</div>
```

- [ ] **Step 6: Verify the app launches and shows the project picker**

```bash
npm run tauri dev
```

Expected: App launches in dark mode at `/` showing the Flupi project picker with "New Project" and "Open Folder" buttons. Choosing a project navigates to `/requests`.

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add project picker route with recent projects"
```

---

### Task 1.6: Main Layout Shell

**Files:**
- Create: `src/lib/components/layout/TopBar.svelte`
- Create: `src/lib/components/layout/Sidebar.svelte`
- Create: `src/routes/(app)/+layout.svelte`
- Create: `src/routes/(app)/requests/+page.svelte` (placeholder)
- Create: `src/routes/(app)/environments/+page.svelte` (placeholder)
- Create: `src/routes/(app)/scenarios/+page.svelte` (placeholder)
- Create: `src/routes/(app)/openapi/+page.svelte` (placeholder)
- Create: `src/routes/(app)/settings/+page.svelte` (placeholder)

This establishes the app shell before any section UI is built. Real section components replace placeholders in later phases.

- [ ] **Step 1: Build TopBar**

`src/lib/components/layout/TopBar.svelte` — Flupi logo, active project name, environment switcher dropdown (reads from environment store), "Switch Project" button that calls `goto('/')`.

- [ ] **Step 2: Build Sidebar**

`src/lib/components/layout/Sidebar.svelte` — 5 navigation icons (Environments, OpenAPI, Requests, Scenarios, Settings). Uses `$page.url.pathname` to highlight the active route. Each icon calls `goto('/requests')` etc.

```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  const sections = [
    { path: '/environments', label: 'Environments', icon: '⚙' },
    { path: '/openapi',      label: 'OpenAPI',      icon: '📄' },
    { path: '/requests',     label: 'Requests',     icon: '↗' },
    { path: '/scenarios',    label: 'Scenarios',    icon: '▶' },
    { path: '/settings',     label: 'Settings',     icon: '🔧' },
  ];
</script>

<nav class="flex flex-col w-14 border-r bg-sidebar">
  {#each sections as s}
    <button
      class="p-4 hover:bg-accent {$page.url.pathname.startsWith(s.path) ? 'bg-accent' : ''}"
      onclick={() => goto(s.path)}
      title={s.label}
    >{s.icon}</button>
  {/each}
</nav>
```

- [ ] **Step 3: Build app group layout**

`src/routes/(app)/+layout.svelte` — the shell that wraps all app pages. Redirects to `/` if no project is open.

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import { project } from '$lib/stores/project';

  onMount(() => {
    if (!$project.isOpen) goto('/');
  });
</script>

<div class="flex flex-col h-screen">
  <TopBar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>
</div>
```

- [ ] **Step 4: Create placeholder section pages**

Each of the 5 route files just renders a placeholder for now:

```svelte
<!-- src/routes/(app)/requests/+page.svelte -->
<p class="p-8 text-muted-foreground">Requests — coming soon</p>
```

Repeat for `environments`, `scenarios`, `openapi`, `settings`.

- [ ] **Step 5: Verify navigation between all sections works**

Open the project picker, create a project, confirm you land on `/requests`. Click each sidebar icon and confirm the URL and active highlight update correctly.

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add SvelteKit app shell with TopBar, Sidebar, and section route placeholders"
```

---

## Phase 2: Environment Management

### Task 2.1: Environment Models and CRUD

**Files:**
- Create: `src-tauri/src/models/environment.rs`
- Create: `src-tauri/src/commands/environment.rs`

- [ ] **Step 1: Write tests for environment models**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::services::file_io;

    #[test]
    fn test_load_environment() {
        let dir = TempDir::new().unwrap();
        let env_dir = dir.path().join("environments");
        std::fs::create_dir(&env_dir).unwrap();

        let env = Environment {
            name: "Dev".to_string(),
            variables: [("baseUrl".to_string(), "https://dev.api".to_string())]
                .into_iter().collect(),
            secrets: vec![],
        };
        file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();

        let loaded: Environment = file_io::read_json(&env_dir.join("dev.json")).unwrap();
        assert_eq!(loaded.name, "Dev");
        assert_eq!(loaded.variables["baseUrl"], "https://dev.api");
    }

    #[test]
    fn test_load_environment_with_secrets() {
        let dir = TempDir::new().unwrap();
        let env_dir = dir.path().join("environments");
        std::fs::create_dir(&env_dir).unwrap();

        let env = Environment {
            name: "Dev".to_string(),
            variables: [
                ("baseUrl".to_string(), "https://dev.api".to_string()),
                ("client_secret".to_string(), String::new()),
            ].into_iter().collect(),
            secrets: vec!["client_secret".to_string()],
        };
        file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();

        let secrets: std::collections::HashMap<String, String> =
            [("client_secret".to_string(), "supersecret".to_string())]
                .into_iter().collect();
        file_io::write_json(&env_dir.join("dev.secrets.json"), &secrets).unwrap();

        let resolved = resolve_env_variables(&env_dir.join("dev.json")).unwrap();
        assert_eq!(resolved["baseUrl"], "https://dev.api");
        assert_eq!(resolved["client_secret"], "supersecret");
    }

    #[test]
    fn test_secret_key_excluded_from_variables() {
        let dir = TempDir::new().unwrap();
        let env_dir = dir.path().join("environments");
        std::fs::create_dir(&env_dir).unwrap();

        let env = Environment {
            name: "Dev".to_string(),
            variables: [
                ("client_secret".to_string(), "placeholder-should-not-load".to_string()),
            ].into_iter().collect(),
            secrets: vec!["client_secret".to_string()],
        };
        file_io::write_json(&env_dir.join("dev.json"), &env).unwrap();
        // No secrets file exists

        let resolved = resolve_env_variables(&env_dir.join("dev.json")).unwrap();
        // Secret key should NOT be in the resolved map (no secrets file)
        assert!(!resolved.contains_key("client_secret"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement environment model and resolution**

```rust
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use crate::error::Result;
use crate::services::file_io;

/// Uses IndexMap instead of HashMap to preserve insertion order,
/// producing stable JSON output and clean Git diffs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: IndexMap<String, String>,
    #[serde(default)]
    pub secrets: Vec<String>,
}

pub fn resolve_env_variables(env_path: &Path) -> Result<HashMap<String, String>> {
    let env: Environment = file_io::read_json(env_path)?;
    let mut vars: HashMap<String, String> = env.variables
        .into_iter()
        .filter(|(key, _)| !env.secrets.contains(key))
        .collect();

    // Load secrets file if it exists
    // Use string replace instead of with_extension to handle filenames with dots
    let env_name = env_path.file_name().unwrap().to_string_lossy();
    let secrets_name = env_name.replace(".json", ".secrets.json");
    let secrets_path = env_path.with_file_name(secrets_name);
    if secrets_path.exists() {
        let secrets: HashMap<String, String> = file_io::read_json(&secrets_path)?;
        vars.extend(secrets);
    }

    Ok(vars)
}

pub fn list_environments(project_path: &Path) -> Result<Vec<(String, Environment)>> {
    let env_dir = project_path.join("environments");
    let mut envs = Vec::new();

    if env_dir.exists() {
        for entry in std::fs::read_dir(&env_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && !path.file_name().unwrap().to_string_lossy().ends_with(".secrets.json")
            {
                let env: Environment = file_io::read_json(&path)?;
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                envs.push((file_name, env));
            }
        }
    }

    Ok(envs)
}
```

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Implement environment Tauri commands**

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;
use crate::error::FlupiError;
use crate::models::environment::{self, Environment};
use crate::services::file_io;

#[command]
pub fn list_environments(project_path: PathBuf) -> Result<Vec<(String, Environment)>, FlupiError> {
    environment::list_environments(&project_path)
}

#[command]
pub fn save_environment(project_path: PathBuf, file_name: String, env: Environment) -> Result<(), FlupiError> {
    let path = project_path.join("environments").join(format!("{}.json", file_name));
    file_io::write_json(&path, &env)
}

#[command]
pub fn save_secrets(project_path: PathBuf, file_name: String, secrets: HashMap<String, String>) -> Result<(), FlupiError> {
    let path = project_path.join("environments").join(format!("{}.secrets.json", file_name));
    file_io::write_json(&path, &secrets)
}

#[command]
pub fn get_resolved_variables(project_path: PathBuf, file_name: String) -> Result<HashMap<String, String>, FlupiError> {
    let path = project_path.join("environments").join(format!("{}.json", file_name));
    environment::resolve_env_variables(&path)
}

#[command]
pub fn delete_environment(project_path: PathBuf, file_name: String) -> Result<(), FlupiError> {
    let env_path = project_path.join("environments").join(format!("{}.json", file_name));
    let secrets_path = project_path.join("environments").join(format!("{}.secrets.json", file_name));
    file_io::delete_file(&env_path)?;
    if secrets_path.exists() {
        file_io::delete_file(&secrets_path)?;
    }
    Ok(())
}
```

- [ ] **Step 6: Register commands, verify compilation**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add environment model, secrets resolution, and CRUD commands"
```

---

### Task 2.2: Environment UI

**Files:**
- Create: `src/lib/stores/environment.ts`
- Create: `src/lib/components/environments/EnvironmentList.svelte`
- Create: `src/lib/components/environments/EnvironmentEditor.svelte`
- Create: `src/lib/components/shared/KeyValueTable.svelte`
- Create: `src/lib/services/debounced-save.ts`
- Modify: `src/routes/(app)/environments/+page.svelte` ← replace placeholder with real components

- [ ] **Step 1: Create debounced save utility**

```typescript
export function createDebouncedSave(saveFn: () => Promise<void>, delay = 500): {
  trigger: () => void;
  flush: () => Promise<void>;
} {
  let timer: ReturnType<typeof setTimeout> | null = null;

  function trigger() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      saveFn();
      timer = null;
    }, delay);
  }

  async function flush() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    await saveFn();
  }

  return { trigger, flush };
}
```

- [ ] **Step 2: Create environment store**

```typescript
import { writable } from 'svelte/store';
import type { Environment } from '$lib/services/tauri-commands';

export interface EnvironmentEntry {
  fileName: string;
  environment: Environment;
  secrets: Record<string, string>;
}

export const environments = writable<EnvironmentEntry[]>([]);
export const activeEnvironment = writable<string | null>(null);
```

- [ ] **Step 3: Build KeyValueTable shared component**

A reusable key-value editor with add row, edit inline, delete row. Accepts props for whether to show a "secret" toggle column.

```svelte
<script lang="ts">
  interface Row {
    key: string;
    value: string;
    isSecret?: boolean;
  }

  interface Props {
    rows: Row[];
    showSecretToggle?: boolean;
    readOnlyKeys?: string[];
    onUpdate: (rows: Row[]) => void;
  }

  let { rows, showSecretToggle = false, readOnlyKeys = [], onUpdate }: Props = $props();

  function addRow() {
    onUpdate([...rows, { key: '', value: '', isSecret: false }]);
  }

  function removeRow(index: number) {
    onUpdate(rows.filter((_, i) => i !== index));
  }

  function updateRow(index: number, field: keyof Row, value: string | boolean) {
    const updated = [...rows];
    updated[index] = { ...updated[index], [field]: value };
    onUpdate(updated);
  }
</script>

<div class="space-y-1">
  {#each rows as row, i}
    <div class="flex gap-2 items-center">
      <input
        class="flex-1 bg-input border rounded px-2 py-1 text-sm"
        value={row.key}
        readonly={readOnlyKeys.includes(row.key)}
        oninput={(e) => updateRow(i, 'key', e.currentTarget.value)}
        placeholder="Key"
      />
      <input
        class="flex-1 bg-input border rounded px-2 py-1 text-sm"
        type={row.isSecret ? 'password' : 'text'}
        value={row.value}
        oninput={(e) => updateRow(i, 'value', e.currentTarget.value)}
        placeholder="Value"
      />
      {#if showSecretToggle}
        <label class="flex items-center gap-1 text-xs text-muted-foreground">
          <input
            type="checkbox"
            checked={row.isSecret}
            onchange={(e) => updateRow(i, 'isSecret', e.currentTarget.checked)}
          />
          Secret
        </label>
      {/if}
      <button class="text-destructive text-sm" onclick={() => removeRow(i)}>×</button>
    </div>
  {/each}
  <button class="text-sm text-primary" onclick={addRow}>+ Add variable</button>
</div>
```

- [ ] **Step 4: Build EnvironmentList component**

Lists all environments, highlights active one, allows switching and creating new environments.

- [ ] **Step 5: Build EnvironmentEditor component**

Integrates KeyValueTable with secret toggle. Auto-saves via debounced save. Shows secret values masked with reveal toggle.

- [ ] **Step 6: Verify environments screen works end-to-end**

Launch app, create project, navigate to Environments, create an environment, add variables, toggle secrets, verify auto-save writes correct JSON files.

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add environment management UI with secret toggle and auto-save"
```

---

## Phase 3: Request Library

### Task 3.1: Request and Collection Models

**Files:**
- Create: `src-tauri/src/models/request.rs`
- Create: `src-tauri/src/models/collection.rs`

- [ ] **Step 1: Write tests for request ID derivation**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_request_id_collection() {
        let project = Path::new("/project");
        let path = Path::new("/project/collections/auth-service/requests/get-token.json");
        assert_eq!(derive_request_id(project, path), "auth-service/get-token");
    }

    #[test]
    fn test_derive_request_id_collection_nested() {
        let project = Path::new("/project");
        let path = Path::new("/project/collections/auth-service/requests/admin/create-user.json");
        assert_eq!(derive_request_id(project, path), "auth-service/admin/create-user");
    }

    #[test]
    fn test_derive_request_id_root() {
        let project = Path::new("/project");
        let path = Path::new("/project/requests/health-check.json");
        assert_eq!(derive_request_id(project, path), "health-check");
    }

    #[test]
    fn test_derive_request_id_root_nested() {
        let project = Path::new("/project");
        let path = Path::new("/project/requests/monitoring/status.json");
        assert_eq!(derive_request_id(project, path), "monitoring/status");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement request model with ID derivation**

```rust
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub headers: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "templateRef")]
    pub template_ref: Option<TemplateRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AuthConfig {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "inherit")]
    Inherit,
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "basic")]
    Basic { username: String, password: String },
    #[serde(rename = "apiKey")]
    ApiKey { header: String, value: String },
    #[serde(rename = "custom")]
    Custom { headers: HashMap<String, String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum BodyConfig {
    #[serde(rename = "json")]
    Json { content: serde_json::Value },
    #[serde(rename = "form")]
    Form { content: HashMap<String, String> },
    #[serde(rename = "raw")]
    Raw { content: String },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateRef {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    #[serde(rename = "schemaHash")]
    pub schema_hash: String,
    #[serde(rename = "requestSchema")]
    pub request_schema: serde_json::Value,
    #[serde(rename = "responseSchema")]
    pub response_schema: serde_json::Value,
}

pub fn derive_request_id(project_root: &Path, request_path: &Path) -> String {
    let relative = request_path.strip_prefix(project_root).unwrap();
    let parts: Vec<&str> = relative.iter().map(|s| s.to_str().unwrap()).collect();

    if parts[0] == "collections" {
        // collections/{folderName}/requests/[subpath/]fileName.json
        let folder_name = parts[1];
        // Skip "requests" at parts[2]
        let rest: Vec<&str> = parts[3..].to_vec();
        let file_stem = Path::new(rest.last().unwrap()).file_stem().unwrap().to_str().unwrap();
        let mut id_parts = vec![folder_name];
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        id_parts.join("/")
    } else {
        // requests/[subpath/]fileName.json
        let rest: Vec<&str> = parts[1..].to_vec();
        let file_stem = Path::new(rest.last().unwrap()).file_stem().unwrap().to_str().unwrap();
        let mut id_parts: Vec<&str> = Vec::new();
        for part in &rest[..rest.len() - 1] {
            id_parts.push(part);
        }
        id_parts.push(file_stem);
        id_parts.join("/")
    }
}
```

- [ ] **Step 4: Implement collection model**

```rust
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use crate::models::request::AuthConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub name: String,
    #[serde(rename = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub headers: IndexMap<String, String>,
}
```

- [ ] **Step 5: Run tests to verify they pass**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add request and collection models with runtime ID derivation"
```

---

### Task 3.2: Request CRUD Commands

**Files:**
- Create: `src-tauri/src/commands/request.rs`
- Create: `src-tauri/src/commands/collection.rs`

- [ ] **Step 1: Write tests for loading request tree**

Test that `load_request_tree` returns a proper tree structure with collections, nested folders, and root requests — each with derived IDs.

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement request tree loading**

Build a function that scans `collections/` and `requests/` directories, reads all JSON files, derives IDs, and returns a tree structure. Define a `RequestTreeNode` enum:

```rust
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum RequestTreeNode {
    Collection {
        name: String,
        folder_name: String,
        children: Vec<RequestTreeNode>,
    },
    Folder {
        name: String,
        children: Vec<RequestTreeNode>,
    },
    Request {
        id: String,
        name: String,
        method: String,
        file_path: String,
    },
}
```

- [ ] **Step 4: Implement CRUD commands**

Commands: `load_request_tree`, `get_request`, `save_request`, `create_request`, `delete_request`, `rename_request`, `move_request`, `duplicate_request`, `create_collection`, `save_collection`, `delete_collection`, `rename_collection`.

- [ ] **Step 5: Run tests to verify they pass**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add request and collection CRUD commands with tree loading"
```

---

### Task 3.3: Request Library UI — Tree Sidebar

**Files:**
- Create: `src/lib/stores/requests.ts`
- Create: `src/lib/stores/collections.ts`
- Create: `src/lib/components/shared/TreeNode.svelte`
- Create: `src/lib/components/shared/ContextMenu.svelte`
- Create: `src/lib/components/requests/RequestTree.svelte`
- Modify: `src/routes/(app)/requests/+page.svelte` ← replace placeholder

- [ ] **Step 1: Create request and collection stores**

Stores hold the tree data and active request selection.

- [ ] **Step 2: Build recursive TreeNode component**

A reusable recursive tree component that renders folders, collections, and requests. Supports expand/collapse, selection, right-click context menu.

- [ ] **Step 3: Build ContextMenu component**

A positioned context menu that appears on right-click with configurable items (New Request, New Folder, Rename, Duplicate, Move to..., Delete).

- [ ] **Step 4: Build RequestTree component**

Composes TreeNode and ContextMenu. Loads tree data from Tauri commands. Handles all context menu actions.

- [ ] **Step 5: Verify tree renders correctly with test data**

Create sample collection and request JSON files in a test project, launch app, verify the tree renders with correct nesting.

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add request library sidebar tree with context menus"
```

---

### Task 3.4: Request Editor UI

**Files:**
- Create: `src/lib/components/requests/RequestEditor.svelte`
- Create: `src/lib/components/requests/ParamsTab.svelte`
- Create: `src/lib/components/requests/HeadersTab.svelte`
- Create: `src/lib/components/requests/AuthTab.svelte`
- Create: `src/lib/components/requests/BodyTab.svelte`
- Create: `src/lib/components/requests/ResponsePanel.svelte`
- Create: `src/lib/components/shared/JsonEditor.svelte`

- [ ] **Step 1: Build RequestEditor with tab navigation**

Tab bar with: Params, Headers, Auth, Body, Schema (if template), Effective Request. Each tab renders its sub-component. Auto-save on all edits.

- [ ] **Step 2: Build ParamsTab**

Key-value table for URL path parameters and query parameters. Values support `{{variable}}` tokens.

- [ ] **Step 3: Build HeadersTab**

Key-value table. Inherited headers shown as read-only with "inherited" badge. Own headers editable.

- [ ] **Step 4: Build AuthTab**

Dropdown selector (Inherit / None / Bearer / Basic / API Key / Custom) + relevant fields per type.

- [ ] **Step 5: Build BodyTab**

Body type selector (None / JSON / Form / Raw) + content editor. JSON type uses a code editor with syntax highlighting (use a simple textarea with monospace font for v1, or integrate a lightweight code editor).

- [ ] **Step 6: Build ResponsePanel**

Displays: status code (color-coded), response time, headers as collapsible list, body (pretty-printed JSON or raw text with truncation at 1MB).

- [ ] **Step 7: Verify the request editor works with all tab types**

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: add request editor with params, headers, auth, body tabs and response panel"
```

---

### Task 3.5: Drag-and-Drop in Request Library

**Files:**
- Modify: `src/lib/components/requests/RequestTree.svelte`

- [ ] **Step 1: Integrate svelte-dnd-action into the tree**

Add drag handles to request nodes. Allow dragging requests between collections, folders, and root. On drop, call `move_request` Tauri command.

- [ ] **Step 2: Show informational notice on cross-collection moves**

When a request is dragged from one collection to another, show a brief toast: "Request will now inherit auth and headers from [target collection]."

- [ ] **Step 3: Verify drag-and-drop updates file system and tree**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add drag-and-drop request reordering in library sidebar"
```

---

## Phase 4: Collection Inheritance and Variable Resolution

### Task 4.1: Variable Resolution Engine

**Files:**
- Create: `src-tauri/src/models/variable.rs`
- Create: `src-tauri/src/services/variable_resolver.rs`

- [ ] **Step 1: Write tests for variable resolution**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_simple_variable() {
        let ctx = VariableContext::new();
        ctx.set("baseUrl", "https://api.dev");

        let result = resolve_string("{{baseUrl}}/auth", &ctx);
        assert_eq!(result, "https://api.dev/auth");
    }

    #[test]
    fn test_resolve_multiple_variables() {
        let ctx = VariableContext::new();
        ctx.set("host", "api.dev");
        ctx.set("token", "abc123");

        let result = resolve_string("https://{{host}}/auth?token={{token}}", &ctx);
        assert_eq!(result, "https://api.dev/auth?token=abc123");
    }

    #[test]
    fn test_unresolved_variable_preserved() {
        let ctx = VariableContext::new();
        let result = resolve_string("{{missing}}", &ctx);
        assert_eq!(result, "{{missing}}");
    }

    #[test]
    fn test_list_unresolved_variables() {
        let ctx = VariableContext::new();
        ctx.set("host", "api.dev");

        let unresolved = find_unresolved("{{host}}/{{path}}", &ctx);
        assert_eq!(unresolved, vec!["path"]);
    }

    #[test]
    fn test_priority_order() {
        // Later sources win
        let mut ctx = VariableContext::new();
        ctx.set("key", "env-value");        // from environment
        ctx.set("key", "input-value");      // from scenario input (overrides)

        let result = resolve_string("{{key}}", &ctx);
        assert_eq!(result, "input-value");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement variable resolver**

```rust
use std::collections::HashMap;
use regex::Regex;

pub struct VariableContext {
    variables: HashMap<String, String>,
    secret_keys: Vec<String>,
}

impl VariableContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            secret_keys: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn mark_secret(&mut self, key: &str) {
        if !self.secret_keys.contains(&key.to_string()) {
            self.secret_keys.push(key.to_string());
        }
    }

    pub fn is_secret(&self, key: &str) -> bool {
        self.secret_keys.contains(&key.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}

pub fn resolve_string(template: &str, ctx: &VariableContext) -> String {
    let re = Regex::new(r"\{\{([a-zA-Z0-9_.-]+)\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        ctx.get(key).unwrap_or(&caps[0]).to_string()
    }).to_string()
}

pub fn find_unresolved(template: &str, ctx: &VariableContext) -> Vec<String> {
    let re = Regex::new(r"\{\{([a-zA-Z0-9_.-]+)\}\}").unwrap();
    re.captures_iter(template)
        .filter_map(|cap| {
            let key = cap[1].to_string();
            if ctx.get(&key).is_none() { Some(key) } else { None }
        })
        .collect()
}

/// Build a full variable context for execution
pub fn build_context(
    env_vars: HashMap<String, String>,
    secret_keys: &[String],
    scenario_inputs: Option<&HashMap<String, String>>,
    extracted: Option<&HashMap<String, String>>,
) -> VariableContext {
    let mut ctx = VariableContext::new();

    // Layer 1: environment variables
    for (k, v) in &env_vars {
        ctx.set(k, v);
    }

    // Mark secret keys
    for key in secret_keys {
        ctx.mark_secret(key);
    }

    // Layer 2: scenario inputs (override env vars)
    if let Some(inputs) = scenario_inputs {
        for (k, v) in inputs {
            ctx.set(k, v);
        }
    }

    // Layer 3: extracted variables (override everything)
    if let Some(ext) = extracted {
        for (k, v) in ext {
            ctx.set(k, v);
        }
    }

    ctx
}
```

Note: `regex` is already in Cargo.toml from Task 1.1.

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add variable resolution engine with priority-based context merging"
```

---

### Task 4.2: Inheritance Resolution

**Files:**
- Create: `src-tauri/src/services/inheritance.rs`

- [ ] **Step 1: Write tests for inheritance resolution**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::{Request, AuthConfig, BodyConfig};
    use crate::models::collection::Collection;
    use std::collections::HashMap;

    #[test]
    fn test_inherit_auth_from_collection() {
        let collection = Collection {
            name: "Auth".to_string(),
            base_url: None,
            auth: Some(AuthConfig::Bearer { token: "{{token}}".to_string() }),
            headers: HashMap::new(),
        };
        let request = Request {
            name: "Get".to_string(),
            method: "GET".to_string(),
            path: "/resource".to_string(),
            auth: Some(AuthConfig::Inherit),
            headers: HashMap::new(),
            body: None,
            template_ref: None,
        };

        let effective = resolve_inheritance(&request, Some(&collection));
        match effective.auth.unwrap() {
            AuthConfig::Bearer { token } => assert_eq!(token, "{{token}}"),
            _ => panic!("expected bearer auth"),
        }
    }

    #[test]
    fn test_merge_headers_request_wins() {
        let collection = Collection {
            name: "Test".to_string(),
            base_url: None,
            auth: None,
            headers: [("Content-Type".to_string(), "application/json".to_string())].into(),
        };
        let request = Request {
            name: "Get".to_string(),
            method: "GET".to_string(),
            path: "/resource".to_string(),
            auth: None,
            headers: [("Content-Type".to_string(), "text/plain".to_string())].into(),
            body: None,
            template_ref: None,
        };

        let effective = resolve_inheritance(&request, Some(&collection));
        assert_eq!(effective.headers["Content-Type"], "text/plain");
    }

    #[test]
    fn test_prepend_base_url() {
        let collection = Collection {
            name: "Test".to_string(),
            base_url: Some("https://api.dev".to_string()),
            auth: None,
            headers: HashMap::new(),
        };
        let request = Request {
            name: "Get".to_string(),
            method: "GET".to_string(),
            path: "/resource".to_string(),
            auth: None,
            headers: HashMap::new(),
            body: None,
            template_ref: None,
        };

        let effective = resolve_inheritance(&request, Some(&collection));
        assert_eq!(effective.path, "https://api.dev/resource");
    }

    #[test]
    fn test_absolute_url_not_prepended() {
        let collection = Collection {
            name: "Test".to_string(),
            base_url: Some("https://api.dev".to_string()),
            auth: None,
            headers: HashMap::new(),
        };
        let request = Request {
            name: "Get".to_string(),
            method: "GET".to_string(),
            path: "https://other.api/resource".to_string(),
            auth: None,
            headers: HashMap::new(),
            body: None,
            template_ref: None,
        };

        let effective = resolve_inheritance(&request, Some(&collection));
        assert_eq!(effective.path, "https://other.api/resource");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement inheritance resolution**

```rust
use crate::models::request::{Request, AuthConfig};
use crate::models::collection::Collection;

pub fn resolve_inheritance(request: &Request, collection: Option<&Collection>) -> Request {
    let mut effective = request.clone();

    if let Some(col) = collection {
        // Auth: inherit if absent or explicit Inherit
        match &effective.auth {
            None | Some(AuthConfig::Inherit) => {
                effective.auth = col.auth.clone();
            }
            _ => {}
        }

        // Headers: merge, request wins on conflict
        let mut merged = col.headers.clone();
        merged.extend(effective.headers.clone());
        effective.headers = merged;

        // BaseUrl: prepend if path is relative
        if let Some(base_url) = &col.base_url {
            if !effective.path.starts_with("http://") && !effective.path.starts_with("https://") {
                effective.path = format!("{}{}", base_url.trim_end_matches('/'), effective.path);
            }
        }
    }

    effective
}
```

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add collection inheritance resolution for auth, headers, and base URL"
```

---

### Task 4.3: Effective Request Preview and Frontend Variable Resolution

**Files:**
- Create: `src/lib/services/variable-resolver.ts`
- Create: `src/lib/components/requests/EffectiveRequestTab.svelte`
- Create: `src/lib/components/requests/CollectionEditor.svelte`

- [ ] **Step 1: Implement frontend variable resolver**

A TypeScript port of the resolve/find-unresolved logic for preview purposes:

```typescript
export function resolveString(template: string, variables: Record<string, string>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    return variables[key] ?? match;
  });
}

export function findUnresolved(template: string, variables: Record<string, string>): string[] {
  const unresolved: string[] = [];
  template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    if (!(key in variables)) unresolved.push(key);
    return match;
  });
  return unresolved;
}
```

- [ ] **Step 2: Build EffectiveRequestTab**

Read-only preview showing the fully assembled request after inheritance and variable resolution for the active environment. Calls Tauri command to get resolved request, then substitutes variables on the frontend for live preview.

- [ ] **Step 3: Build CollectionEditor**

Fields: Name, Base URL, Auth (same auth selector as requests), default Headers. All changes auto-save via debounced save.

- [ ] **Step 4: Verify effective request preview updates when switching environments**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add effective request preview and collection editor"
```

---

## Phase 5: HTTP Execution

### Task 5.1: HTTP Client Service

**Files:**
- Create: `src-tauri/src/services/http_client.rs`

- [ ] **Step 1: Write tests for HTTP client**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_reqwest_request() {
        let req = ExecutableRequest {
            method: "GET".to_string(),
            url: "https://httpbin.org/get".to_string(),
            headers: [("Accept".to_string(), "application/json".to_string())].into(),
            body: None,
            timeout_ms: 30000,
        };

        let built = build_request(&req).unwrap();
        assert_eq!(built.method().as_str(), "GET");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement HTTP client**

```rust
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutableRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    #[serde(rename = "json")]
    Json { content: serde_json::Value },
    #[serde(rename = "form")]
    Form { content: HashMap<String, String> },
    #[serde(rename = "raw")]
    Raw { content: String },
}

#[derive(Debug, Serialize, Clone)]
pub struct HttpResponse {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
    #[serde(rename = "bodyTruncated")]
    pub body_truncated: bool,
}

const MAX_BODY_SIZE: usize = 1_048_576; // 1MB

pub async fn execute_request(req: &ExecutableRequest) -> Result<HttpResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(req.timeout_ms))
        .build()?;

    let mut builder = match req.method.to_uppercase().as_str() {
        "GET" => client.get(&req.url),
        "POST" => client.post(&req.url),
        "PUT" => client.put(&req.url),
        "DELETE" => client.delete(&req.url),
        "PATCH" => client.patch(&req.url),
        "HEAD" => client.head(&req.url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &req.url),
        method => client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &req.url),
    };

    for (key, value) in &req.headers {
        builder = builder.header(key, value);
    }

    if let Some(body) = &req.body {
        builder = match body {
            RequestBody::Json { content } => builder.json(content),
            RequestBody::Form { content } => builder.form(content),
            RequestBody::Raw { content } => builder.body(content.clone()),
        };
    }

    let start = std::time::Instant::now();
    let response = builder.send().await?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();
    let headers: HashMap<String, String> = response.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Stream response body up to MAX_BODY_SIZE to avoid OOM on large responses
    let mut body_bytes = Vec::new();
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    let mut body_truncated = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        body_bytes.extend_from_slice(&chunk);
        if body_bytes.len() > MAX_BODY_SIZE {
            body_bytes.truncate(MAX_BODY_SIZE);
            body_truncated = true;
            break;
        }
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        duration_ms,
        body_truncated,
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add HTTP client service with timeout and response truncation"
```

---

### Task 5.2: Send Request Command and Execution Lock

**Files:**
- Create: `src-tauri/src/commands/execution.rs`
- Create: `src/lib/stores/execution.ts`

- [ ] **Step 1: Implement execution command**

The `send_request` command takes a project path, request ID, and active environment. It:
1. Loads the request file
2. Finds its parent collection (if any)
3. Resolves inheritance
4. Builds variable context from active environment
5. Resolves all `{{variables}}`
6. Executes the HTTP request
7. Returns the response

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::command;
use crate::error::FlupiError;
use crate::models::request;
use crate::models::environment;
use crate::services::{http_client, inheritance, variable_resolver, file_io};

static EXECUTION_LOCK: AtomicBool = AtomicBool::new(false);

fn acquire_lock() -> Result<(), FlupiError> {
    if EXECUTION_LOCK.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err(FlupiError::Custom("Another execution is already in progress".to_string()));
    }
    Ok(())
}

fn release_lock() {
    EXECUTION_LOCK.store(false, Ordering::SeqCst);
}

#[command]
pub async fn send_request(
    project_path: PathBuf,
    request_id: String,
    env_file_name: String,
    timeout_ms: u64,
) -> Result<http_client::HttpResponse, FlupiError> {
    acquire_lock()?;

    let result = execute_single_request(&project_path, &request_id, &env_file_name, timeout_ms, &HashMap::new()).await;

    release_lock();
    result
}
```

- [ ] **Step 2: Create execution store on frontend**

```typescript
import { writable } from 'svelte/store';

export const isExecuting = writable(false);
```

- [ ] **Step 3: Wire "Send" button to execution command**

In RequestEditor, add a Send button that calls `send_request` and displays the response in ResponsePanel. Disable all Send/Run buttons while `isExecuting` is true.

- [ ] **Step 4: Verify ad-hoc send works end-to-end**

Launch app, open a project with a request, select an environment, click Send, verify response displays.

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add ad-hoc request execution with concurrency lock"
```

---

## Phase 6: Scenario Editor and Runner

### Task 6.1: Scenario Model

**Files:**
- Create: `src-tauri/src/models/scenario.rs`
- Create: `src-tauri/src/commands/scenario.rs`

- [ ] **Step 1: Write tests for scenario model serialization**

- [ ] **Step 2: Implement scenario model**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<ScenarioInput>,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioStep {
    pub id: String,
    pub name: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(default)]
    pub overrides: HashMap<String, String>,
    #[serde(default)]
    pub extract: Vec<Extraction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extraction {
    pub variable: String,
    pub from: String,     // "response.body" or "response.headers"
    pub path: String,     // JSONPath expression
}
```

- [ ] **Step 3: Implement scenario CRUD commands**

Commands: `load_scenario_tree`, `get_scenario`, `save_scenario`, `create_scenario`, `delete_scenario`, `rename_scenario`, `move_scenario`, `duplicate_scenario`.

- [ ] **Step 4: Run tests, verify pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add scenario model and CRUD commands"
```

---

### Task 6.2: Scenario Editor UI

**Files:**
- Create: `src/lib/stores/scenarios.ts`
- Create: `src/lib/components/scenarios/ScenarioTree.svelte`
- Create: `src/lib/components/scenarios/ScenarioEditor.svelte`
- Create: `src/lib/components/scenarios/InputsList.svelte`
- Create: `src/lib/components/scenarios/StepCard.svelte`
- Create: `src/lib/components/scenarios/OverridesPanel.svelte`
- Modify: `src/routes/(app)/scenarios/+page.svelte` ← replace placeholder
- Create: `src/lib/components/scenarios/ExtractionsPanel.svelte`

- [ ] **Step 1: Create scenario store**

- [ ] **Step 2: Build ScenarioTree**

Sidebar tree showing scenario groups (nested folders) and scenarios. Context menus per spec Section 15. Support drag-and-drop of scenarios between groups (using svelte-dnd-action).

- [ ] **Step 3: Build ScenarioEditor**

Inputs section (list of input definitions with add/edit/delete/reorder) and steps section.

- [ ] **Step 4: Build StepCard**

Collapsible card with drag handle (svelte-dnd-action). Shows step number, name, method + path when collapsed. Expands to show OverridesPanel and ExtractionsPanel. If the step's `requestId` does not match any request in the loaded request tree, show a warning badge ("Request not found") on the collapsed card — this handles broken references from manual file edits outside Flupi.

- [ ] **Step 5: Build OverridesPanel**

Key-value table. Key field accepts dot-notation paths (e.g., `body.warehouseId`). Value field accepts `{{variable}}` tokens.

- [ ] **Step 6: Build ExtractionsPanel**

List of extraction definitions. Each row: variable name, source selector (response.body / response.headers), JSONPath expression.

- [ ] **Step 7: Wire drag-and-drop for step reordering**

Use svelte-dnd-action for reordering steps within the scenario editor.

- [ ] **Step 8: Verify scenario editor end-to-end**

Create a scenario, add steps referencing existing requests, configure overrides and extractions, verify JSON saves correctly.

- [ ] **Step 9: Commit**

```bash
git commit -m "feat: add scenario editor UI with step cards, overrides, and extractions"
```

---

### Task 6.3: Scenario Runner — Backend

**Files:**
- Modify: `src-tauri/src/commands/execution.rs`
- Modify: `src-tauri/src/services/variable_resolver.rs`

- [ ] **Step 1: Write tests for scenario execution flow**

Test the sequential execution with variable extraction and propagation. Test fail-fast behavior.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_extraction_from_body() {
        let body = r#"{"access_token": "abc123", "expires_in": 3600}"#;
        let extraction = Extraction {
            variable: "token".to_string(),
            from: "response.body".to_string(),
            path: "$.access_token".to_string(),
        };

        let result = apply_extraction(&extraction, body, &HashMap::new()).unwrap();
        assert_eq!(result, "abc123");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement dot-notation override application**

Step overrides use dot-notation keys (e.g., `body.warehouseId`, `headers.Authorization`) to modify the request before execution. Implement a function that navigates the request structure and applies the override value:

```rust
use serde_json::Value;

/// Apply dot-notation overrides to a request's JSON representation.
/// Keys like "body.warehouseId" navigate into nested fields.
/// Keys like "headers.Authorization" set header values directly.
pub fn apply_overrides(request_json: &mut Value, overrides: &HashMap<String, String>) {
    for (key, value) in overrides {
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        if parts.len() == 2 {
            if let Some(parent) = request_json.get_mut(parts[0]) {
                if let Some(obj) = parent.as_object_mut() {
                    // For body content, navigate into the "content" field
                    if parts[0] == "body" {
                        if let Some(content) = obj.get_mut("content") {
                            set_nested_value(content, parts[1], value);
                        }
                    } else {
                        obj.insert(parts[1].to_string(), Value::String(value.clone()));
                    }
                }
            }
        }
    }
}

fn set_nested_value(target: &mut Value, path: &str, value: &str) {
    let parts: Vec<&str> = path.splitn(2, '.').collect();
    match target {
        Value::Object(map) => {
            if parts.len() == 1 {
                map.insert(parts[0].to_string(), Value::String(value.to_string()));
            } else if let Some(child) = map.get_mut(parts[0]) {
                set_nested_value(child, parts[1], value);
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Implement scenario runner command**

The `run_scenario` command:
1. Acquires execution lock
2. Loads scenario and all referenced requests
3. Builds initial variable context (env + secrets + scenario inputs)
4. For each step:
   a. Resolve inheritance for the referenced request
   b. Apply step overrides via `apply_overrides`
   c. Resolve all `{{variables}}` in the assembled request
   d. Execute HTTP request
   e. If error (non-2xx or network error): emit error event, stop
   f. If success: apply extractions (using jsonpath on response body/headers), add to variable context
   g. Emit step-completed event with results
5. Release lock

Use Tauri events to stream step results to the frontend in real-time:

```rust
use tauri::{command, AppHandle, Emitter};

#[derive(Debug, Serialize, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: String,  // "success" | "error"
    pub response: Option<http_client::HttpResponse>,
    pub error: Option<String>,
    pub extracted: HashMap<String, String>,
}

#[command]
pub async fn run_scenario(
    app: AppHandle,
    project_path: PathBuf,
    scenario_path: String,
    env_file_name: String,
    inputs: HashMap<String, String>,
    timeout_ms: u64,
) -> Result<(), FlupiError> {
    acquire_lock()?;

    let result = execute_scenario(
        &app, &project_path, &scenario_path, &env_file_name, &inputs, timeout_ms
    ).await;

    release_lock();
    result
}
```

For JSONPath extraction, use `serde_json_path` (already in Cargo.toml from Task 1.1).

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add scenario runner with sequential execution, extraction, and fail-fast"
```

---

### Task 6.4: Scenario Runner — UI

**Files:**
- Create: `src/lib/components/scenarios/ScenarioRunner.svelte`
- Create: `src/lib/components/scenarios/PreRunForm.svelte`
- Create: `src/lib/components/scenarios/RunnerStepper.svelte`
- Create: `src/lib/components/scenarios/StepResultCard.svelte`
- Create: `src/lib/components/scenarios/VariableStatePanel.svelte`

- [ ] **Step 1: Build ScenarioRunner with pre-run form**

Shows all scenario inputs with defaults pre-filled. Unresolved `{{variable}}` tokens highlighted in red. "Run" button starts execution, "Back" returns to editor.

- [ ] **Step 2: Build RunnerStepper**

Vertical stepper showing step cards with states: waiting (grey), running (spinner), success (green check), error (red X). Each completed step shows status code, duration, extracted variables. Expandable to show full request/response.

- [ ] **Step 3: Build VariableStatePanel**

Live key-value table of all variables in context. Updates via Tauri events as steps complete. Secret values masked as `••••••`.

- [ ] **Step 4: Wire post-run actions**

"Run Again" returns to pre-run form with previous values. "Back to Editor" returns to scenario editor.

- [ ] **Step 5: Listen to Tauri events for real-time updates**

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<StepResult>('scenario-step-result', (event) => {
  // Update stepper state
});
```

- [ ] **Step 6: Verify full scenario execution flow end-to-end**

Create a multi-step scenario, run it, verify steps execute sequentially, variables extract and propagate, UI updates in real-time.

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add scenario runner UI with stepper, variable panel, and real-time updates"
```

---

## Phase 7: Referential Integrity

### Task 7.1: Auto-Update References on Rename/Move

**Files:**
- Create: `src-tauri/src/services/referential_integrity.rs`

- [ ] **Step 1: Write tests for reference scanning and updating**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_scenarios_referencing_request() {
        let dir = TempDir::new().unwrap();
        let scenarios_dir = dir.path().join("scenarios");
        std::fs::create_dir_all(&scenarios_dir).unwrap();

        let scenario = serde_json::json!({
            "name": "Test",
            "steps": [
                { "id": "s1", "name": "Step 1", "requestId": "auth/get-token", "overrides": {}, "extract": [] }
            ]
        });
        file_io::write_json(&scenarios_dir.join("test.json"), &scenario).unwrap();

        let refs = find_references(dir.path(), "auth/get-token").unwrap();
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_update_references() {
        let dir = TempDir::new().unwrap();
        let scenarios_dir = dir.path().join("scenarios");
        std::fs::create_dir_all(&scenarios_dir).unwrap();

        let scenario = serde_json::json!({
            "name": "Test",
            "steps": [
                { "id": "s1", "name": "Step 1", "requestId": "auth/get-token", "overrides": {}, "extract": [] }
            ]
        });
        file_io::write_json(&scenarios_dir.join("test.json"), &scenario).unwrap();

        update_references(dir.path(), "auth/get-token", "auth/login").unwrap();

        let updated: serde_json::Value = file_io::read_json(&scenarios_dir.join("test.json")).unwrap();
        assert_eq!(updated["steps"][0]["requestId"], "auth/login");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement referential integrity service**

```rust
use std::path::Path;
use crate::error::Result;
use crate::models::scenario::Scenario;
use crate::services::file_io;

pub fn find_references(project_path: &Path, request_id: &str) -> Result<Vec<std::path::PathBuf>> {
    let scenarios_dir = project_path.join("scenarios");
    let scenario_files = file_io::list_json_files(&scenarios_dir)?;
    let mut referencing = Vec::new();

    for file in scenario_files {
        let scenario: Scenario = file_io::read_json(&file)?;
        if scenario.steps.iter().any(|s| s.request_id == request_id) {
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
            if step.request_id == old_id {
                step.request_id = new_id.to_string();
                modified = true;
            }
        }

        if modified {
            file_io::write_json(&file, &scenario)?;
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Integrate into rename/move commands**

Modify `rename_request` and `move_request` commands to call `update_references` after the file system operation.

- [ ] **Step 5: Add delete confirmation with reference check**

Modify `delete_request` to call `find_references` and return the list of affected scenarios to the frontend for confirmation dialog.

- [ ] **Step 6: Run tests to verify they pass**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add referential integrity - auto-update scenario references on request rename/move"
```

---

## Phase 8: OpenAPI Import and Drift Detection

### Task 8.1: OpenAPI Models and Import Service

**Files:**
- Create: `src-tauri/src/models/openapi.rs`
- Create: `src-tauri/src/services/openapi_import.rs`
- Create: `src-tauri/src/commands/openapi.rs`

- [ ] **Step 1: Write tests for OpenAPI parsing and request generation**

Test parsing an OpenAPI 3.0 JSON spec, extracting operations, generating request files with correct method, path, body schema, and templateRef.

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement OpenAPI models**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenApiSources {
    pub sources: Vec<OpenApiSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum OpenApiSource {
    #[serde(rename = "url")]
    Url {
        id: String,
        name: String,
        url: String,
        #[serde(rename = "lastFetchedAt")]
        last_fetched_at: Option<String>,
        #[serde(rename = "lastHash")]
        last_hash: Option<String>,
    },
    #[serde(rename = "file")]
    File {
        id: String,
        name: String,
        path: String,
        #[serde(rename = "lastFetchedAt")]
        last_fetched_at: Option<String>,
        #[serde(rename = "lastHash")]
        last_hash: Option<String>,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct ImportableOperation {
    pub tag: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
}
```

- [ ] **Step 4: Implement OpenAPI import service**

Functions to: fetch spec from URL, read from file, parse operations, generate request JSON files, compute schema hash (SHA-256 of the serialized operation object).

- [ ] **Step 5: Implement OpenAPI Tauri commands**

Commands: `add_openapi_source`, `remove_openapi_source`, `fetch_operations` (returns importable operations for wizard), `import_operations` (generates request files), `refresh_source` (re-fetch and detect drift).

- [ ] **Step 6: Run tests to verify they pass**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add OpenAPI import service with operation parsing and request generation"
```

---

### Task 8.2: Drift Detection Service

**Files:**
- Create: `src-tauri/src/services/drift_detection.rs`

- [ ] **Step 1: Write tests for drift detection**

Test that hash comparison correctly identifies drifted requests. Test both URL and file-based sources.

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement drift detection**

On refresh: re-fetch/re-read the OpenAPI spec, for each request with a `templateRef` matching this source, compute the current operation's hash and compare to stored `schemaHash`. Return a list of drifted request IDs.

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add drift detection via schema hash comparison"
```

---

### Task 8.3: OpenAPI UI — Sources, Import Wizard, Drift Panel

**Files:**
- Create: `src/lib/stores/openapi.ts`
- Create: `src/lib/components/openapi/SourceList.svelte`
- Create: `src/lib/components/openapi/AddSourceForm.svelte`
- Create: `src/lib/components/openapi/ImportWizard.svelte`
- Create: `src/lib/components/requests/SchemaTab.svelte`
- Modify: `src/routes/(app)/openapi/+page.svelte` ← replace placeholder
- Create: `src/lib/components/openapi/DriftPanel.svelte`

- [ ] **Step 1: Build SourceList**

List of registered sources: name, URL/path, last fetched timestamp, drifted request count. "Add Source" button, per-source "Fetch & Sync" / "Re-scan" button, edit, delete. "Sync All" global button.

- [ ] **Step 2: Build AddSourceForm**

Form with input method tabs: URL (text input), File (OS file picker button), Drag-and-drop zone. Name + ID fields.

- [ ] **Step 3: Build ImportWizard**

Shows operations grouped by tag with checkboxes (select all / none per group). Destination collection selector. "Import" button generates request files.

- [ ] **Step 4: Build SchemaTab**

Tree view of `requestSchema` and `responseSchema` snapshots. Only shown for template-derived requests.

- [ ] **Step 5: Build DriftPanel**

Side-by-side view: left = current request config, right = new schema + auto-generated example. Field-level diff highlighting. "Mark as resolved" button.

- [ ] **Step 6: Add drift badges to RequestTree**

Red drift badge on drifted requests, propagated upward to parent folders and collections.

- [ ] **Step 7: Verify OpenAPI import and drift detection end-to-end**

Add an OpenAPI source, import operations, modify the source spec, refresh, verify drift badges appear, resolve drift.

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: add OpenAPI sources UI, import wizard, schema tab, and drift panel"
```

---

## Phase 9: Intellisense

### Task 9.1: Variable Token Autocomplete

**Files:**
- Create: `src/lib/components/shared/VariableAutocomplete.svelte`

- [ ] **Step 1: Build VariableAutocomplete component**

An input wrapper that shows a dropdown when `{{` is typed. Lists all variables in context: environment variables, secret variables (masked), scenario inputs, extracted variables from preceding steps. Shows variable name and current resolved value.

- [ ] **Step 2: Integrate into all fields that accept {{variable}} tokens**

URL, headers, body values, override values, input defaults, auth fields.

- [ ] **Step 3: Verify autocomplete works across all editors**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add variable token autocomplete for {{variable}} fields"
```

---

### Task 9.2: Override Key and JSONPath Autocomplete

**Files:**
- Modify: `src/lib/components/scenarios/OverridesPanel.svelte`
- Modify: `src/lib/components/scenarios/ExtractionsPanel.svelte`

- [ ] **Step 1: Add override key autocomplete from request schema**

When a step's request has a `requestSchema` snapshot, suggest dot-notation paths (e.g., `body.warehouseId`, `headers.X-Custom`) with type subtitles.

- [ ] **Step 2: Add extraction JSONPath autocomplete from response schema**

When a step's request has a `responseSchema` snapshot, suggest valid JSONPath expressions (e.g., `$.access_token`, `$.data.id`).

- [ ] **Step 3: Verify intellisense works for template-derived requests**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add schema-based autocomplete for override keys and extraction JSONPaths"
```

---

## Phase 10: Keyboard Shortcuts and Search

### Task 10.1: Keyboard Shortcuts

**Files:**
- Create: `src/lib/services/keyboard-shortcuts.ts`
- Modify: `src/routes/(app)/+layout.svelte` ← register shortcuts in the app shell's `onMount`

- [ ] **Step 1: Implement keyboard shortcut service**

```typescript
interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  handler: () => void;
}

export function registerShortcuts(shortcuts: Shortcut[]) {
  function handleKeydown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    for (const s of shortcuts) {
      if (e.key === s.key && !!ctrl === !!s.ctrl && !!e.shiftKey === !!s.shift) {
        e.preventDefault();
        s.handler();
        return;
      }
    }
  }

  window.addEventListener('keydown', handleKeydown);
  return () => window.removeEventListener('keydown', handleKeydown);
}
```

- [ ] **Step 2: Register all keyboard shortcuts**

| Action | Shortcut |
|---|---|
| Send request | `Ctrl/Cmd + Enter` |
| Force save | `Ctrl/Cmd + S` |
| New request | `Ctrl/Cmd + N` |
| Search requests | `Ctrl/Cmd + P` |
| Switch environment | `Ctrl/Cmd + E` |
| Run scenario | `Ctrl/Cmd + Shift + Enter` |

- [ ] **Step 3: Verify shortcuts work**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add keyboard shortcuts for common actions"
```

---

### Task 10.2: Request Search Modal

**Files:**
- Create: `src/lib/components/shared/SearchModal.svelte`

- [ ] **Step 1: Build SearchModal**

A command-palette-style modal triggered by `Ctrl/Cmd + P`. Lists all requests with fuzzy search by name and ID. Selecting a request navigates to it in the request library.

- [ ] **Step 2: Wire to keyboard shortcut**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: add request search modal (Ctrl+P)"
```

---

## Phase 11: Settings and Polish

### Task 11.1: Settings Screen

**Files:**
- Create: `src/lib/components/settings/SettingsPanel.svelte`
- Modify: `src/routes/(app)/settings/+page.svelte` ← replace placeholder

- [ ] **Step 1: Build SettingsPanel**

Two sections:
- **Project:** project display name (derived from folder, read-only), folder path (read-only)
- **App:** theme selector (Dark / Light / System, default Dark), default request timeout in ms

Theme changes apply immediately and persist to `preferences.json`. Timeout changes save via debounced save.

- [ ] **Step 2: Wire theme store to preferences persistence**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: add settings panel with theme and timeout configuration"
```

---

### Task 11.2: Empty States and Error States

**Files:**
- Modify: Various components

- [ ] **Step 1: Add empty states to all list/tree views**

- Environments: "No environments yet. Create one to get started."
- Requests: "No requests yet. Create a request or import from OpenAPI."
- Scenarios: "No scenarios yet. Create one to chain requests together."
- OpenAPI Sources: "No sources registered. Add one to import API operations."

- [ ] **Step 2: Add error handling for failed operations**

Toast notifications or inline error messages for: file write failures, HTTP errors during OpenAPI fetch, invalid JSON, etc.

- [ ] **Step 3: Add loading states**

Skeleton loaders or spinners for: project loading, environment switching, OpenAPI fetching.

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add empty states, error handling, and loading indicators"
```

---

### Task 11.3: Branding and Final Polish

**Files:**
- Modify: `src/app.css`
- Modify: `src/lib/components/layout/TopBar.svelte`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Apply brand styling**

- Accent color: electric cyan or violet
- Monospace font for request/response data
- Clean sans-serif for UI chrome
- Window title: "Flupi"

- [ ] **Step 2: Configure Tauri window**

```json
{
  "app": {
    "windows": [
      {
        "title": "Flupi",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ]
  }
}
```

- [ ] **Step 3: Add app icon**

Create or source a Flupi icon (flowing arrow / chain link concept). Generate all required icon sizes for Tauri.

- [ ] **Step 4: Final smoke test**

Walk through the full user journey: launch → create project → add environment → create collection → create request → send request → create scenario → run scenario → verify everything works.

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: apply Flupi branding, icons, and final polish"
```

---

## Summary

| Phase | Tasks | Description |
| --- | --- | --- |
| 1 | 1.1–1.6 | Project scaffold, file I/O, project init, app data, project picker, main layout |
| 2 | 2.1–2.2 | Environment models, secrets, CRUD, UI |
| 3 | 3.1–3.5 | Request/collection models, CRUD, tree, editor, DnD |
| 4 | 4.1–4.3 | Variable resolver, inheritance, effective request preview |
| 5 | 5.1–5.2 | HTTP client, send command, execution lock |
| 6 | 6.1–6.4 | Scenario model, editor, runner backend, runner UI |
| 7 | 7.1 | Referential integrity — auto-update references |
| 8 | 8.1–8.3 | OpenAPI import, drift detection, UI |
| 9 | 9.1–9.2 | Intellisense — variable, override, JSONPath autocomplete |
| 10 | 10.1–10.2 | Keyboard shortcuts, request search modal |
| 11 | 11.1–11.3 | Settings, empty/error states, branding |
