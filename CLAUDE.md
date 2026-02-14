# CLAUDE.md

## Project Overview
Sakya is a writing application built with Tauri 2.x (Rust backend) and SvelteKit 5 (Svelte 5 runes, TypeScript frontend). It uses Bun as the JS runtime/package manager.

## Directory Layout
- `src/` — SvelteKit frontend (routes, components, stores)
- `src-tauri/` — Rust backend (Tauri commands, core logic)
- `e2e/` — Playwright E2E tests
- `@tasks/` — PRD task management system (has its own CLAUDE.md)

## IMPORTANT instructions
- Commit as you go. Like this:
  <emoji> [<category>.<subcategory>] <title>
- Colocate any update to @tasks/* with the commit where the update happened. Try to be granular about it.

## Commands
- `bun run dev` -> development server
- `bun run build` -> production build
- `cargo test` -> unit tests
- `bun run test:e2e` -> end to end tests (write them thoroughly, every time)
- `emacsclient -e '(prd-validate-all-cli)'` (after you've loaded the elisp files, outputs JSON by default)
  IMPORTANT: Read the `@tasks/reference.org` for a detailed understanding of the way to use the PRD system

- Run `cargo fmt` before commits (Rust)
- Run `cargo clippy` for Rust linting
- E2E tests in `e2e/` directory

## Task Management
When working on tasks from the @tasks system, read `@tasks/CLAUDE.md` first.
For the full PRD specification, see `@tasks/reference.org`.
