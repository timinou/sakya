---
auto-generated: true
name: multi-file-changes
description: "Methodology for complex changes spanning 3+ files, vendored code, or cross-crate boundaries. Consult before any multi-file change."
---

# Overview

Methodology for complex changes that span multiple files, crates, or submodule boundaries. These disciplines ensure clean, well-coordinated changes rather than iterative patch-and-fix cycles.

# Disciplines

## 1. Read Before Write

Read every file and symbol you will touch AND every file that references what you will change. For cross-boundary work this may mean a dozen or more targeted reads (struct definitions, trait implementations, module exports, command handlers, frontend type bindings) before writing a single line.

**Anti-pattern:** Read 2–3 files, form a mental model, start writing, patch when things break.

## 2. Trace the Type/Ownership Chain

For vendored/patched/deep code, trace the full path from where you are to the API you need. Document the chain explicitly so every line in a handler is intentional, not a guess.

## 3. Enumerate Second-Order Effects

After designing a change, ask: "what else touches the thing I'm changing?"

- Changing a struct field → what code reads/writes that field?
- Gating a code path → does the variable assignment need gating too? (unused warnings)
- Modifying a public API → which downstream consumers break?

## 4. Respect Conditional Compilation Boundaries

`#[cfg(...)]` blocks are load-bearing. Platform-specific code stays platform-specific. Do not unify into one abstraction when platforms genuinely differ.

## 5. Match Existing Patterns Exactly

Before writing a new test, commit, or task entry, read 2–3 existing examples of the same kind. Match structure, naming, style.

## 6. Verify at Each Layer

For multi-crate or submodule work: `cargo check` in submodule → `cargo check` in parent → `cargo fmt` → only then commit. Do not batch-verify at the end.

# When to Consult This

- Before any change touching 3+ files
- Before any change to vendored/patched code
- Before any change that crosses a crate/submodule boundary
- After a failed attempt at a complex change (re-read before retrying)

# See Also

- [Process Registry](index.org)
- [Rust Architect](../agents/rust-architect.org) - primary agent for cross-crate work
- [Reference Guide](../reference.org)
