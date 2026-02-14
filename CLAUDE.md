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
- Use the [[@tasks/]] system diligently. When in plan mode, write the `BUG-*`, `IMP-*`, and `PROJ-*` and `ITEM-*` in the relevant files, and detail the plan in relation to completing the tasks and subtasks in the org-mode files. Update them as you go.
- Colocate any update to @tasks/* with the commit where the update happened. Try to be granular about it.
- Include your orchestration plan, mentioning the agents within each phase and the testing logic. Include, in the orchestration, the parallelisation framework you'll use.
- Use TDD as much as possible, making sure tests are thorough (through a test matrix and deep reflectiona about the edge cases), and making sure they fail (RED) before they pass (GREEN). Apply that rigour across the testing (unit, behavioural, integration, end-to-end)
- Use tools such as MCPs in your research and manual testing phases to thoroughly understand a problem before you engage in planning. Root causes, be apex.
- Cite your research, and do more research if needed
- Use tasks to encapsulate the work, as much as possible.

## Commands
- `bun run tauri dev` -> development server
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

## Research
When researching online, create documents that are easily readable and comprehensive in `@research`. Link those files to other relevant files in the system.
