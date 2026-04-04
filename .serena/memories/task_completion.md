# Flupi — Task Completion Checklist

## After completing any coding task:

### Frontend (TypeScript/Svelte)
- [ ] Run `npm run check` — must pass with 0 errors
- [ ] Verify Svelte 5 runes syntax used (not legacy reactive syntax)
- [ ] Confirm CSS tokens used (no hardcoded color bypasses like `bg-white dark:bg-gray-800`)
- [ ] File stays under ~200 lines; split if needed

### Rust Backend
- [ ] Run `cd src-tauri && cargo check` — must compile cleanly
- [ ] Run `cd src-tauri && cargo test` — all tests must pass
- [ ] New tests go in `src-tauri/src/<module>/tests/<module_name>.rs`, NOT inline
- [ ] Source file references test file via `#[cfg(test)] #[path = "tests/module_name.rs"] mod tests;`

### Both
- [ ] No file exceeds ~200 lines
- [ ] No inline test modules in Rust
- [ ] Semantic token usage in CSS (not raw Tailwind color classes for surfaces)
