#!/usr/bin/env fish
# Hook 1: Session Start — Project State Loader
# Event: SessionStart
# Injects PRD dashboard, git status, DOING items, and compact recovery state.

read -z INPUT

set -l project_dir "$CLAUDE_PROJECT_DIR"
set -l lines

source "$project_dir/.claude/hooks/lib/sakya-emacs.fish"

# --- PRD Dashboard ---
if sakya_emacs_ensure
    set -l dashboard (sakya_emacs_eval '(prd-dashboard-cli)')
    if test $status -eq 0; and test -n "$dashboard"
        set -a lines "## PRD Dashboard"
        set -a lines "$dashboard"
        set -a lines ""
    end

    set -l quick_status (sakya_emacs_eval '(prd-quick-status)')
    if test $status -eq 0; and test -n "$quick_status"
        set -a lines "## Quick Status"
        set -a lines "$quick_status"
        set -a lines ""
    end

    set -l next_ids (sakya_emacs_eval '(prd-next-ids-cli)')
    if test $status -eq 0; and test -n "$next_ids"
        set -a lines "## Next Available IDs"
        set -a lines "$next_ids"
        set -a lines ""
    end
else
    set -a lines "## PRD System"
    set -a lines "Emacs daemon 'sakya' not available. Start with:"
    set -a lines '```'
    set -a lines "emacs --daemon=sakya -l @tasks/elisp/prd-tasks.el"
    set -a lines '```'
    set -a lines ""
end

# --- Git status ---
set -l git_status (git status --porcelain 2>/dev/null)
if test -n "$git_status"
    set -a lines "## Uncommitted Changes"
    set -a lines '```'
    set -a lines "$git_status"
    set -a lines '```'
    set -a lines ""
end

# --- Compact recovery ---
set -l state_file "$project_dir/.claude/hooks/.working-state.json"
if test -f "$state_file"
    set -l working_state (cat "$state_file" 2>/dev/null)
    if test -n "$working_state"
        set -a lines "## Recovered Working State (from compaction)"
        set -a lines '```json'
        set -a lines "$working_state"
        set -a lines '```'
        set -a lines ""
        set -a lines "Review this state and continue where you left off."
    end
end

# --- Output ---
if test (count $lines) -gt 0
    printf '%s\n' $lines | jq -Rs '{hookSpecificOutput: {hookEventName: "SessionStart", additionalContext: .}}'
else
    echo '{}'
end
