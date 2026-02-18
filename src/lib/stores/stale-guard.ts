/**
 * StaleGuard — prevents stale async responses from overwriting fresh data.
 *
 * Two modes of operation:
 *
 * 1. `snapshot()` — for concurrent-safe guards (most store methods):
 *    Multiple methods can run concurrently without invalidating each other.
 *    Only `reset()` (project switch) invalidates snapshot tokens.
 *    ```ts
 *    async loadData(projectPath: string): Promise<void> {
 *      const token = this.guard.snapshot();  // STALE GUARD
 *      const data = await invoke('get_data', { projectPath });
 *      if (this.guard.isStale(token)) return;  // STALE GUARD
 *      this.data = data;
 *    }
 *    ```
 *
 * 2. `begin()` — for exclusive "last call wins" guards:
 *    Each `begin()` invalidates all previous tokens (both snapshot and begin).
 *    Use sparingly — only when concurrent calls to the SAME method must discard
 *    earlier results (e.g., two rapid calls to `open(projectA)` then `open(projectB)`).
 *    ```ts
 *    async open(path: string): Promise<void> {
 *      const token = this.guard.begin();  // STALE GUARD — invalidates prior open()
 *      const data = await invoke('open_project', { path });
 *      if (this.guard.isStale(token)) return;
 *      this.data = data;
 *    }
 *    ```
 *
 * `reset()` always invalidates all in-flight tokens (both modes).
 */
export class StaleGuard {
  version = 0;

  /** Start an exclusive guarded operation. Invalidates ALL previous tokens. */
  begin(): number {
    return ++this.version;
  }

  /** Capture the current version without invalidating previous tokens.
   *  Use for methods that may run concurrently within the same store. */
  snapshot(): number {
    return this.version;
  }

  /** Check if the token is stale (version has changed since token was captured). */
  isStale(token: number): boolean {
    return token !== this.version;
  }

  /** Invalidate all in-flight tokens (e.g. on store reset / project switch). */
  reset(): void {
    ++this.version;
  }
}
