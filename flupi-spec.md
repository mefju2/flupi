# Flupi — Product Specification

## 1. Overview

**Flupi** is a desktop application for creating and running API test scenarios. It is built for development teams who need to chain HTTP requests in a defined sequence, pass data between steps, and version-control everything in Git.

The name derives from *flux* (flow) + *api* via verlan — data flowing through an API pipeline.

### Core principles
- **Git-native** — all project configuration is stored as human-readable JSON files. No database, no proprietary binary formats. Teams collaborate via Git pull/push.
- **UI-first** — every operation is available from the UI. Users never need to edit JSON files manually.
- **Sequential scenarios** — test flows are ordered step-by-step. No branching or conditional logic in v1.
- **OpenAPI-aware** — requests can be imported from OpenAPI/Swagger sources and tracked for schema drift.

---

## 2. Tech Stack

| Layer | Choice |
|---|---|
| Desktop framework | Tauri (Rust backend + web frontend) |
| Frontend | Svelte + TypeScript |
| Styling | Tailwind CSS + shadcn-svelte |
| File I/O | Tauri `fs` plugin (reads/writes local JSON files) |
| HTTP client | Rust `reqwest` via Tauri command (executes requests) |
| JSONPath | `jsonpath-plus` (npm, used in frontend for extraction preview and intellisense) |
| Drag and drop | `svelte-dnd-action` (step reordering in scenario editor) |

---

## 3. Branding Direction

**Name:** Flupi
**Tone:** Friendly, developer-focused, slightly playful — not corporate.

**Visual direction:**
- Dark-mode-first UI (standard preference in developer tooling)
- Accent color: electric cyan or violet — suggests data flow and energy
- Logo concept: stylized flowing arrow or chain link, reflecting the "flux through API steps" etymology
- Typography: monospace for request/response data; clean sans-serif for UI chrome

---

## 4. Project Model

Flupi works with **project folders**. A project folder is any directory the user opens in Flupi. The folder structure within it is managed entirely by Flupi via the UI. Users never need to edit files manually.

### 4.1 Multi-project support
- On launch, Flupi shows a **project picker** — a list of recently opened projects
- "Open Folder" lets the user open any existing directory as a project
- "New Project" initialises a fresh folder structure in a chosen directory
- The active project name is shown in the top bar at all times
- Users can switch projects at any time via the top bar
- Recently opened projects are stored in Tauri's local app data (outside the project folder — not committed to Git)

### 4.2 Folder structure

```
/my-project/
├── .gitignore                        ← auto-generated on first open
├── .env.local                        ← gitignored, secrets only
├── openapi-sources.json              ← registered OpenAPI source URLs
├── environments/
│   ├── dev.json
│   └── staging.json
├── collections/
│   └── auth-service/
│       ├── collection.json           ← collection-level config (auth, headers, baseUrl)
│       └── requests/
│           ├── get-token.json
│           └── refresh-token.json
├── requests/                         ← root-level requests (no collection)
│   └── health-check.json
└── scenarios/
    ├── smoke-tests/                  ← scenario group (plain folder)
    │   └── basic-flow.json
    └── warehouse/
        └── full-warehouse-flow.json
```

Flupi generates a `.gitignore` at project root on first open if one does not already exist, ensuring `.env.local` is always excluded.

---

## 5. File Schemas

### 5.1 `.env.local`
Plain `key=value` format. Never committed to Git.
```
clientId=abc123
clientSecret=supersecret
```

### 5.2 `environments/dev.json`
```json
{
  "name": "Development",
  "variables": {
    "baseUrl": "https://api.dev.internal",
    "warehouseId": "WH-DEV-01"
  }
}
```

### 5.3 `openapi-sources.json`
```json
{
  "sources": [
    {
      "id": "auth-service",
      "name": "Auth Service",
      "url": "https://auth.internal/openapi.json",
      "lastFetchedAt": "2026-03-20T10:00:00Z",
      "lastHash": "sha256:a3f9..."
    }
  ]
}
```

### 5.4 `collections/{name}/collection.json`
```json
{
  "name": "Auth Service",
  "baseUrl": "{{baseUrl}}",
  "auth": {
    "type": "bearer",
    "token": "{{token}}"
  },
  "headers": {
    "Content-Type": "application/json"
  }
}
```

### 5.5 Request file (`collections/{name}/requests/*.json` or `requests/*.json`)

