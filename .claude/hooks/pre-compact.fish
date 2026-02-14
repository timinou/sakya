#!/usr/bin/env fish
# Hook 5: Pre-Compact — Context Preservation
# Event: PreCompact
# Saves working state to JSON before context compaction.

read -z INPUT

set -l project_dir (echo -n "$INPUT" | jq -r '.cwd // empty')
if test -z "$project_dir"
    set project_dir (pwd)
end

set -l state_file "$project_dir/.claude/hooks/.working-state.json"
mkdir -p (dirname "$state_file") 2>/dev/null

# --- Gather state ---

source "$CLAUDE_PROJECT_DIR/.claude/hooks/lib/sakya-emacs.fish"

# DOING items from PRD
set -l doing_items '""'
if sakya_emacs_ensure
    set -l quick_status (sakya_emacs_eval '(prd-quick-status)')
    if test $status -eq 0; and test -n "$quick_status"
        # sakya_emacs_eval returns clean text — wrap as JSON string for jq --argjson
        set doing_items (printf '%s' "$quick_status" | jq -Rs '.')
    end
end

# Uncommitted changes
set -l uncommitted (git status --porcelain 2>/dev/null | jq -Rs '.')
if test -z "$uncommitted"
    set uncommitted '""'
end

# Recent commits
set -l recent_commits (git log --oneline -5 2>/dev/null | jq -Rs '.')
if test -z "$recent_commits"
    set recent_commits '""'
end

# Current branch
set -l branch (git branch --show-current 2>/dev/null)
if test -z "$branch"
    set branch "unknown"
end

# Last test status
set -l test_status "unknown"
set -l test_file "$project_dir/.claude/hooks/.last-test-status"
if test -f "$test_file"
    set test_status (cat "$test_file" | string trim)
end

# Timestamp
set -l timestamp (date -Iseconds 2>/dev/null; or date '+%Y-%m-%dT%H:%M:%S')

# --- Write state file ---
jq -n \
    --arg timestamp "$timestamp" \
    --arg branch "$branch" \
    --argjson doing_items "$doing_items" \
    --argjson uncommitted "$uncommitted" \
    --argjson recent_commits "$recent_commits" \
    --arg test_status "$test_status" \
    '{
        timestamp: $timestamp,
        branch: $branch,
        doing_items: $doing_items,
        uncommitted_changes: $uncommitted,
        recent_commits: $recent_commits,
        last_test_status: $test_status
    }' > "$state_file"

# Output context for the compact summary
set -l lines
set -a lines "## Pre-Compaction State Saved"
set -a lines ""
set -a lines "Working state has been saved to .claude/hooks/.working-state.json"
set -a lines "This will be automatically restored after compaction."
set -a lines ""
set -a lines "Branch: $branch | Tests: $test_status"

printf '%s\n' $lines | jq -Rs '{hookSpecificOutput: {hookEventName: "PreCompact", additionalContext: .}}'
