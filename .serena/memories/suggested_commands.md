# Flupi — Suggested Commands

## Running the App
```bash
# Full dev mode (frontend + Rust backend, hot reload)
npm run tauri dev

# Frontend only (Vite dev server)
npm run dev

# Build production app
npm run build
npm run tauri build  # full Tauri production build
```

## TypeScript / Frontend
```bash
# Type-check frontend
npm run check

# Type-check in watch mode
npm run check:watch
```

## Rust Backend
```bash
# Check if Rust code compiles
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Build Rust only
cd src-tauri && cargo build
```

## Git / System (macOS/Darwin)
```bash
git status
git log --oneline -10
ls -la
find . -name "*.svelte" -not -path "*/node_modules/*"
grep -r "pattern" src/ --include="*.ts" --include="*.svelte"
```
