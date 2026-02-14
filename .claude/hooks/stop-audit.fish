#!/usr/bin/env fish
# Hook 6: Stop Audit — End-of-Turn Gate
# Event: Stop
# Blocks stopping if there are uncommitted code changes or failing tests.

read -z INPUT

# Guard against infinite loops: if stop_hook_active is true, we're already
# continuing from a previous stop block — don't block again
set -l stop_active (echo -n "$INPUT" | jq -r '.stop_hook_active // false')
if test "$stop_active" = "true"
    exit 0
end

set -l project_dir (echo -n "$INPUT" | jq -r '.cwd // empty')
if test -z "$project_dir"
    set project_dir (pwd)
end

set -l block_reasons

# ============================================================
# CHECK 1: Uncommitted code changes in tracked files
# ============================================================

# Only check tracked files that have been modified (not untracked)
set -l dirty_code
for f in (git diff --name-only 2>/dev/null)
    if string match -rq '^(src/|src-tauri/|e2e/)' -- "$f"
        set -a dirty_code "$f"
    end
end

# Also check staged but uncommitted
for f in (git diff --cached --name-only 2>/dev/null)
    if string match -rq '^(src/|src-tauri/|e2e/)' -- "$f"
        set -a dirty_code "$f"
    end
end

if test (count $dirty_code) -gt 0
    set -a block_reasons "UNCOMMITTED CODE: You have "(count $dirty_code)" uncommitted code file(s): "(string join ', ' -- $dirty_code[1..3])
    if test (count $dirty_code) -gt 3
        set block_reasons[-1] "$block_reasons[-1] (and "(math (count $dirty_code) - 3)" more)"
    end
    set block_reasons[-1] "$block_reasons[-1]. Commit or stash before stopping."
end

# ============================================================
# CHECK 2: Failing tests
# ============================================================

set -l test_file "$project_dir/.claude/hooks/.last-test-status"
if test -f "$test_file"
    set -l test_status (cat "$test_file" | string trim)
    if test "$test_status" = "FAILED"
        set -a block_reasons "FAILING TESTS: Last test run FAILED. Fix failing tests before stopping. Run tests again to verify."
    end
end

# ============================================================
# DECISION
# ============================================================

if test (count $block_reasons) -gt 0
    string join \n\n -- $block_reasons | jq -Rs '{decision: "block", reason: .}'
else
    exit 0
end
