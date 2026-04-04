# CK Search Path Guide

When using CK for semantic/hybrid search, scope the path correctly:

- Rust/backend code (commands, models, services) → `src-tauri/src/` or no path
- Frontend/Svelte/TypeScript (components, stores, routes) → `src/`
- When unsure which layer contains what you're looking for → omit path (searches whole project from root)

**Common mistake**: searching `src/` for Rust concepts like `injected_vars`, `execute_single_request`, `variable_resolver` — those are in `src-tauri/src/` and will return 0 results from `src/`.
