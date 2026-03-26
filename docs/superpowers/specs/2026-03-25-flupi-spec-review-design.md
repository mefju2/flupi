# Flupi Spec Review — Design Addendum

This document captures decisions and clarifications from the spec review of `flupi-spec.md`. These amendments should be applied to the main spec before implementation begins.

---

## 1. Scenario Error Handling

**Decision:** Fail-fast — the scenario runner stops execution on the first step that errors.

- When a step returns a non-2xx status or a network/timeout error, execution halts immediately.
- HTTP redirects (3xx) are followed automatically by the HTTP client (`reqwest`). The final status code after redirect resolution is what determines success or error.
- All subsequent steps remain in `waiting` state.
- The user sees the error details on the failed step and can choose "Run Again" or "Back to Editor."
- "Run Again" returns to the pre-run input form with the previous values pre-filled, then re-runs the entire scenario from step 1. It does not skip the input form or resume from the failed step.
- Rationale: steps are sequential and typically depend on extracted variables from prior steps. A failed step means downstream variables won't be populated, making continuation unreliable.
- Note: v1 does not support per-step expected status codes. All non-2xx responses (after redirect resolution) are treated as errors. If a step legitimately expects a non-2xx response (e.g., a 404 to confirm something doesn't exist), the user must accept that the scenario will stop there. Per-step expected status codes may be considered for v2.

---

## 2. Request Timeouts

**Decision:** Global timeout only for v1. Timeout triggers a standard error.

- The global default timeout (configured in Settings, in milliseconds) applies to all requests.
- No per-request timeout override in v1.
- When a request exceeds the timeout, it is treated as an error: the response panel shows a timeout message, and in scenario mode it triggers fail-fast.

---

## 3. Request IDs — Derived at Runtime

**Decision:** Remove the stored `id` field from request JSON files. Derive IDs at runtime from file paths.

**Derivation rules:**

Request IDs are derived from the filesystem path relative to the project root. The ID uses the **folder name on disk** (not the display `name` from `collection.json`).

- Collection request: `collections/{folderName}/requests/[{subpath}/]{fileName}.json` → `{folderName}/[{subpath}/]{fileName}`
  - Example: `collections/auth-service/requests/get-token.json` → `auth-service/get-token`
  - Example with nesting: `collections/auth-service/requests/admin/get-token.json` → `auth-service/admin/get-token`
  - Note: `folderName` is `auth-service` (the directory name), not `"Auth Service"` (the `name` field in `collection.json`)
- Root-level request: `requests/[{subpath}/]{fileName}.json` → `[{subpath}/]{fileName}`
  - Example: `requests/health-check.json` → `health-check`
  - Example with nesting: `requests/monitoring/health-check.json` → `monitoring/health-check`

`{subpath}` is optional — it is only present when the request is in a nested subfolder.

**Benefits:** No stale IDs, no need to update file contents on rename/move, one less field to keep in sync.

**Main spec updates required:**

- Section 5.5: Remove the `"id"` field from the request file schema example. The auth examples (`{{token}}`, `{{pass}}`, etc.) remain valid — these variables now come from `*.secrets.json` instead of `.env.local` but the template syntax is unchanged.
- Section 5.6: The `requestId` values in the scenario schema example (e.g., `"auth/get-token"`) remain valid under the new derivation rules — no changes needed to scenario examples.

---

## 4. Referential Integrity

**Decision:** Auto-update references on rename/move, warn on delete.

- When a request is renamed or moved via the UI, Flupi automatically updates all `requestId` references in scenario files.
- When a request is deleted, Flupi scans for referencing scenarios and shows the user a confirmation dialog listing affected scenarios before proceeding.
- Broken references (e.g., from manual file edits outside Flupi) are shown as warnings in the scenario editor — a badge on the step card indicating the referenced request was not found.
- When moving a request between collections via drag-and-drop or "Move to...", the UI shows an informational notice that the request's inherited auth and headers may change (since it will now inherit from a different collection). This is informational only — the move proceeds regardless.

---

## 5. Per-Environment Secrets

**Decision:** Replace the single `.env.local` with per-environment `*.secrets.json` files.

### Updated folder structure

```
/my-project/
├── .gitignore                        ← auto-generated, includes *.secrets.json
├── openapi-sources.json
├── openapi/                          ← copied OpenAPI spec files (see Section 9)
│   └── auth-service.json
├── environments/
│   ├── dev.json
│   ├── dev.secrets.json              ← gitignored
│   ├── staging.json
│   └── staging.secrets.json          ← gitignored
├── collections/
│   └── auth-service/
│       ├── collection.json
│       └── requests/
│           ├── get-token.json
│           └── admin/                ← nested folders allowed (see Section 10)
│               └── create-user.json
├── requests/                         ← root-level requests, nesting allowed
│   └── health-check.json
└── scenarios/
    ├── regression/                   ← nested groups allowed (see Section 12)
    │   └── api/
    │       └── auth-flow.json
    └── smoke-tests/
        └── basic-flow.json
```

`.gitignore` includes `*.secrets.json`. The previous `.env.local` file is removed from the project model entirely. The auto-generated `.gitignore` no longer includes `.env.local` — it is replaced by `*.secrets.json`.

### Secrets UX model

- Secrets are managed within the environment editor, not a separate screen.
- Each environment variable has a "secret" toggle. When toggled on, the value is stored in the corresponding `*.secrets.json` file instead of the main environment JSON.
- The committed `dev.json` retains the key with an empty value (e.g., `"client_secret": ""`), so team members can see which secrets are required.
- **Secret key loading:** When loading an environment, keys listed in the `secrets` array are excluded from the variables map entirely. They are loaded only from the `*.secrets.json` file. The empty placeholder in the committed JSON is never loaded into the runtime variable context.
- If the `*.secrets.json` file does not exist or does not contain the key, the variable is treated as **unresolved** — it shows the red unresolved highlight in the UI, same as any other missing variable. The empty placeholder in the committed JSON is never used as a fallback.
- On first clone, a teammate opens the environment editor, sees empty secret fields, fills them in, and the `*.secrets.json` file is auto-created on save.
- Secret values are masked by default in the UI with a toggle to reveal.

### Updated environment file schema

**`environments/dev.json`:**

```json
{
  "name": "Development",
  "variables": {
    "baseUrl": "https://api.dev.internal",
    "warehouseId": "WH-DEV-01",
    "client_secret": ""
  },
  "secrets": ["client_secret"]
}
```

The `secrets` array lists which keys have their values stored in the secrets file. For any key in this array, the value in `variables` is ignored — it exists only as a placeholder for discoverability.

**`environments/dev.secrets.json`:**

```json
{
  "client_secret": "supersecret-dev-value"
}
```

### Updated variable resolution order

At runtime, variables are resolved from a merged context built in this priority order (later sources win on conflict):

1. Active environment file `variables` (keys listed in `secrets` array are excluded at load time — they never enter the context from this source)
2. Active environment secrets file (`*.secrets.json`)
3. Scenario-level inputs (provided by the user in the pre-run form)
4. Extracted variables accumulated from completed steps

**Secret key resolution:** The `secrets` array acts as a routing mechanism at the environment layer only (steps 1–2). Keys in the `secrets` array are sourced exclusively from `*.secrets.json` (step 2), never from the committed `variables` (step 1). However, scenario-level inputs (step 3) and extracted variables (step 4) can still override any key, including secret keys — they operate on the merged context and are not affected by the `secrets` array filter. This means a scenario input or extraction can intentionally override a secret value for testing purposes.

### Main spec updates required

- **Section 4.2** (folder structure): Replace with the updated folder structure above. This also adds the `openapi/` directory (see Section 9). Update the `.gitignore` description to reference `*.secrets.json` instead of `.env.local`.
- **Section 5.1** (`.env.local` schema): Remove entirely — replaced by per-environment `*.secrets.json` files described above.
- **Section 5.3** (openapi-sources.json schema): Update to include the `type` field (see Section 9).
- **Section 5.5** (request file schema): Auth examples remain valid — `{{token}}`, `{{pass}}` etc. now come from secrets files instead of `.env.local` but the template syntax is unchanged.
- **Section 6** (variable resolution): Replace the resolution order with the one above.
- **Section 8.1** (adding a source): Update to mention file-based sources and drag-and-drop in addition to URL (see Section 9).
- **Section 9.3** (Environments screen): Remove the read-only notice about `.env.local`. Replace with: the environment editor shows a "secret" toggle per variable.
- **Section 9.6** (Scenarios sidebar): Update to reflect nested group navigation in the sidebar tree.
- **Section 9.7** (Scenario Runner variable state panel): Change "Secrets from `.env.local` shown masked" to "Secret variables (from `*.secrets.json`) shown masked as `••••••`."
- **Section 10.3** (Variable token autocomplete): Change "`.env.local` keys (masked)" to "secret variables (masked)."

---

## 6. Auto-Save

**Decision:** Auto-save with ~500ms debounce across all editors.

- Applies to: environment editor, collection editor, request editor, scenario editor.
- Changes are written to disk after 500ms of inactivity (no further edits).
- `Ctrl/Cmd + S` forces an immediate flush of the debounce timer and saves to disk, with a brief visual "Saved" indicator.
- No other explicit "Save" button. The UI may show a subtle "Saved" indicator after auto-save writes complete.
- Since all data is JSON on disk and Git tracks history, there is no risk of losing prior versions.

---

## 7. Concurrent Execution

**Decision:** One execution at a time for v1.

- While any execution is in progress (scenario run or ad-hoc request send), all other "Send" buttons and "Run" buttons are disabled.
- This applies in both directions: a running scenario blocks ad-hoc sends, and an in-flight ad-hoc send blocks scenario runs.
- The UI shows a clear indicator that an execution is in progress.
- Rationale: the variable context is shared mutable state — concurrent executions would introduce race conditions.

---

## 8. Response Body Handling

**Decision:** Truncate large responses, metadata-only for non-text, in-memory only.

- **Large responses:** Responses above 1MB are truncated in the rendered view. A "Show full" and "Copy to clipboard" option is available.
- **Non-text responses:** Binary content (images, PDFs, etc.) shows content-type and size only. No inline rendering for v1.
- **Persistence:** Responses are kept in-memory only for the current session. They are not written to disk. Closing the app or switching projects clears all response data.

---

## 9. OpenAPI Source Import

**Decision:** No authentication for v1. Support URL, local file path, and drag-and-drop.

### Input methods

1. **URL** — fetches the spec from a remote endpoint (unauthenticated)
2. **File path** — user selects a local JSON file via OS file picker
3. **Drag-and-drop** — user drags a JSON file onto the sources panel

For file-based imports (file picker or drag-and-drop), the file is copied into the `openapi/` directory at the project root. This directory is auto-created on first file-based import. The copied file is Git-trackable and self-contained.

### Updated source schema

**Main spec Section 5.3 must be updated** to include the `type` field.

Each source must have a unique `id`. Flupi enforces uniqueness — if a user tries to add a source with an ID that already exists, the UI shows an error and prompts them to choose a different ID or update the existing source.

URL-based source:

```json
{
  "id": "auth-service",
  "name": "Auth Service",
  "type": "url",
  "url": "https://auth.internal/openapi.json",
  "lastFetchedAt": "2026-03-20T10:00:00Z",
  "lastHash": "sha256:a3f9..."
}
```

File-based source:

```json
{
  "id": "auth-service",
  "name": "Auth Service",
  "type": "file",
  "path": "openapi/auth-service.json",
  "lastFetchedAt": "2026-03-20T10:00:00Z",
  "lastHash": "sha256:a3f9..."
}
```

The `path` field is relative to the project root.

### Drift detection for file-based sources

For URL-based sources, drift detection works as described in the main spec (re-fetch on startup and manual refresh).

For file-based sources:

- On startup and manual "Refresh," Flupi re-reads the local file from the `openapi/` directory and recomputes the hash.
- If the user updates the source file outside Flupi (e.g., downloads a newer version and replaces it), drift is detected on next refresh.
- The "Fetch & Sync" button label changes to "Re-scan" for file-based sources.
- To update the source file itself, the user can drag-and-drop a new version or use the file picker — the old file in `openapi/` is overwritten.

**Main spec updates required:**

- **Section 4.2** (folder structure): Add `openapi/` directory to the project structure (also covered in Section 5's folder structure update).
- **Section 5.3** (openapi-sources.json): Update schema to include `type` and `path` fields.
- **Section 8.1** (adding a source): Update to describe URL, file picker, and drag-and-drop input methods.

---

## 10. Collection Nesting

**Decision:** Allow nested folders within collections.

```
collections/
└── auth-service/
    ├── collection.json
    └── requests/
        ├── get-token.json
        └── admin/
            ├── create-user.json
            └── delete-user.json
```

- The sidebar tree reflects the folder nesting.
- Request IDs include the subfolder path: `auth-service/admin/create-user`.
- Nested folders can be created via right-click context menu ("New Folder").

Root-level `requests/` also supports nested folders:

```
requests/
├── health-check.json
└── monitoring/
    └── status.json
```

Root-level nested request IDs: `monitoring/status`.

---

## 11. Drag-and-Drop in Request Library

**Decision:** Full drag-and-drop support in the sidebar tree.

- Requests can be dragged between collections, between folders within a collection, and to/from root-level requests.
- Dragging triggers the auto-update of `requestId` references in all scenarios.
- When moving between collections, an informational notice is shown about inherited auth/header changes (see Section 4).
- Right-click "Move to..." remains available as an alternative for both requests and scenarios. This replaces "Move to group" from the main spec Section 9.6 — the label is generalized since scenarios can now be moved to nested groups.
- Uses `svelte-dnd-action` (already in the stack for scenario step reordering).

---

## 12. Scenario Group Nesting

**Decision:** Allow nested scenario groups.

```
scenarios/
├── regression/
│   ├── api/
│   │   └── auth-flow.json
│   └── ui/
│       └── login-flow.json
└── smoke-tests/
    └── basic-flow.json
```

- Consistent with the nested collection folders decision.
- The sidebar tree reflects the full nesting.
- Groups can be created at any depth via right-click context menu.

**Main spec updates required:**

- **Section 9.6** (Scenarios sidebar): Update "Scenario groups (folders) → scenarios" to reflect that groups can be nested to arbitrary depth.

---

## 13. Duplication

**Decision:** "Duplicate" option in right-click context menu for requests and scenarios.

- Creates a copy of the file with a `-copy` suffix (e.g., `get-token-copy.json`).
- If a `-copy` file already exists, a numeric suffix is appended: `-copy-2`, `-copy-3`, etc.
- The duplicate opens immediately in the editor for renaming and editing.
- Scenario duplicates copy all steps, inputs, and configuration.
- Duplication applies to individual request files and scenario files only — not to collections or folders. To duplicate an entire collection, users should copy the folder via Git or their file manager.

---

## 14. Keyboard Shortcuts

The following keyboard shortcuts should be supported:

| Action | Shortcut |
|---|---|
| Send request | `Ctrl/Cmd + Enter` |
| Force save (flushes debounce, writes immediately) | `Ctrl/Cmd + S` |
| New request | `Ctrl/Cmd + N` |
| Search requests | `Ctrl/Cmd + P` |
| Switch environment | `Ctrl/Cmd + E` |
| Run scenario | `Ctrl/Cmd + Shift + Enter` |

---

## 15. Context Menu Consolidation

The right-click context menus across the app should include the following items. These replace the context menu definitions in main spec Sections 9.5 and 9.6.

**Request library sidebar — on a request:**
New Request / New Folder / Rename / Duplicate / Move to... / Delete

**Request library sidebar — on a collection:**
New Request / New Folder / Rename / Delete

Collections are top-level entities and cannot be moved or duplicated via the UI. To reorganize or duplicate collections, users should use Git or their file manager.

**Request library sidebar — on a folder (within a collection or root-level requests):**
New Request / New Folder / Rename / Move to... / Delete

**Scenario sidebar — on a scenario:**
Rename / Duplicate / Move to... / Delete

**Scenario sidebar — on a group (folder):**
New Scenario / New Group / Rename / Move to... / Delete

---

## 16. Theme

**Decision:** Dark mode by default, light mode supported.

- Flupi launches in dark mode on first use.
- Theme can be changed in Settings: Dark / Light / System (follows OS preference).
- The main spec Section 9.8 already lists "theme (dark / light / system)" — this addendum confirms dark is the default.

**Main spec updates required:**

- **Section 3** (Branding Direction): Already says "dark-mode-first UI" — no change needed.
- **Section 9.8** (Settings): Add that the default is dark mode.

---

## 17. App Data & First Launch Experience

### Flupi app data directory

Flupi stores its own application state (not project data) in the platform-standard app data directory managed by Tauri:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.flupi.app/` |
| Windows | `%APPDATA%\com.flupi.app\` |
| Linux | `~/.config/com.flupi.app/` |

This directory contains:

- **`recent-projects.json`** — list of recently opened projects with their disk paths, last opened timestamp, and display name
- **`preferences.json`** — app-level settings (theme, default timeout, window size/position)

This directory is **not** a Git repository and does not contain project data. Project data lives entirely within the user-chosen project folder.

### `recent-projects.json` schema

```json
{
  "projects": [
    {
      "name": "My API Tests",
      "path": "/Users/jane/projects/my-api-tests",
      "lastOpenedAt": "2026-03-25T14:30:00Z"
    }
  ]
}
```

### First launch experience

When Flupi is opened for the very first time (no `recent-projects.json` exists or it's empty):

1. The **Project Picker** screen is shown with an empty state — no recent projects listed.
2. The empty state shows a welcome message and two prominent actions:
   - **"New Project"** — opens an OS folder picker. The user selects (or creates) a directory. Flupi initializes the folder structure (`.gitignore`, `environments/`, `collections/`, `requests/`, `scenarios/`) in that directory.
   - **"Open Folder"** — opens an OS folder picker. The user selects an existing directory. Flupi loads it as a project (creating any missing structural folders as needed).
3. The chosen path is saved to `recent-projects.json` and the project opens.

### Default project path

When the user clicks "New Project," the OS folder picker opens to the platform-standard documents or home directory (Tauri default behavior). There is no Flupi-specific default project directory — projects live wherever the user chooses to put them. This is consistent with the Git-native philosophy: projects are just folders on disk that the user manages with Git.

### Project validation

When opening a folder (whether from "Open Folder" or the recent projects list):

- If the folder doesn't exist (e.g., it was deleted or moved since last opened), Flupi shows an error and removes it from the recent projects list.
- If the folder exists but has no Flupi structure, Flupi offers to initialize it ("This folder doesn't appear to be a Flupi project. Initialize it?").
- If the folder has a partial structure (e.g., has `environments/` but no `scenarios/`), Flupi creates the missing directories silently.

**Main spec updates required:**

- **Section 4.1** (Multi-project support): Already covers the project picker and recent projects. Add details about the app data directory path and `recent-projects.json` schema. Clarify that the recent projects list and app preferences are stored in Tauri's app data directory, not in any project folder.
- **Section 9.1** (Project Picker): Add the first-launch empty state and welcome message. Add project validation behavior (missing folder, non-Flupi folder, partial structure).

---

## 18. Items Not in v1 (Updated)

The following are added to the explicit v1 exclusions:

- Per-request timeout overrides (global timeout only)
- Per-step expected status codes (all non-2xx is an error)
- Concurrent request/scenario execution (single execution at a time)
- Inline rendering of non-text response bodies (metadata only)
- Authenticated OpenAPI source fetching (URL must be publicly accessible, or use file import)
- Environment-scoped collection visibility
- Import/export of secrets templates (teammates set up secrets manually via UI)
- Collection/folder duplication (only individual request and scenario files)
- Collection move/reorder (collections are top-level, managed via filesystem)
