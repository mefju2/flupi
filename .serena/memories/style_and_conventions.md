# Flupi — Code Style & Conventions

## General
- **Max file size**: ~200 lines. Split files that grow beyond that. Many small focused files preferred.
- **No inline comments** unless they explain non-obvious logic.

## TypeScript / Svelte
- **Svelte 5 runes** throughout: `$props()`, `$state()`, `$derived()`, `$effect()` — NOT legacy `export let` or `$:` reactive statements.
- **State management**: Svelte `writable` stores in `src/lib/stores/` (one file per domain).
- **Tauri commands** called via `@tauri-apps/api` `invoke()`.
- **CSS tokens**: always use semantic app tokens (`bg-app-bg`, `bg-app-card`, `bg-app-panel`, `text-app-text`, `text-app-text-2`, `text-app-text-3`, `text-app-text-4`, `border-app-border`, `border-app-border-2`, `bg-app-hover`) — never hardcode `bg-white dark:bg-gray-900` etc.
- **Typography rule**: monospace (`font-mono`) for all data (URLs, headers, body, variable tokens `{{...}}`). Sans-serif for UI chrome (labels, nav, headings).
- **Component props**: use `interface Props` + `let { ... }: Props = $props()`.
- **Tailwind v4** (CSS-first config via `app.css` `@theme`).
- **OKLCH color tokens** defined in `src/app.css` with `--app-*` custom properties.

## Rust
- **Never write inline tests** (`#[cfg(test)] mod tests { ... }` inline). Always put tests in a separate file:
  ```rust
  // In source file:
  #[cfg(test)]
  #[path = "tests/module_name.rs"]
  mod tests;
  ```
  Test file lives at e.g. `src-tauri/src/models/tests/environment.rs` and starts with `use super::*;`.
- **No `mod.rs`** in `tests/` directories — each module references its own test file via `#[path]`.
- **Error handling**: use `thiserror` for error types. Tauri commands return `Result<T, Error>`.
- **Async**: use `tokio` async/await throughout backend.
- **Serialization**: `serde` + `serde_json`, `indexmap` for ordered maps where order matters.

## Design Principles
- Dark mode by default; light mode supported.
- Accent: electric cyan (`#06b6d4`) — used sparingly for active states, badges, CTAs.
- No gradients, no glassmorphism. Flat and solid.
- Immediate visual feedback for every action.
- Density is acceptable when organized. Tight within sections, generous between sections.
