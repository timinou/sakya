# Task Completion Checklist

When completing a task:

1. Run `cargo fmt` if Rust files changed
2. Run `cargo clippy` if Rust files changed
3. Run `bun run check` for TypeScript/Svelte checking
4. Run relevant tests (`bun run test`, `cargo test`, `bun run test:e2e`)
5. Update task status in @tasks/*.org files
6. Run `emacsclient -s sakya -e '(prd-validate-all-cli)'` to validate task files
7. Commit with format: `<emoji> [<category>.<subcategory>] <title>`
8. Colocate @tasks updates with the implementation commit
