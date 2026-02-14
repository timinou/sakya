# CLAUDE.md

## IMPORTANT instructions
- Commit as you go. Like this:
  <emoji> [<category>.<subcategory>] <title>
- Colocate any update to @tasks/* with the commit where the update happened. Try to be granular about it.

## Commands
- `bun run dev` -> development server
- `bun run build` -> production build
- `cargo test` -> unit tests
- `bun run test:e2e` -> end to end tests (write them thoroughly, every time)
- `emacsclient -e '(prd-validate-all-cli :format json)'` (after you've loaded the elisp files)
  IMPORTANT: Read the `@tasks/reference.org` for a detailed understanding of the way to use the PRD system

- Run `cargo fmt` before commits (Rust)
- Run `cargo clippy` for Rust linting
- E2E tests in `e2e/` directory
