/**
 * StaleGuard — prevents stale async responses from overwriting fresh data.
 *
 * Usage pattern in store methods:
 * ```ts
 * async loadData(projectPath: string): Promise<void> {
 *   const token = this.guard.begin();  // STALE GUARD
 *   const data = await invoke('get_data', { projectPath });
 *   if (this.guard.isStale(token)) return;  // STALE GUARD
 *   this.data = data;
 * }
 * ```
 *
 * How it works:
 * - `begin()` increments a version counter and returns a token (the version at call time)
 * - `isStale(token)` returns true if a newer `begin()` has been called since
 * - This means only the most recent async operation will write its results
 */
export class StaleGuard {
  version = 0;

  /** Start a new guarded operation. Returns a token to check later. */
  begin(): number {
    return ++this.version;
  }

  /** Check if the token is stale (a newer operation has started). */
  isStale(token: number): boolean {
    return token !== this.version;
  }

  /** Reset the guard (e.g. on store reset). */
  reset(): void {
    this.version = 0;
  }
}