```json
{
  "id": "auth/get-token",
  "name": "Get Token",
  "method": "POST",
  "path": "/auth/token",
  "auth": {
    "type": "none"
  },
  "headers": {
    "Content-Type": "application/x-www-form-urlencoded"
  },
  "body": {
    "type": "form",
    "content": {
      "clientId": "{{clientId}}",
      "secret": "{{secret}}"
    }
  },
  "templateRef": {
    "sourceId": "auth-service",
    "operationId": "postAuthToken",
    "schemaHash": "sha256:a3f9...",
    "requestSchema": {},
    "responseSchema": {}
  }
}
```

`templateRef` is present only on requests imported from an OpenAPI source. `requestSchema` and `responseSchema` are snapshots of the OpenAPI operation's schema at import time, used for intellisense and drift detection. If `auth` is absent at request level, the parent collection's auth is inherited.

**Auth type variants:**
```json
{ "type": "none" }
{ "type": "inherit" }
{ "type": "bearer", "token": "{{token}}" }
{ "type": "basic", "username": "{{user}}", "password": "{{pass}}" }
{ "type": "apiKey", "header": "X-API-Key", "value": "{{apiKey}}" }
{ "type": "custom", "headers": { "X-Tenant": "{{tenantId}}" } }
```

`"inherit"` explicitly inherits from the collection. `"none"` explicitly disables auth even if the collection has one configured.

**Body type variants:**
```json
{ "type": "json", "content": {} }
{ "type": "form", "content": { "key": "value" } }
{ "type": "raw", "content": "plain string" }
{ "type": "none" }
```

### 5.6 Scenario file (`scenarios/**/*.json`)

```json
{
  "name": "Full Warehouse Flow",
  "inputs": [
    {
      "name": "warehouseId",
      "description": "Target warehouse identifier",
      "default": "{{warehouseId}}",
      "required": true
    },
    {
      "name": "duration",
      "description": "Simulation duration in seconds",
      "default": "60",
      "required": false
    }
  ],
  "steps": [
    {
      "id": "step-1",
      "name": "Get Token",
      "requestId": "auth/get-token",
      "overrides": {},
      "extract": [
        {
          "variable": "token",
          "from": "response.body",
          "path": "$.access_token"
        }
      ]
    },
    {
      "id": "step-2",
      "name": "Stop Simulation",
      "requestId": "simulation/stop",
      "overrides": {
        "headers.Authorization": "Bearer {{token}}"
      },
      "extract": []
    },
    {
      "id": "step-3",
      "name": "Start Simulation",
      "requestId": "simulation/start",
      "overrides": {
        "body.warehouseId": "{{warehouseId}}",
        "body.duration": "{{duration}}"
      },
      "extract": []
    }
  ]
}
```

---

## 6. Variable Resolution

At runtime, variables are resolved from a merged context built in this priority order (later sources win on conflict):

1. `.env.local` key=value pairs
2. Active environment file `variables`
3. Scenario-level inputs (provided by the user in the pre-run form)
4. Extracted variables accumulated from completed steps

Any `{{variableName}}` token in a URL, header value, body field, or auth field is substituted from this context before the request is sent.

**Unresolved variables** (no value found in context) are highlighted in red in the UI — in the request editor, scenario step editor, and the pre-run form. The runner warns before starting if any unresolved variables are detected.

---

## 7. Inheritance Resolution

When a request is executed (ad-hoc or as part of a scenario), the effective request is assembled as follows:

1. Start with the request file's own fields
2. If `request.auth` is `"inherit"` or absent, apply `collection.auth`
3. Merge `collection.headers` with `request.headers` — request-level headers win on key conflicts
4. If `request.path` is relative, prepend `collection.baseUrl`; if absolute, use as-is
5. Apply step `overrides` on top of the merged result using dot-notation keys (e.g. `body.warehouseId`, `headers.Authorization`)
6. Resolve all `{{variables}}` from the variable context

The UI shows an **"Effective Request" tab** on every request — a read-only preview of the fully assembled, inheritance-applied, variable-substituted request for the active environment.

---

## 8. OpenAPI Import & Drift Detection

### 8.1 Adding a source
The user registers an OpenAPI source by providing a name and a URL pointing to a JSON OpenAPI spec (typically `/openapi.json` or `/v3/api-docs`). Sources are stored in `openapi-sources.json`.

### 8.2 Import wizard
1. Flupi fetches the OpenAPI JSON from the source URL
2. The import wizard displays all operations grouped by tag
3. The user selects which operations to import (checkboxes, select all / none per group)
4. For each selected operation, Flupi generates a request file pre-filled with:
   - `method` and `path`
   - Path/query parameters with defaults from the schema
   - Body structure matching the request schema
   - `templateRef` containing `sourceId`, `operationId`, `schemaHash` (SHA-256 of the operation object), and snapshots of `requestSchema` and `responseSchema`
