#!/usr/bin/env fish
# Hook 4: Test Failure Tracker — records test pass/fail state
# Event: PostToolUse (matcher: Bash)
# Tracks test results to .last-test-status for the stop-audit hook.

read -z INPUT

# Extract command
set -l cmd (echo -n "$INPUT" | jq -r '.tool_input.command // empty')

# Fast path: not a test command
set -l is_test false
if string match -q '*cargo test*' -- "$cmd"
    set is_test true
else if string match -q '*bun run test*' -- "$cmd"
    set is_test true
else if string match -q '*bun test*' -- "$cmd"
    set is_test true
else if string match -q '*vitest*' -- "$cmd"
    set is_test true
else if string match -rq 'bun(x)?\s+playwright' -- "$cmd"
    set is_test true
else if string match -q '*test:e2e*' -- "$cmd"
    set is_test true
else if string match -q '*test:unit*' -- "$cmd"
    set is_test true
end

if not $is_test
    exit 0
end

# Get response as string for failure detection
set -l response (echo -n "$INPUT" | jq -r '.tool_response | if type == "string" then . elif type == "object" then (.stdout // "") + (.stderr // "") + (if .exitCode then "EXIT:" + (.exitCode | tostring) else "" end) else tostring end' 2>/dev/null)

# Detect failure patterns
set -l failed false

if string match -q '*FAIL*' -- "$response"
    set failed true
else if string match -q '*FAILED*' -- "$response"
    set failed true
else if string match -q '*error\[*' -- "$response"
    set failed true
else if string match -q '*panicked*' -- "$response"
    set failed true
else if string match -q '*test result: FAILED*' -- "$response"
    set failed true
else if string match -q '*EXIT:1*' -- "$response"
    set failed true
else if string match -q '*failures:*' -- "$response"
    set failed true
end

# Write status file
set -l project_dir (echo -n "$INPUT" | jq -r '.cwd // empty')
if test -z "$project_dir"
    set project_dir (pwd)
end
set -l status_file "$project_dir/.claude/hooks/.last-test-status"

# Ensure directory exists
mkdir -p (dirname "$status_file") 2>/dev/null

if $failed
    echo "FAILED" > "$status_file"

    # Output strong warning
    set -l lines
    set -a lines "## Test Failure Detected"
    set -a lines ""
    set -a lines "Command: `$cmd`"
    set -a lines ""
    set -a lines "Tests are FAILING. Per TDD workflow:"
    set -a lines "1. If this is expected (RED phase), write the implementation to make them pass"
    set -a lines "2. If unexpected, investigate and fix before continuing"
    set -a lines "3. Do NOT commit with failing tests"
    set -a lines ""
    set -a lines "The stop-audit hook will block you from stopping with failing tests."

    printf '%s\n' $lines | jq -Rs '{hookSpecificOutput: {hookEventName: "PostToolUse", additionalContext: .}}'
else
    echo "PASSED" > "$status_file"
    # Silent on success
    exit 0
end
