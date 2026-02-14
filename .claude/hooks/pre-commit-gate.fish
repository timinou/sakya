#!/usr/bin/env fish
# Hook 2: Pre-Commit Gate — Format + Co-location + PRD Validation
# Event: PreToolUse (matcher: Bash)
# Blocks commits with bad format, missing @tasks co-location, or invalid PRD files.

read -z INPUT

# Extract the command (read -lz preserves newlines from heredoc-style commands)
echo -n "$INPUT" | jq -r '.tool_input.command // empty' | read -lz cmd

# Fast path: not a git commit — exit immediately
if not string match -q '*git commit*' -- "$cmd"
    exit 0
end

# Skip --amend commits (user explicitly requested amend)
if string match -q '*--amend*' -- "$cmd"
    exit 0
end

# --- Helper: output deny decision ---
function deny_commit
    set -l reason $argv[1]
    echo "$reason" | jq -Rs '{hookSpecificOutput: {hookEventName: "PreToolUse", decision: {behavior: "deny", message: .}}}'
    exit 0
end

# ============================================================
# CHECK 1: Commit message format
# Expected: <emoji> [category.subcategory] title
# ============================================================

set -l msg ""

# Try heredoc format: <<'EOF' ... EOF or << 'EOF' ... EOF
set -l in_heredoc false
for line in (string split \n -- "$cmd")
    if $in_heredoc
        if string match -rq '^\s*EOF\s*$' -- "$line"
            break
        end
        if test -z "$msg"
            set msg (string trim -- "$line")
        end
    end
    if string match -rq "<<'?EOF'?" -- "$line"
        set in_heredoc true
    end
end

# Try -m flag if heredoc didn't work
if test -z "$msg"
    # Handle -m "msg" or -m 'msg'
    # Use string match on the full command string
    set -l extracted (string match -r -- '-m\s+["\x27]([^"\x27]+)' "$cmd")
    if test (count $extracted) -ge 2
        set msg (string trim -- "$extracted[2]")
    end
end

# If we can't extract message, let it through (avoid false blocks)
if test -z "$msg"
    exit 0
end

# Get first line only (the subject)
set msg (string split \n -- "$msg")[1]
set msg (string trim -- "$msg")

# Validate format: <non-space> [word.word] text
if not string match -rq '^\S+ \[\S+\.\S+\] .+' -- "$msg"
    deny_commit "Commit message format violation.

Expected: <emoji> [category.subcategory] title
Got: $msg

Examples:
  - \"(unicode-emoji) [fix.core] Fix tab close bug\"
  - \"(unicode-emoji) [tests.e2e] Add E2E test suite\"
  - \"(unicode-emoji) [chores.security] Add pre-commit hook\""
end

# ============================================================
# CHECK 2: Co-location — code files need @tasks/ co-location
# ============================================================

# Determine what files are being committed
set -l has_code false
set -l has_tasks false
set -l code_files

# Currently staged files
for f in (git diff --cached --name-only 2>/dev/null)
    if string match -rq '^(src/|src-tauri/|e2e/)' -- "$f"
        set has_code true
        set -a code_files "$f"
    end
    if string match -rq '^@tasks/' -- "$f"
        set has_tasks true
    end
end

# Also check files being added in the same command (before 'git commit')
set -l before_commit (string split 'git commit' -- "$cmd")[1]
if test -n "$before_commit"
    # Check for explicit paths
    if string match -rq '(src/|src-tauri/|e2e/)' -- "$before_commit"
        set has_code true
    end
    if string match -q '*@tasks/*' -- "$before_commit"
        set has_tasks true
    end

    # Handle 'git add .' or 'git add -A' — check working tree
    if string match -rq 'git add\s+(-A|\.(\s|&|$)|--all)' -- "$before_commit"
        for f in (git diff --name-only 2>/dev/null)
            if string match -rq '^(src/|src-tauri/|e2e/)' -- "$f"
                set has_code true
            end
            if string match -rq '^@tasks/' -- "$f"
                set has_tasks true
            end
        end
        # Also check untracked files
        for f in (git ls-files --others --exclude-standard 2>/dev/null)
            if string match -rq '^@tasks/' -- "$f"
                set has_tasks true
            end
        end
    end
end

# Config-only exemption: if ALL code files are config-ish, skip co-location check
set -l config_patterns 'Cargo.toml' 'Cargo.lock' 'package.json' 'bun.lockb' 'tsconfig' 'svelte.config' 'vite.config' 'tailwind.config' 'postcss.config' 'app.html' 'app.css'
if $has_code; and not $has_tasks
    set -l has_non_config false
    for f in $code_files
        set -l is_config false
        for pattern in $config_patterns
            if string match -q "*$pattern*" -- "$f"
                set is_config true
                break
            end
        end
        if not $is_config
            set has_non_config true
            break
        end
    end

    if $has_non_config
        deny_commit "Co-location violation: Code files are staged but no @tasks/ files.

Staged code files: "(string join ', ' -- $code_files)"

Either:
1. Stage @tasks/ updates alongside code changes
2. If this is a config-only change, no code files in src/src-tauri/e2e should be staged"
    end
end

# ============================================================
# CHECK 3: PRD validation (if @tasks files are being committed)
# ============================================================

if $has_tasks
    source "$CLAUDE_PROJECT_DIR/.claude/hooks/lib/sakya-emacs.fish"

    if sakya_emacs_ensure
        set -l validation (sakya_emacs_eval '(prd-validate-all-cli)')
        set -l valid (echo -n "$validation" | jq -r '.valid // empty' 2>/dev/null)

        if test "$valid" = "false"
            set -l errors (echo -n "$validation" | jq -r '.errors[]? | "  - [\(.file // "?"):\(.line // "?")] \(.message // "unknown error")"' 2>/dev/null)
            if test -z "$errors"
                set errors "  (could not parse error details)"
            end
            deny_commit "PRD validation failed. Fix these errors before committing:

$errors

Run validation manually:
  emacsclient -s sakya -e '(prd-validate-all-cli)' | jq ."
        end
    end
    # If daemon unavailable, allow commit (graceful degradation)
end

# All checks passed
exit 0
