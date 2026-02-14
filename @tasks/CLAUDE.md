# PRD Task Management - Claude Code Integration

This document instructs Claude Code on how to work with the `@tasks` PRD system.

## Quick Reference

- **Reference Guide**: [reference.org](reference.org)
- **Elisp Documentation**: [elisp/readme.org](elisp/readme.org)
- **Agent Definitions**: [agents/index.org](agents/index.org)
- **Current Dashboard**: [dashboard.org](dashboard.org)
- **Process Guides**: [process/index.org](process/index.org)
  - [Multi-File Changes](process/multi-file-changes.org) — `/multi-file-changes`
  - [Task Runner](process/task-runner.org) — `/task`
  - [Dashboard](process/dashboard.org) — `/dashboard`

## Validation Commands

Before making changes to @tasks files, always run validation:

```bash
# Validate all @tasks files (JSON output, the default for -cli variants)
emacsclient -s sakya -e '(prd-validate-all-cli)'

# Validate specific file
emacsclient -s sakya -e '(prd-validate-file-cli "/path/to/file.org")'

# Get current dashboard metrics
emacsclient -s sakya -e '(prd-dashboard-cli)'
```

## Claude Code Hooks

### PostToolUse Hook (Edit/Write)

After editing any file in `@tasks/`, run validation:

```bash
emacsclient -s sakya -e '(prd-validate-file-cli "{{file}}")'
```

Parse the JSON output:
- If `valid: false`, show errors to user and offer to fix
- If `warnings` exist, inform user but don't block
- Update links if `needs_link_sync: true`

### PreToolUse Hook (Git Commit)

Before committing changes that include @tasks files:

```bash
emacsclient -s sakya -e '(prd-validate-all-cli)'
```

Block commit if `valid: false`.

### Session Start

On session start, optionally show dashboard:

```bash
emacsclient -s sakya -e '(prd-dashboard-cli)'
```

## Output Format

### Validation JSON Structure

```json
{
  "valid": true|false,
  "errors": [
    {
      "file": "relative/path.org",
      "line": 42,
      "rule": "rule-name",
      "severity": "error",
      "message": "Human readable error",
      "hint": "How to fix this",
      "context": "Surrounding text for context"
    }
  ],
  "warnings": [...],
  "info": [...],
  "needs_link_sync": true|false,
  "metrics": {
    "total_items": 45,
    "complete": 12,
    "in_progress": 3,
    "blocked": 2,
    "pending": 28
  }
}
```

### Dashboard JSON Structure

```json
{
  "timestamp": "2026-02-03T10:30:00Z",
  "categories": [
    {
      "id": "PROJ-001",
      "title": "Feature Name",
      "total": 8,
      "complete": 6,
      "progress": 0.75
    }
  ],
  "agents": {
    "svelte-developer": {"assigned": 12, "complete": 8},
    "rust-architect": {"assigned": 5, "complete": 3},
    "testing-engineer": {"assigned": 8, "complete": 4}
  },
  "velocity": {
    "last_7_days": 4.2,
    "trend": "increasing"
  },
  "blockers": [
    {
      "item_id": "ITEM-015",
      "blocked_by": ["ITEM-012", "ITEM-014"]
    }
  ]
}
```

## Working with Tasks

### Creating New ITEMs

`ITEM` is a TODO keyword. Tasks are written as:

```org
** ITEM Short descriptive title
:PROPERTIES:
:CUSTOM_ID: ITEM-XXX
:AGENT: [[file:agents/AGENT-TYPE.org::#SECTION][agent-type:section]]
:EFFORT: Xh
:PRIORITY: #B
:DEPENDS: ITEM-YYY (if applicable)
:COMPONENT_REF: [[file:../../src-tauri/src/path/to/module.rs][Module Name]]
:FILES: src-tauri/src/path/to/file.rs, src/lib/components/Component.svelte
:TEST_PLAN: compile, test-rust, test-svelte, e2e
:END:

*** Description
Clear description of what to accomplish.

*** Acceptance Criteria
- [ ] Specific criterion 1
- [ ] Specific criterion 2
```

**Status keywords:** `ITEM` -> `DOING` -> `REVIEW` -> `DONE` (or `BLOCKED`)

**Task categories:** Tasks are organized into three folders:

```org
# Projects (new features, major work)
@tasks/projects/PROJ-001-feature-name.org

# Bugfixes
@tasks/bugfixes/BUG-001-description.org

# Improvements (refactors, optimizations)
@tasks/improvements/IMP-001-description.org
```

