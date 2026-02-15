# Sakya - Project Overview

## Purpose
Sakya is a writing application (think Scrivener-like) for long-form writing with entity management, manuscript management, and notes/corkboard features.

## Tech Stack
- **Backend**: Tauri 2.x (Rust)
- **Frontend**: SvelteKit 5 with Svelte 5 runes, TypeScript
- **Package Manager / JS Runtime**: Bun
- **Editor**: Lexical (rich text editor via svelte-lexical)
- **Icons**: lucide-svelte
- **Testing**: Vitest (unit), Playwright (E2E)
- **Platform**: Linux (dev machine)

## Code Style
- Svelte 5 runes: `$state()`, `$derived()`, `$effect()`, `$props()`
- Use `{@render}` blocks for children/snippets, NOT `<slot>`
- Scoped `<style>` blocks in Svelte components
- CSS custom properties from `src/app.css`
- TypeScript throughout frontend
- Rust fmt + clippy for backend

## Directory Structure
- `src/` - SvelteKit frontend
  - `src/lib/stores/` - Svelte 5 rune-based stores (singleton pattern)
  - `src/lib/types/` - TypeScript type definitions
  - `src/lib/components/` - Svelte components
  - `src/routes/` - SvelteKit routes
- `src-tauri/` - Rust/Tauri backend
- `e2e/` - Playwright E2E tests
- `@tasks/` - PRD task management system (org-mode)

## Commit Style
`<emoji> [<category>.<subcategory>] <title>`
