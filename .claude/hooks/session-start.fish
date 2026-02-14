#!/usr/bin/env fish
# Hook 1: Session Start — Project State Loader
# Event: SessionStart
# Injects PRD dashboard, git status, DOING items, and compact recovery state.

read -z INPUT

set -l project_dir "$CLAUDE_PROJECT_DIR"
set -l lines

# --- Emacs daemon check ---
if not emacsclient -s sakya -e 't' >/dev/null 2>&1
    # Attempt auto-start
    if test -n "$project_dir" -a -f "$project_dir/@tasks/elisp/prd-tasks.el"
        emacs --daemon=sakya -l "$project_dir/@tasks/elisp/prd-tasks.el" >/dev/null 2>&1 &
        # Give it a moment
        sleep 2
    end
end

set -l emacs_ok false
if emacsclient -s sakya -e 't' >/dev/null 2>&1
    set emacs_ok true
end

# --- PRD Dashboard ---
if $emacs_ok
    set -l dashboard (emacsclient -s sakya -e '(prd-dashboard-cli)' 2>/dev/null)
    if test $status -eq 0; and test -n "$dashboard"
        set -a lines "## PRD Dashboard"
        set -a lines "$dashboard"
        set -a lines ""
    end

    set -l quick_status (emacsclient -s sakya -e '(prd-quick-status)' 2>/dev/null)
    if test $status -eq 0; and test -n "$quick_status"
        set -a lines "## Quick Status"
        set -a lines "$quick_status"
        set -a lines ""
    end

    set -l next_ids (emacsclient -s sakya -e '(prd-next-ids-cli)' 2>/dev/null)
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
