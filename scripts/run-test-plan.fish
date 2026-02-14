#!/usr/bin/env fish
# Maps @tasks TEST_PLAN values to test commands.
# Usage: ./scripts/run-test-plan.fish <test-plan-value>
#
# Valid values: compile, test-rust, test-svelte, clippy, e2e, all

set -l plan $argv[1]

if test -z "$plan"
    echo "Usage: run-test-plan.fish <compile|test-rust|test-svelte|clippy|e2e|all>"
    exit 1
end

function run_compile
    echo "==> Checking Rust compilation..."
    cd src-tauri && cargo check && cd ..
    echo "==> Checking Svelte compilation..."
    bun run check
end

function run_test_rust
    echo "==> Running Rust tests..."
    cd src-tauri && cargo test && cd ..
end

function run_test_svelte
    echo "==> Running Svelte/Vitest tests..."
    bun run test
end

function run_clippy
    echo "==> Running Clippy..."
    cd src-tauri && cargo clippy -- -D warnings && cd ..
end

function run_e2e
    echo "==> Running Playwright E2E tests..."
    bun run test:e2e
end

function run_all
    run_compile
    and run_clippy
    and run_test_rust
    and run_test_svelte
    and run_e2e
end

switch $plan
    case compile
        run_compile
    case test-rust
        run_test_rust
    case test-svelte
        run_test_svelte
    case clippy
        run_clippy
    case e2e
        run_e2e
    case all
        run_all
    case '*'
        echo "Unknown test plan: $plan"
        echo "Valid values: compile, test-rust, test-svelte, clippy, e2e, all"
        exit 1
end
