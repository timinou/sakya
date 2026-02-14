<script lang="ts">
  import { sprintStore } from '$lib/stores';

  interface Props {
    chapterSlug?: string | null;
    projectPath?: string;
    currentWordCount?: number;
    onSprintEnd?: () => void;
  }

  let {
    chapterSlug = null,
    projectPath = '',
    currentWordCount = 0,
    onSprintEnd,
  }: Props = $props();

  let displayMinutes = $derived(Math.floor(sprintStore.remainingSeconds / 60));
  let displaySeconds = $derived(sprintStore.remainingSeconds % 60);
  let timeDisplay = $derived(
    `${String(displayMinutes).padStart(2, '0')}:${String(displaySeconds).padStart(2, '0')}`
  );

  // Word count goal progress (0 to 1)
  let wordsWritten = $derived(Math.max(0, currentWordCount - sprintStore.startWordCount));
  let goalProgress = $derived(
    sprintStore.sprintGoal && sprintStore.sprintGoal > 0
      ? Math.min(1, wordsWritten / sprintStore.sprintGoal)
      : 0
  );
  let goalMet = $derived(
    sprintStore.sprintGoal ? wordsWritten >= sprintStore.sprintGoal : false
  );

  function handlePauseResume(): void {
    if (sprintStore.isPaused) {
      sprintStore.resume();
    } else {
      sprintStore.pause();
    }
  }

  async function handleStop(): Promise<void> {
    await sprintStore.stop(currentWordCount, projectPath);
    onSprintEnd?.();
  }

  async function handleSave(): Promise<void> {
    window.dispatchEvent(new CustomEvent('sakya:save'));
  }
</script>

{#if sprintStore.isActive}
  <div class="sprint-overlay" class:paused={sprintStore.isPaused}>
    <!-- Vignette border for focus feel -->
    <div class="vignette vignette-top"></div>
    <div class="vignette vignette-bottom"></div>
    <div class="vignette vignette-left"></div>
    <div class="vignette vignette-right"></div>

    <!-- Floating sprint bar at the top -->
    <div class="sprint-bar">
      <div class="sprint-bar-left">
        <span class="sprint-label">Sprint</span>
      </div>

      <div class="sprint-bar-center">
        <span class="countdown" class:paused={sprintStore.isPaused}>
          {timeDisplay}
        </span>
        {#if sprintStore.isPaused}
          <span class="pause-indicator">Paused</span>
        {/if}
      </div>

      <div class="sprint-bar-right">
        <button
          class="bar-btn"
          onclick={handlePauseResume}
          title={sprintStore.isPaused ? 'Resume sprint' : 'Pause sprint'}
          aria-label={sprintStore.isPaused ? 'Resume sprint' : 'Pause sprint'}
        >
          {sprintStore.isPaused ? 'Resume' : 'Pause'}
        </button>
        <button
          class="bar-btn bar-btn-stop"
          onclick={handleStop}
          title="Stop sprint"
          aria-label="Stop sprint"
        >
          Stop
        </button>
        <button
          class="bar-btn bar-btn-save"
          onclick={handleSave}
          title="Save (Ctrl+S)"
          aria-label="Save document"
        >
          Save
        </button>
      </div>
    </div>

    <!-- Elapsed progress bar (thin line under sprint bar) -->
    <div class="elapsed-track">
      <div
        class="elapsed-fill"
        style:width="{sprintStore.elapsed * 100}%"
      ></div>
    </div>

    <!-- Word count goal progress bar (only if sprint goal is set) -->
    {#if sprintStore.sprintGoal}
      <div class="goal-bar">
        <div class="goal-track">
          <div
            class="goal-fill"
            class:goal-met={goalMet}
            style:width="{goalProgress * 100}%"
          ></div>
        </div>
        <span class="goal-text">
          {wordsWritten} / {sprintStore.sprintGoal} words
          {#if goalMet}
            — Goal reached!
          {/if}
        </span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .sprint-overlay {
    position: fixed;
    inset: 0;
    z-index: 500;
    pointer-events: none;
    animation: sprint-fade-in 300ms ease forwards;
  }

  @keyframes sprint-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  /* === Vignette borders for focus feel === */
  .vignette {
    position: absolute;
    pointer-events: none;
  }

  .vignette-top {
    top: 0;
    left: 0;
    right: 0;
    height: 60px;
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0.08), transparent);
  }

  .vignette-bottom {
    bottom: 0;
    left: 0;
    right: 0;
    height: 60px;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.08), transparent);
  }

  .vignette-left {
    top: 0;
    bottom: 0;
    left: 0;
    width: 40px;
    background: linear-gradient(to right, rgba(0, 0, 0, 0.06), transparent);
  }

  .vignette-right {
    top: 0;
    bottom: 0;
    right: 0;
    width: 40px;
    background: linear-gradient(to left, rgba(0, 0, 0, 0.06), transparent);
  }

  /* === Floating sprint bar === */
  .sprint-bar {
    position: absolute;
    top: var(--spacing-sm);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    pointer-events: auto;
    z-index: 501;
    min-width: 360px;
    justify-content: space-between;
    user-select: none;
  }

  .sprint-bar-left {
    display: flex;
    align-items: center;
  }

  .sprint-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sprint-bar-center {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .countdown {
    font-family: var(--font-mono);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    line-height: 1;
  }

  .countdown.paused {
    opacity: 0.5;
  }

  .pause-indicator {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .sprint-bar-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .bar-btn {
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .bar-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .bar-btn-stop {
    border-color: var(--color-error);
    color: var(--color-error);
  }

  .bar-btn-stop:hover {
    background: var(--color-error);
    color: var(--text-inverse);
  }

  .bar-btn-save {
    border-color: var(--accent-primary);
    color: var(--accent-primary);
  }

  .bar-btn-save:hover {
    background: var(--accent-primary);
    color: var(--text-inverse);
  }

  /* === Elapsed progress bar === */
  .elapsed-track {
    position: absolute;
    top: calc(var(--spacing-sm) + 40px);
    left: 50%;
    transform: translateX(-50%);
    width: 360px;
    height: 2px;
    background: var(--border-secondary);
    border-radius: 1px;
    overflow: hidden;
    z-index: 501;
  }

  .elapsed-fill {
    height: 100%;
    background: var(--accent-primary);
    border-radius: 1px;
    transition: width 1s linear;
  }

  /* === Word count goal bar === */
  .goal-bar {
    position: absolute;
    top: calc(var(--spacing-sm) + 48px);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    pointer-events: auto;
    z-index: 501;
  }

  .goal-track {
    width: 160px;
    height: 4px;
    background: var(--border-secondary);
    border-radius: 2px;
    overflow: hidden;
  }

  .goal-fill {
    height: 100%;
    background: var(--accent-primary);
    border-radius: 2px;
    transition: width 500ms ease;
  }

  .goal-fill.goal-met {
    background: var(--color-success);
  }

  .goal-text {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
  }
</style>