5. The user selects a destination collection or root folder
6. Files are written and the requests appear immediately in the library

### 8.3 Drift detection
- On app startup and on manual "Refresh" per source, Flupi re-fetches all source URLs
- For each request with a `templateRef`, Flupi computes the current operation's schema hash and compares to the stored `schemaHash`
- If they differ, the request is marked **drifted**
- A red drift badge 🔴 propagates upward from the request to its parent folder and collection in the sidebar tree

### 8.4 Drift resolution UI
When a user opens a drifted request, a drift panel appears:
- **Left side:** the current request as configured in Flupi
- **Right side:** the new schema from the source + an auto-generated example request body based on the new schema
- Field-level diff highlighting between left and right
- The user manually edits the left side to reconcile
- "Mark as resolved" updates `schemaHash` to the new value and clears the drift state

---

## 9. Screens

### 9.1 Project Picker (launch screen)
- List of recently opened projects (name + folder path)
- "Open Folder" — OS folder picker, loads any folder as a project
- "New Project" — OS folder picker, then initialises folder structure and `.gitignore`
- Recent project list stored in Tauri local app data (not in the project folder)

### 9.2 Main layout
Once a project is open:
- **Top bar:** Flupi logo, active project name, environment switcher dropdown, "Switch Project" button
- **Left sidebar:** navigation icons for the 5 main sections (Environments, OpenAPI Sources, Requests, Scenarios, Settings)
- **Main content area:** changes per active section

### 9.3 Environments
- List of all environment files; active one highlighted
- Click any environment to make it active (immediate switch)
- Edit variables via a key-value table: add row, edit key/value inline, delete row
- "New Environment" button creates a new file
- Read-only notice: *"Secrets go in `.env.local` at project root — this file is gitignored and never committed."*
- All edits auto-save to the environment JSON file

### 9.4 OpenAPI Sources
- List of registered sources: name, URL, last fetched timestamp, drifted request count
- "Add Source" — form for name + URL
- Per-source: "Fetch & Sync" button, edit, delete
- "Sync All" global button
- After sync: each source shows count of newly drifted requests

### 9.5 Requests Library

**Layout:** left sidebar tree + right content panel

**Left sidebar tree:**
- Collections → (folders →) requests
- Root-level requests below collections
- Drift badge 🔴 on requests, propagated upward to collections
- "New Collection" button
- Right-click context menu on any node: New Request / New Folder / Rename / Delete

**Request editor (right panel) — tabs:**

- **Params:** URL path parameters and query parameters as a key-value table; values support `{{variable}}` tokens with autocomplete
- **Headers:** key-value table; inherited headers from collection shown as read-only with an "inherited" badge; own headers editable
- **Auth:** dropdown selector (Inherit / None / Bearer / Basic / API Key / Custom) + relevant fields; shows resolved inherited auth when "Inherit" selected
- **Body:** body type selector (None / JSON / Form / Raw) + content editor; JSON type uses a code editor with syntax highlighting
- **Schema** *(template-derived requests only):* readable tree view of `requestSchema` and `responseSchema` snapshots; drift panel appears here when drifted
- **Effective Request:** read-only preview of the fully assembled request after inheritance and variable resolution for the active environment

**Ad-hoc send:**
- "Send" button executes the request immediately using the active environment
- Response panel below: status code, response time, response headers, body (pretty-printed JSON or raw text)

**Collection editor:**
- Clicking a collection name opens collection-level settings in the right panel
- Fields: Name, Base URL, Auth (same auth selector), default Headers
- All changes auto-save

### 9.6 Scenarios

**Layout:** left sidebar tree + right content panel

**Left sidebar tree:**
- Scenario groups (folders) → scenarios
- "New Group" and "New Scenario" buttons
- Scenarios can be dragged between groups
- Right-click context menu: Rename / Delete / Move to group

**Scenario editor (right panel):**

*Inputs section:*
- List of scenario-level input definitions
- Each input: name (identifier used as `{{name}}`), description, default value, required toggle
- "Add Input" button; inputs can be reordered and deleted

*Steps section:*
- Ordered list of step cards with drag handles for reordering
- Each collapsed card shows: step number, step name, request method + path
- "Add Step" button → searchable dropdown of all requests in the library
- Expanding a step card reveals:
  - **Step name** — editable inline label
  - **Overrides panel** — key-value table. Key field: dot-notation path (e.g. `body.warehouseId`, `headers.Authorization`) with intellisense autocomplete from `requestSchema`. Value field: free text supporting `{{variable}}` tokens with autocomplete from available variables.
  - **Extractions panel** — list of extraction definitions. Each row: variable name, source selector (`response.body` / `response.headers`), JSONPath expression field with intellisense autocomplete from `responseSchema`. "Add Extraction" button.
