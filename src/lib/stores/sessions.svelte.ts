import { invoke } from '@tauri-apps/api/core';
import type { WritingSession, SessionStats } from '$lib/types';

class SessionsStore {
  sessions = $state<WritingSession[]>([]);
  stats = $state<SessionStats | null>(null);
  isLoading = $state(false);

  private loadedPath = $state<string | null>(null);

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
    this.isLoading = true;

    try {
      const [sessions, stats] = await Promise.all([
        invoke<WritingSession[]>('get_sessions', { projectPath }),
        invoke<SessionStats>('get_session_stats', { projectPath }),
      ]);
      this.sessions = sessions;
      this.stats = stats;
      this.loadedPath = projectPath;
    } catch (err) {
      console.error('[SessionsStore] Failed to load sessions:', err);
    } finally {
      this.isLoading = false;
    }
  }

  /** Re-fetch data (e.g. after a sprint ends). */
  async refresh(): Promise<void> {
    if (!this.loadedPath) return;
    await this.loadSessions(this.loadedPath);
  }
}

export const sessionsStore = new SessionsStore();
