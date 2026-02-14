import { invoke } from '@tauri-apps/api/core';

class SprintStore {
  isActive = $state(false);
  isPaused = $state(false);
  durationMinutes = $state(25);
  remainingSeconds = $state(0);
  sessionId = $state<string | null>(null);
  startWordCount = $state(0);
  sprintGoal = $state<number | undefined>(undefined);
  chapterSlug = $state<string | null>(null);

  /** Fraction of time elapsed, from 0 to 1. */
  elapsed = $derived(
    this.durationMinutes > 0
      ? 1 - this.remainingSeconds / (this.durationMinutes * 60)
      : 0
  );

  private intervalId: ReturnType<typeof setInterval> | null = null;

  selectDuration(minutes: number): void {
    if (this.isActive) return;
    this.durationMinutes = minutes;
  }

  async start(
    durationMinutes: number,
    chapterSlug: string,
    projectPath: string,
    currentWordCount: number,
    sprintGoal?: number
  ): Promise<void> {
    if (this.isActive) return;

    this.durationMinutes = durationMinutes;
    this.remainingSeconds = durationMinutes * 60;
    this.startWordCount = currentWordCount;
    this.sprintGoal = sprintGoal;
    this.chapterSlug = chapterSlug;
    this.isPaused = false;
    this.isActive = true;

    try {
      const id = await invoke<string>('start_session', {
        projectPath,
        chapterSlug,
        sprintGoal: sprintGoal ?? null,
      });
      this.sessionId = id;
    } catch (err) {
      console.error('Failed to start session on backend:', err);
      // Sprint continues locally even if backend call fails
    }

    this.startTimer();
  }

  pause(): void {
    if (!this.isActive || this.isPaused) return;
    this.isPaused = true;
    this.clearTimer();
  }

  resume(): void {
    if (!this.isActive || !this.isPaused) return;
    this.isPaused = false;
    this.startTimer();
  }

  async stop(currentWordCount: number, projectPath: string): Promise<void> {
    if (!this.isActive) return;

    this.clearTimer();

    const wordsWritten = Math.max(0, currentWordCount - this.startWordCount);

    if (this.sessionId) {
      try {
        await invoke('end_session', {
          projectPath,
          sessionId: this.sessionId,
          wordsWritten,
        });
      } catch (err) {
        console.error('Failed to end session on backend:', err);
      }
    }

    this.reset();
  }

  private startTimer(): void {
    this.clearTimer();
    this.intervalId = setInterval(() => {
      if (this.remainingSeconds <= 1) {
        this.remainingSeconds = 0;
        this.onTimerComplete();
      } else {
        this.remainingSeconds -= 1;
      }
    }, 1000);
  }

  private clearTimer(): void {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Called when the countdown reaches zero.
   * Auto-stops the sprint and calls end_session on the backend.
   */
  private async onTimerComplete(): Promise<void> {
    this.clearTimer();

    if (this.sessionId) {
      try {
        // Word count delta is unknown here — the component that
        // integrates this store should provide the current word count
        // via the `stop()` method. For auto-complete, we pass 0 delta
        // and rely on ITEM-089 to wire the actual word count.
        await invoke('end_session', {
          projectPath: '',
          sessionId: this.sessionId,
          wordsWritten: 0,
        });
      } catch (err) {
        console.error('Failed to end session on timer complete:', err);
      }
    }

    this.reset();
  }

  private reset(): void {
    this.isActive = false;
    this.isPaused = false;
    this.sessionId = null;
    this.startWordCount = 0;
    this.sprintGoal = undefined;
    this.chapterSlug = null;
    // Keep durationMinutes so the user's last selection is remembered
  }
}

export const sprintStore = new SprintStore();
