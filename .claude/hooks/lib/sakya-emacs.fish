# Shared library for Sakya Claude Code hooks.
# Provides reusable functions for Emacs daemon interaction.
#
# Usage (in hook scripts):
#   source "$CLAUDE_PROJECT_DIR/.claude/hooks/lib/sakya-emacs.fish"

# --- sakya_prd_elisp_path ---
# Locate @tasks/elisp/prd-tasks.el, trying multiple paths.
# Prints the absolute path on success, returns 1 on failure.
function sakya_prd_elisp_path
    for dir in "$CLAUDE_PROJECT_DIR" (pwd) /home/user/code/personal/sakya
        if test -n "$dir" -a -f "$dir/@tasks/elisp/prd-tasks.el"
            echo "$dir/@tasks/elisp/prd-tasks.el"
            return 0
        end
    end
    return 1
end

# --- sakya_emacs_ensure ---
# Ensure the 'sakya' Emacs daemon is running AND PRD functions are loaded.
# - If daemon is not running: starts it with -l prd-tasks.el, polls for readiness.
# - If daemon is running but functions missing: loads prd-tasks.el without restart.
# Returns 0 on success, 1 on failure.
function sakya_emacs_ensure
    if not emacsclient -s sakya -e 't' >/dev/null 2>&1
        # Daemon not running — start it
        set -l elisp_path (sakya_prd_elisp_path)
        or return 1

        emacs --daemon=sakya -l "$elisp_path" >/dev/null 2>&1 &

        # Poll for readiness: 500ms intervals, max 5s (10 attempts)
        set -l attempts 10
        while test $attempts -gt 0
            sleep 0.5
            if emacsclient -s sakya -e 't' >/dev/null 2>&1
                return 0
            end
            set attempts (math $attempts - 1)
        end
        return 1
    end

    # Daemon is running — check if PRD functions are loaded
    set -l has_fn (emacsclient -s sakya -e "(fboundp 'prd-next-ids-cli)" 2>/dev/null)
    if test "$has_fn" != t
        # Functions missing — load without restarting
        set -l elisp_path (sakya_prd_elisp_path)
        or return 1

        emacsclient -s sakya -e "(load \"$elisp_path\")" >/dev/null 2>&1

        # Verify
        set has_fn (emacsclient -s sakya -e "(fboundp 'prd-next-ids-cli)" 2>/dev/null)
        if test "$has_fn" != t
            return 1
        end
    end

    return 0
end

# --- sakya_emacs_eval ---
# Evaluate an elisp expression via emacsclient and return clean output.
# Strips the elisp string wrapping that emacsclient -e adds ("..." with \" and \\).
# Output is raw text/JSON suitable for piping to jq or direct use.
#
# Usage: sakya_emacs_eval '(prd-dashboard-cli)'
#        sakya_emacs_eval '(prd-validate-file-cli "/path/to/file.org")'
function sakya_emacs_eval
    set -l expr $argv[1]
    set -l raw (emacsclient -s sakya -e "$expr" 2>/dev/null)
    or return 1

    # Detect elisp string wrapping: output starts and ends with "
    if string match -qr '^".*"$' -- "$raw"
        # Strip outer quotes
        set raw (string replace -r '^"(.*)"$' '$1' -- "$raw")
        # Unescape elisp string escapes (order: \\ first, then \")
        set raw (string replace -a '\\\\' \\ -- "$raw")
        set raw (string replace -a '\"' '"' -- "$raw")
    end

    printf '%s' "$raw"
end
