# Flupi — Project Guidelines

## Session Start

At the start of every session:
1. Run `ToolSearch` with query `"serena"` to load Serena MCP tool schemas.
2. Call `mcp__serena__initial_instructions` to load the Serena usage manual.
3. Call `mcp__serena__check_onboarding_performed` to confirm project memories are ready.

Do all three before any code search or file reading.

## General

Prioritize using serena and ck for gathering context, searching files and text across the repository. Always prefer `mcp__serena__find_symbol`, `mcp__serena__get_symbols_overview`, or `mcp__serena__search_for_pattern` over `Read` or `Grep` when exploring the codebase.

## Hybrid Code Search with ck

Use `ck` for finding code by meaning, not just keywords.

### Search Modes

- `ck --sem "concept"` - Semantic search (by meaning)
- `ck --lex "keyword"` - Lexical search (full-text)
- `ck --hybrid "query"` - Combined regex + semantic
- `ck --regex "pattern"` - Traditional regex search

### Best Practices

::: tip Recommended Usage Patterns
1. **Index once per session**: Run `ck --index .` at project start
2. **Use semantic for concepts**: "error handling", "database queries"
3. **Use lexical for names**: "getUserById", "AuthController"
4. **Tune threshold**: `--threshold 0.7` for high-confidence results
5. **Limit results**: `--limit 20` for focused output
:::

### Example Workflows

#### Find authentication logic
ck --sem "user authentication" src/

#### Find all TODO comments
ck --lex "TODO" .

#### Find error handling patterns with high confidence
ck --sem --threshold 0.8 "error handling" src/

## Design Context

### Users
Mixed audience: backend engineers integrating microservices, solo full-stack developers, and QA/test engineers. All are tool-savvy and comfortable with developer tooling. Design for the power user without alienating the newcomer — density is acceptable when it's organized.

### Brand Personality
Friendly, developer-focused, slightly playful — not corporate.
Three words: **fluid, precise, alive**.
The name comes from *flux* + *api* (verlan) — data flowing through an API pipeline. The personality echoes that: things move, things connect, nothing is static or heavy.

### Emotional Goal
**Fast & fluid.** Using Flupi should feel like flow state — snappy responses, no friction, the UI disappears and the work takes center stage. Every interaction should feel immediate.

### Aesthetic Direction
- **Dark mode by default**, light mode supported. Toggle in Settings.
- **Accent color**: electric cyan (`#06b6d4` range) or violet (`#7c3aed` range). Pick one and commit — don't mix both as primaries. Used for emphasis only: active states, badges, CTAs. Not decoration.
- **Typography**: monospace for all request/response data (URLs, headers, body, extracted values, variable tokens). Clean sans-serif for UI chrome (labels, nav, headings).
- **References**: Linear (dark, polished, fast, confident typography, subtle motion) and Bruno/Hoppscotch (familiar API tool patterns, not reinventing the wheel for users coming from those tools).
- **Anti-references**: Postman — avoid toolbar-heavy layouts, icon overload, and cluttered panels. No gradients. No glassmorphism. Flat and solid. Timeless over fashionable.

### Design Principles

1. **Invisible chrome.** UI elements should recede. Sidebars, tab bars, and toolbars use low-contrast backgrounds. The content (requests, responses, scenario steps) is always the brightest thing on screen.

2. **Density with structure.** Developer tools display a lot of information. Don't fight it — organize it. Use tight spacing within sections, generous spacing between sections. Every pixel of density should be intentional.

3. **Immediate feedback.** Every action gets instant visual response: save indicators appear and vanish quickly, step cards update in real time during runs, autocomplete opens without delay. Never leave the user wondering if something happened.

4. **Monospace is data, sans-serif is UI.** Apply this distinction consistently. Variable tokens (`{{...}}`), request paths, response bodies, extracted values — monospace. Labels, tooltips, nav items, headings — sans-serif. This creates a reliable visual grammar.

5. **No decorative complexity.** Color, motion, and visual weight are reserved for meaning: active state, error state, drift badge, running indicator. If an element doesn't communicate state or guide attention, simplify or remove it.

---

## Coding Conventions

### File size

No file should exceed ~200 lines. When a file grows beyond that, split it. Prefer many small, focused files over large monoliths — a file that does one thing is easier to test, read, and reuse.

### Rust test placement

**Never write inline tests.** Tests for each Rust module must go in a dedicated file under a `tests/` subdirectory alongside the source. The source file declares them with:

```rust
#[cfg(test)]
#[path = "tests/module_name.rs"]
mod tests;
```

The test file lives at e.g. `src-tauri/src/models/tests/environment.rs` and starts with `use super::*;`. There is no `mod.rs` in the `tests/` directories — each module references its own test file directly via `#[path]`. This keeps source files focused on implementation and makes tests easy to locate.
