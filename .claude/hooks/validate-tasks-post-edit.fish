#!/usr/bin/env fish
# Hook 3: Validate Tasks Post-Edit — @tasks file validation feedback
# Event: PostToolUse (matcher: Write|Edit)
# Provides validation feedback after editing @tasks/*.org files.

read -z INPUT

# Extract the file path from tool_input
set -l file_path (echo -n "$INPUT" | jq -r '.tool_input.file_path // empty')

# Fast path: not an @tasks .org file
if test -z "$file_path"
    exit 0
end
if not string match -q '*@tasks/*' -- "$file_path"
    exit 0
end
if not string match -q '*.org' -- "$file_path"
    exit 0
end

source "$CLAUDE_PROJECT_DIR/.claude/hooks/lib/sakya-emacs.fish"

# Check emacs daemon + functions
if not sakya_emacs_ensure
    exit 0
end

# Run validation on the specific file
set -l validation (sakya_emacs_eval "(prd-validate-file-cli \"$file_path\")")
set -l valid (echo -n "$validation" | jq -r '.valid // empty' 2>/dev/null)

# If we can't parse, exit silently
if test -z "$valid"
    exit 0
end

set -l lines

if test "$valid" = "false"
    set -a lines "## @tasks Validation ERRORS"
    set -a lines ""
    set -a lines "File: $file_path"
    set -a lines ""

    set -l errors (echo -n "$validation" | jq -r '.errors[]? | "- **[\(.rule // "?")]** \(.message // "unknown")\n  Hint: \(.hint // "none")\n  Context: \(.context // "n/a")"' 2>/dev/null)
    if test -n "$errors"
        set -a lines "$errors"
    end

    set -a lines ""
    set -a lines "Fix these errors before committing. The pre-commit gate will block commits with invalid @tasks files."
else
    # Check for warnings
    set -l warning_count (echo -n "$validation" | jq -r '.warnings | length // 0' 2>/dev/null)
    if test "$warning_count" -gt 0 2>/dev/null
        set -a lines "## @tasks Validation Warnings"
        set -a lines ""
        set -a lines "File: $file_path"
        set -a lines ""

        set -l warnings (echo -n "$validation" | jq -r '.warnings[]? | "- **[\(.rule // "?")]** \(.message // "unknown")"' 2>/dev/null)
        if test -n "$warnings"
            set -a lines "$warnings"
        end
    else
        # Valid with no warnings — silent exit
        exit 0
    end
end

# Output feedback
if test (count $lines) -gt 0
    printf '%s\n' $lines | jq -Rs '{hookSpecificOutput: {hookEventName: "PostToolUse", additionalContext: .}}'
end
