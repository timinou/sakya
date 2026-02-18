import { invoke } from '@tauri-apps/api/core';
import type { WritingSession, SessionStats } from '$lib/types';
import { StaleGuard } from './stale-guard';

class SessionsStore {
  sessions = $state<WritingSession[]>([]);
  stats = $state<SessionStats | null>(null);
  isLoading = $state(false);

  private loadedPath = $state<string | null>(null);
  private guard = new StaleGuard();

  /** Derived Map<"YYYY-MM-DD", number> grouping sessions by date, summing words. */
  dailyWordCounts = $derived.by(() => {
    const map = new Map<string, number>();
    for (const session of this.sessions) {
      const dateKey = session.start.slice(0, 10); // "YYYY-MM-DD"
      map.set(dateKey, (map.get(dateKey) ?? 0) + session.wordsWritten);
    }
    return map;
  });

  /** Fetch sessions and stats from the backend. */
  async loadSessions(projectPath: string): Promise<void> {
    if (this.isLoading) return;
    const token = this.guard.begin(); // STALE GUARD
    this.isLoading = true;

    try {
      const [sessions, stats] = await Promise.all([
        invoke<WritingSession[]>('get_sessions', { projectPath }),
        invoke<SessionStats>('get_session_stats', { projectPath }),
      ]);
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.sessions = sessions;
      this.stats = stats;
      this.loadedPath = projectPath;
    } catch (err) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      console.error('[SessionsStore] Failed to load sessions:', err);
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoading = false;
      }
    }
  }

  /** Re-fetch data (e.g. after a sprint ends). */
  async refresh(): Promise<void> {
    if (!this.loadedPath) return;
    await this.loadSessions(this.loadedPath);
  }

  reset(): void {
    this.guard.reset();
    this.sessions = [];
    this.stats = null;
    this.isLoading = false;
    this.loadedPath = null;
  }
}

export const sessionsStore = new SessionsStore();
