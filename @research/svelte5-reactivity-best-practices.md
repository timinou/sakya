# Svelte 5 Reactivity Best Practices for Sakya

> Audit conducted 2026-02-18. Based on codebase analysis + web research.

## Current Good Patterns (already implemented)

### 1. Path-Based Loaded Tracking
Used in: `manuscript.svelte.ts`, `notes.svelte.ts`, `entities.svelte.ts`, `sessions.svelte.ts`

```typescript
schemasLoadedPath = $state<string | null>(null);
schemasLoaded = $derived(this.schemasLoadedPath !== null);
```

Prevents infinite loops when data is legitimately empty (vs. `items.length > 0` guard).

### 2. Granular Loading Flags
Used in: `entities.svelte.ts`

```typescript
isLoadingSchemas = $state(false);
isLoadingEntities = $state<Record<string, boolean>>({});
isSaving = $state(false);
isLoading = $derived(this.isLoadingSchemas || this.isSaving || Object.values(this.isLoadingEntities).some(Boolean));
```

### 3. `untrack()` for Async in Effects
Used in: `BacklinksSection.svelte`, `WritingStats.svelte`, `BinderTree.svelte`

```typescript
$effect(() => {
  const t = title;
  const p = projectPath;
  untrack(() => fetchBacklinks(t, p));
});
```

### 4. `$derived` for Computed Values (not `$effect`)
All stores use `$derived` for computed properties, `$effect` only for side effects.

### 5. Event Listener Cleanup
All components return cleanup functions from `$effect` for `addEventListener`.

## Potential Issues Found

### Priority 1: Race Condition Guards for Project Switches
**Affected:** All stores with async load methods.
When a user rapidly switches projects, old IPC responses could overwrite new project data.

**Fix pattern:**
```typescript
private lastRequestPath: string | null = null;

async loadConfig(projectPath: string): Promise<void> {
  const requestPath = projectPath;
  this.lastRequestPath = requestPath;
  this.isLoading = true;
  try {
    const config = await invoke<Config>('get_config', { projectPath });
    if (requestPath === this.lastRequestPath) {
      this.config = config;
    }
  } finally {
    if (requestPath === this.lastRequestPath) {
      this.isLoading = false;
    }
  }
}
```

### Priority 2: Timer Cleanup in AppShell
`AppShell.svelte` has timer references (`peekBinderTimer`, `metadataSaveTimer`) that should use `$effect` cleanup.

### Priority 3: Documentation
- Add comments to every `untrack()` explaining why
- Document deep reactivity assumptions in store classes
- Add JSDoc to `$effect` blocks with non-obvious behavior

## Anti-Patterns to Avoid

| Anti-Pattern | Fix |
|---|---|
| `$effect` guarded by `data.length === 0` | Use `loadedPath !== null` pattern |
| `async` function AS the `$effect` callback | Call async function FROM the effect |
| Reading + writing same state in `$effect` | Use `untrack()` for the write |
| Shared `isLoading` across operations | Split into granular flags |
| `delete` on reactive proxy without comment | Add comment explaining proxy reactivity |
| No cleanup for timers/listeners | Return cleanup function from `$effect` |

## Sources

- [Svelte 5 States: Avoiding Common Reactivity Traps](https://jamesy.dev/blog/svelte-5-states-avoiding-common-reactivity-traps)
- [Avoid Async Effects In Svelte](https://joyofcode.xyz/avoid-async-effects-in-svelte)
- [Fine-Grained Reactivity in Svelte 5](https://frontendmasters.com/blog/fine-grained-reactivity-in-svelte-5/)
- [Svelte 5 $effect Documentation](https://svelte.dev/docs/svelte/$effect)
- [Svelte GitHub Discussion: Cleanup Patterns with Runes](https://github.com/sveltejs/svelte/discussions/11980)
