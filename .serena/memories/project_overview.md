# Flupi — Project Overview

## Purpose
Flupi is a desktop API testing tool (like Postman/Bruno/Hoppscotch) built as a Tauri v2 app. It lets developers define collections of HTTP requests, environments with variables, scenarios (ordered sequences of requests with variable extraction), and run/test API flows with drift detection.

The name comes from *flux + api* (verlan). Brand words: **fluid, precise, alive**.

## Target Users
Backend engineers, solo full-stack devs, QA/test engineers. Power-user density is acceptable.

## Tech Stack
- **Frontend**: SvelteKit 5 (Svelte 5 runes), TypeScript, Tailwind CSS v4, Monaco Editor
- **Backend**: Rust (Tauri v2 commands)
- **Desktop runtime**: Tauri v2
- **UI components**: lucide-svelte, svelte-dnd-action, tailwind-variants, clsx/tailwind-merge
- **Rust deps**: reqwest (HTTP), serde/serde_json, tokio, thiserror, regex, chrono, uuid, indexmap, serde_json_path

## Architecture Overview
```
src/                     — SvelteKit frontend
  lib/
    components/          — UI components (scenarios/, requests/, environments/, shared/, layout/, openapi/, functions/, settings/, project-picker/)
    stores/              — Svelte writable stores: project, requests, collections, environment, execution, scenarios, openapi, functions, ui
    services/            — TypeScript services
    utils/               — Utility functions
  routes/(app)/          — App pages: requests, scenarios, environments, openapi, functions, settings

src-tauri/src/           — Rust backend (Tauri commands)
  models/                — Data models: request, collection, environment, scenario, extraction, variable, app_data, openapi, script_function
  commands/              — Tauri command handlers: request, collection, environment, scenario, execution, execution_runner, openapi, functions, project, app_data, request_tree
  services/              — Business logic: file_io, project, variable_resolver, inheritance, http_client, referential_integrity, openapi_import, drift_detection, schema_defaults
  error.rs, utils.rs
```

## Data Model Key Concepts
- **Collections**: groups of requests
- **Requests**: HTTP requests with headers, body, params, overrides
- **Environments**: named variable sets (with secrets)
- **Scenarios**: ordered sequences of requests with variable extraction between steps
- **Variables**: `{{variableName}}` syntax in request fields
- **Drift detection**: detects when a request has drifted from its OpenAPI spec baseline
- **Extraction**: JSONPath/regex extraction of values from responses into variables
