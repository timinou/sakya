#!/usr/bin/env fish
# Sync @tasks/agents/*.org → .claude/agents/<name>.md
# Sync @tasks/process/*.org → .claude/skills/<name>/SKILL.md
# Run via direnv (.envrc) on directory entry, or manually.
#
# Agents: each org file should have #+MODEL: and #+TOOLS: headers.
# Falls back to model=sonnet, tools="Read, Grep, Glob, Bash, Edit, Write".
#
# Skills: each org file should have #+DESCRIPTION: header.
# Optional #+ALLOWED_TOOLS: restricts tool access.

set REPO_ROOT (cd (dirname (status filename))/.. && pwd)
set AGENTS_SRC "$REPO_ROOT/@tasks/agents"
set AGENTS_OUT "$REPO_ROOT/.claude/agents"
set SKILLS_SRC "$REPO_ROOT/@tasks/process"
set SKILLS_OUT "$REPO_ROOT/.claude/skills"

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
    grep -q '^auto-generated: true' "$md_file"; or continue
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
        and grep -q '^auto-generated: true' "$out_file"
        and test "$org_file" -ot "$out_file"
        continue
    end

    # Safety: if output exists but is NOT auto-generated, don't clobber
    if test -f "$out_file"
        and not grep -q '^auto-generated: true' "$out_file"
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

    # Write YAML frontmatter, then append pandoc body (preserving newlines)
    printf '%s\n' \
        "---" \
        "auto-generated: true" \
        "name: $name" \
        "description: \"$description\"" \
        "tools: $tools" \
        "model: $model" \
        "---" \
        "" \
        > "$out_file"
    pandoc -f org -t gfm --wrap=none "$org_file" >> "$out_file"

    echo "Synced agent: $name"
end

# ---------- Cleanup stale auto-generated skill SKILL.md files ----------
mkdir -p "$SKILLS_OUT"

for skill_dir in $SKILLS_OUT/*/
    test -d "$skill_dir"; or continue
    set skill_file "$skill_dir/SKILL.md"
    test -f "$skill_file"; or continue
    grep -q '^auto-generated: true' "$skill_file"; or continue
    set name (basename "$skill_dir")
    if not test -f "$SKILLS_SRC/$name.org"
        rm -rf "$skill_dir"
        echo "Removed stale skill: $name"
    end
end

# ---------- Sync each skill ----------
for org_file in $SKILLS_SRC/*.org
    test -f "$org_file"; or continue
    set name (basename "$org_file" .org)

    # Skip index.org
    test "$name" = index; and continue

    set out_dir "$SKILLS_OUT/$name"
    set out_file "$out_dir/SKILL.md"

    # Timestamp skip: if output exists, is auto-generated, and org is older → skip
    if test -f "$out_file"
        and grep -q '^auto-generated: true' "$out_file"
        and test "$org_file" -ot "$out_file"
        continue
    end

    # Safety: if output exists but is NOT auto-generated, don't clobber
    if test -f "$out_file"
        and not grep -q '^auto-generated: true' "$out_file"
        echo "Warning: $out_file exists without auto-generated marker — skipping (hand-crafted?)"
        continue
    end

    # Extract metadata from org headers
    set title (grep '^#+TITLE:' "$org_file" | head -1 | sed 's/^#+TITLE: *//')
    set description (grep '^#+DESCRIPTION:' "$org_file" | head -1 | sed 's/^#+DESCRIPTION: *//')
    set allowed_tools (grep '^#+ALLOWED_TOOLS:' "$org_file" | head -1 | sed 's/^#+ALLOWED_TOOLS: *//')

    # Description is required for skills
    if test -z "$description"
        echo "Warning: $org_file has no #+DESCRIPTION: — skipping"
        continue
    end

    # Build SKILL.md
    mkdir -p "$out_dir"

    # Write YAML frontmatter, then append pandoc body (preserving newlines)
    set -l lines \
        "---" \
        "auto-generated: true" \
        "name: $name" \
        "description: \"$description\""
    if test -n "$allowed_tools"
        set -a lines "allowed-tools: $allowed_tools"
    end
    set -a lines \
        "---" \
        ""

    printf '%s\n' $lines > "$out_file"
    pandoc -f org -t gfm --wrap=none "$org_file" >> "$out_file"

    echo "Synced skill: $name"
end
