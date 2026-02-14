import { invoke } from '@tauri-apps/api/core';
import { toastManager } from '$lib/components/common/Toast.svelte';

class SprintStore {
  isActive = $state(false);
  isPaused = $state(false);
  durationMinutes = $state(25);
  remainingSeconds = $state(0);
  sessionId = $state<string | null>(null);
  startWordCount = $state(0);
  sprintGoal = $state<number | undefined>(undefined);
  chapterSlug = $state<string | null>(null);
  projectPath = $state<string>('');

  /** Fraction of time elapsed, from 0 to 1. */
  elapsed = $derived(
    this.durationMinutes > 0
      ? 1 - this.remainingSeconds / (this.durationMinutes * 60)
      : 0
  );

  private intervalId: ReturnType<typeof setInterval> | null = null;

  /**
   * Callback to get the current editor word count.
   * Set by the integrating component (e.g. AppShell) so that
   * auto-complete can calculate the correct word delta.
   */
  getWordCount: (() => number) | null = null;

  /**
   * Callback fired when the timer auto-completes (countdown reaches zero).
   * The integrating component should use this to trigger auto-save, etc.
   */
  onComplete: (() => void) | null = null;

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
    this.projectPath = projectPath;
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
      toastManager.show('Failed to start session on backend', 'error');
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
        toastManager.show('Failed to end session on backend', 'error');
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
   * Uses the stored projectPath and getWordCount callback to get
   * the correct values at the time of completion.
   */
  private async onTimerComplete(): Promise<void> {
    this.clearTimer();

    const currentWordCount = this.getWordCount?.() ?? 0;
    const wordsWritten = Math.max(0, currentWordCount - this.startWordCount);

    if (this.sessionId) {
      try {
        await invoke('end_session', {
          projectPath: this.projectPath,
          sessionId: this.sessionId,
          wordsWritten,
        });
      } catch (err) {
        console.error('Failed to end session on timer complete:', err);
        toastManager.show('Failed to end session on backend', 'error');
      }
    }

    this.reset();

    // Notify the integrating component (e.g. to trigger auto-save)
    this.onComplete?.();
  }

  private reset(): void {
    this.isActive = false;
    this.isPaused = false;
    this.sessionId = null;
    this.startWordCount = 0;
    this.sprintGoal = undefined;
    this.chapterSlug = null;
    this.projectPath = '';
    // Keep durationMinutes so the user's last selection is remembered
  }
}

export const sprintStore = new SprintStore();
