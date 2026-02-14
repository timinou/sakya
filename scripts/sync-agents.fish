#!/usr/bin/env fish
# Sync @tasks/agents/*.org → .claude/agents/<name>.md
# Run via direnv (.envrc) on directory entry, or manually.
#
# Each org file should have #+MODEL: and #+TOOLS: headers.
# Falls back to model=sonnet, tools="Read, Grep, Glob, Bash, Edit, Write".

set REPO_ROOT (cd (dirname (status filename))/.. && pwd)
set AGENTS_SRC "$REPO_ROOT/@tasks/agents"
set AGENTS_OUT "$REPO_ROOT/.claude/agents"

set DEFAULT_MODEL sonnet
set DEFAULT_TOOLS "Read, Grep, Glob, Bash, Edit, Write"

# Guard: pandoc is required for org→markdown conversion
if not command -q pandoc
    exit 0
end

# ---------- Cleanup stale auto-generated agent .md files ----------
mkdir -p "$AGENTS_OUT"

for md_file in $AGENTS_OUT/*.md
    test -f "$md_file"; or continue
    grep -q '^<!-- auto-generated ' "$md_file"; or continue
    set name (basename "$md_file" .md)
    if not test -f "$AGENTS_SRC/$name.org"
        rm -f "$md_file"
        echo "Removed stale agent: $name"
    end
end

# ---------- Sync each agent ----------
for org_file in $AGENTS_SRC/*.org
    test -f "$org_file"; or continue
    set name (basename "$org_file" .org)

    # Skip index.org
    test "$name" = index; and continue

    set out_file "$AGENTS_OUT/$name.md"

    # Timestamp skip: if output exists, is auto-generated, and org is older → skip
    if test -f "$out_file"
        and grep -q '^<!-- auto-generated ' "$out_file"
        and test "$org_file" -ot "$out_file"
        continue
    end

    # Safety: if output exists but is NOT auto-generated, don't clobber
    if test -f "$out_file"
        and not grep -q '^<!-- auto-generated ' "$out_file"
        echo "Warning: $out_file exists without auto-generated marker — skipping (hand-crafted?)"
        continue
    end

    # Extract metadata from org headers
    set title (grep '^#+TITLE:' "$org_file" | head -1 | sed 's/^#+TITLE: *//')
    set model (grep '^#+MODEL:' "$org_file" | head -1 | sed 's/^#+MODEL: *//')
    set tools (grep '^#+TOOLS:' "$org_file" | head -1 | sed 's/^#+TOOLS: *//')

    # Apply defaults
    test -z "$model"; and set model $DEFAULT_MODEL
    test -z "$tools"; and set tools $DEFAULT_TOOLS

    # Extract first sentence of Apex Expertise section
    set apex_first_sentence (
        awk '/^\*\* Apex Expertise$/{found=1; next} found && /^\*/{exit} found && NF{printf "%s ", $0}' "$org_file" \
        | sed 's/  */ /g; s/^ *//; s/ *$//' \
        | sed 's/\. .*/\./'
    )
    set apex_first_sentence (string trim --right --chars='.' -- "$apex_first_sentence")
    set description "$title agent. $apex_first_sentence."

    # Convert org → GitHub-flavored markdown via pandoc
    set body (pandoc -f org -t gfm --wrap=none "$org_file")

    # Write agent .md with marker comment and YAML frontmatter
    printf '%s\n' \
        "<!-- auto-generated from @tasks/agents/$name.org — do not edit -->" \
        "---" \
        "name: $name" \
        "description: \"$description\"" \
        "tools: $tools" \
        "model: $model" \
        "---" \
        "" \
        "$body" \
        > "$out_file"

    echo "Synced agent: $name"
end
