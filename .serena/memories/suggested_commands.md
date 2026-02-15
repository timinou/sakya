# Suggested Commands

## Development
- `bun run tauri dev` - Start development server (Tauri + Vite)
- `bun run dev` - Start Vite dev server only (no Tauri)
- `bun run build` - Production build

## Testing
- `bun run test` - Run Vitest unit tests
- `bun run test:watch` - Run Vitest in watch mode
- `bun run test:e2e` - Run Playwright E2E tests
- `bun run test:all` - Run all tests (unit + E2E)

## Type Checking & Linting
- `bun run check` - SvelteKit sync + svelte-check (TypeScript checking)
- `cargo fmt` - Format Rust code (run before commits)
- `cargo clippy` - Rust linting
- `cargo test` - Rust unit tests

## Task Management
- `emacs --daemon=sakya -l @tasks/elisp/prd-tasks.el` - Start Emacs daemon
- `emacsclient -s sakya -e '(prd-validate-all-cli)'` - Validate all tasks (JSON)
- `emacsclient -s sakya -e '(prd-dashboard-cli)'` - Dashboard metrics

## System Utils
- `git`, `ls`, `grep`, `find` - standard Linux tools