- Delete step (with confirmation)

*Run button:* opens the Scenario Runner overlay

### 9.7 Scenario Runner

**Pre-run input form:**
- Shown before execution starts
- All scenario `inputs` listed with their defaults pre-filled
- User can override any value
- Unresolved `{{variable}}` tokens in defaults highlighted in red with tooltip showing the missing variable name
- "Run" starts execution; "Back" returns to editor without running

**Execution view — vertical stepper:**

Each step renders as a card:

```
✅  Step 1 — Get Token
    POST https://api.dev.internal/auth/token
    200 OK · 143ms
    Extracted: token = "eyJhbGci..."
    ▼ [expand for full request / response]

⏳  Step 2 — Stop Simulation
    Running...

⬜  Step 3 — Start Simulation
    Waiting
```

Step states:
- `waiting` — grey, not yet reached
- `running` — animated spinner
- `success` — green ✅, shows status code and duration
- `error` — red ❌, shows status code or network error message

Expanding a completed step shows:
- **Request sent:** method, full URL, headers, body (all post-resolution)
- **Response received:** status code, headers, body (pretty-printed)

**Variable state panel (sidebar):**
- Live key-value table of all variables currently in context
- Updates in real time as each step completes and extractions populate
- Secrets from `.env.local` shown masked as `••••••`

**Post-run actions:**
- "Run Again" — returns to pre-run input form with previous values pre-filled
- "Back to Editor" — returns to scenario editor

### 9.8 Settings
- Project-level: project display name, folder path (read-only)
- App-level: theme (dark / light / system), default request timeout in ms

---

## 10. Intellisense

### 10.1 Override key autocomplete
When the user types in the key field of a step's overrides panel:
- If the step's request has a `requestSchema` snapshot, Flupi suggests dot-notation paths derived from the schema (e.g. `body.warehouseId`, `body.duration`, `headers.X-Custom`)
- Each suggestion shows the expected type from the schema as a subtitle (e.g. `string`, `integer`)

### 10.2 Extraction JSONPath autocomplete
When the user types in the JSONPath field of a step's extractions panel:
- If the step's request has a `responseSchema` snapshot, Flupi suggests valid JSONPath expressions derived from the schema (e.g. `$.access_token`, `$.data.id`)
- After a step has been run at least once, actual response values are shown alongside schema suggestions as examples

### 10.3 Variable token autocomplete
In any field that accepts `{{...}}` tokens (URL, headers, body values, override values, input defaults):
- Typing `{{` opens a dropdown listing all variables currently available in context: `.env.local` keys (masked), active environment variables, scenario inputs defined above the current step, and variables extracted by preceding steps
- The dropdown shows the variable name and its current resolved value (masked for secrets)

---

## 11. What is NOT in v1

Explicitly deferred to v2 — do not implement in the initial build:

- Branching or conditional logic in scenarios (sequential only in v1)
- Polling / wait steps (recommend wrapping in a helper endpoint)
- Pass/fail test assertions on responses (runner is observational only)
- Response mocking
- Scenario parameterisation (running the same scenario with multiple input data sets)
- Plugin or extension system
- Cloud sync or multi-user real-time collaboration (Git is the collaboration mechanism)
- Export to Postman / Bruno / other formats

---

## 12. Suggested Build Order

Build in this sequence to validate the core data loop before adding UI polish:

1. **Project scaffolding** — Tauri + Svelte setup, open/create project folder, read/write JSON files, `.gitignore` generation
2. **Environment switching** — load environment files, switch active environment, edit variables, resolve `{{variables}}` in strings
3. **Request library** — load requests from disk, display in tree, edit and save a request, ad-hoc send with response panel
4. **Collection inheritance** — collection.json, auth inheritance, header merging, baseUrl prepending, Effective Request preview
5. **Scenario editor** — create scenario, add/reorder steps, configure overrides and extractions
6. **Scenario runner** — pre-run form, sequential execution, variable extraction and propagation, vertical stepper UI, variable state panel
7. **OpenAPI import** — fetch source, import wizard, generate request files, templateRef storage
8. **Drift detection** — hash comparison on sync, drift badges, side-by-side diff UI
9. **Intellisense** — schema-based autocomplete for override keys, extraction paths, and variable tokens
10. **Project picker** — launch screen, recent projects list, multi-project switching
11. **Polish** — settings screen, branding, error states, loading states, empty states