### Required Properties

Every ITEM **must** have:
- `CUSTOM_ID` - Unique identifier (format: ITEM-XXX)
- `AGENT` - Link to agent definition
- `EFFORT` - Time estimate (e.g., `1h`, `2h`, `30m`)
- `PRIORITY` - #A (high), #B (normal), #C (low)

### Recommended Properties

- `DEPENDS` - Dependencies (comma-separated ITEM IDs)
- `COMPONENT_REF` - Link to component/module (validated as info-level)
- `FILES` - Files that will be modified
- `TEST_PLAN` - Verification approach (validated as warning-level)

### Updating Task Status

Use the custom TODO keywords:

```
ITEM -> DOING -> REVIEW -> DONE
                  |
              BLOCKED
```

When marking DONE, the elisp system will:
1. Check all dependencies are met
2. Update downstream blockers
3. Sync bidirectional links
4. Update dashboard metrics

### Handling Validation Errors

When validation fails:

1. **Read the error message** - It includes the exact problem
2. **Check the hint** - Suggested fix is provided
3. **Look at context** - Surrounding text helps locate issue
4. **Fix and re-validate** - Run validation again

Example error handling:

```
ERROR in PROJ-001.org:42
  Missing required property: AGENT

  Fix: Add :AGENT: property referencing an agent definition

  Available agents: svelte-developer, rust-architect, testing-engineer, lexical-specialist, writing-app-designer
```

## Agent Assignment

### Choosing the Right Agent

| Task Type | Agent |
|-----------|-------|
| Svelte pages, components, stores | `svelte-developer` |
| Tauri commands, architecture, modules | `rust-architect` |
| Vitest, Playwright E2E | `testing-engineer` |
| Lexical editor integration, rich text | `lexical-specialist` |
| UX flows, writing workflows, app design | `writing-app-designer` |

### Agent Reference Format

```org
:AGENT: [[file:agents/svelte-developer.org::#core][svelte-developer:core]]
```

The format is: `file:agents/AGENT.org::#SECTION` where SECTION is the relevant expertise area within the agent definition.

## Bidirectional Links

### Adding Links to Documentation

When creating an ITEM that implements a component or feature:

1. Add forward link in ITEM properties
2. Run `emacsclient -s sakya -e '(prd-sync-backlinks)'` to update documentation

Or use the automated sync on save.

### Link Syntax

```org
# Forward link (in ITEM)
:COMPONENT_REF: [[file:../../src-tauri/src/module/mod.rs][Module Name]]

# Backward link (auto-added to doc)
:IMPLEMENTED_BY: [[file:@tasks/projects/PROJ-001.org::#ITEM-001][ITEM-001]]
```

## Metrics and Reporting

### Quick Status Check

```bash
emacsclient -s sakya -e '(prd-quick-status)'
```

Returns: `"45 tasks: 12 done, 3 in-progress, 2 blocked, 28 pending"`

### Velocity Calculation

```bash
emacsclient -s sakya -e '(prd-velocity-report 7)'
```

### Blocked Tasks

```bash
emacsclient -s sakya -e '(prd-list-blocked)'
```

## Error Recovery

### If validation is stuck

```bash
# Force reload all org files
emacsclient -s sakya -e '(prd-reload-all)'

# Clear validation cache
emacsclient -s sakya -e '(prd-clear-cache)'
```

### If links are broken

```bash
# Audit all links
emacsclient -s sakya -e '(prd-audit-links (quote json))'

# Repair broken links (interactive)
emacsclient -s sakya -e '(prd-repair-links)'
```

## Best Practices

1. **Validate early and often** - Run validation after every edit
2. **Keep tasks atomic** - 1-2 hours maximum
3. **Link everything** - Traceability helps debugging
4. **Update status promptly** - Keeps dashboard accurate
5. **Document blockers** - Add notes explaining why something is blocked
6. **Review dependencies** - Before starting work, check dependencies are truly DONE

## Troubleshooting

### emacsclient -s sakya not responding

Ensure the named Emacs daemon is running with the elisp loaded:
```bash
# From the project root
emacs --daemon=sakya -l @tasks/elisp/prd-tasks.el
```

### JSON parsing errors

Check for valid JSON:
```bash
emacsclient -s sakya -e '(prd-validate-all-cli)' | jq .
```

### Missing agent definitions

List available agents:
```bash
emacsclient -s sakya -e '(prd-list-agents)'
```
