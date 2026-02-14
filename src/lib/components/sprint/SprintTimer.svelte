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

  const DURATION_PRESETS = [15, 25, 30, 45, 60] as const;

  let selectedDuration = $state(sprintStore.durationMinutes);

  let displayMinutes = $derived(Math.floor(sprintStore.remainingSeconds / 60));
  let displaySeconds = $derived(sprintStore.remainingSeconds % 60);
  let timeDisplay = $derived(
    `${String(displayMinutes).padStart(2, '0')}:${String(displaySeconds).padStart(2, '0')}`
  );

  // SVG progress ring parameters
  const RING_RADIUS = 90;
  const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;
  let ringOffset = $derived(RING_CIRCUMFERENCE * (1 - sprintStore.elapsed));

  function selectPreset(minutes: number): void {
    selectedDuration = minutes;
    sprintStore.selectDuration(minutes);
  }

  async function handleStart(): Promise<void> {
    if (!chapterSlug || !projectPath) return;
    await sprintStore.start(
      selectedDuration,
      chapterSlug,
      projectPath,
      currentWordCount,
    );
  }

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

  // Watch for auto-complete (remaining hits 0 while active)
  $effect(() => {
    if (sprintStore.remainingSeconds === 0 && !sprintStore.isActive && onSprintEnd) {
      onSprintEnd();
    }
  });
</script>

<div class="sprint-timer" class:active={sprintStore.isActive}>
  {#if !sprintStore.isActive}
    <!-- Duration Selection -->
    <div class="duration-selector">
      <span class="selector-label">Sprint duration</span>
      <div class="preset-buttons">
        {#each DURATION_PRESETS as minutes}
          <button
            class="preset-btn"
            class:selected={selectedDuration === minutes}
            onclick={() => selectPreset(minutes)}
          >
            {minutes}m
          </button>
        {/each}
      </div>
    </div>
    <button
      class="start-btn"
      disabled={!chapterSlug}
      onclick={handleStart}
    >
      Start Sprint
    </button>
    {#if !chapterSlug}
      <span class="hint">Open a chapter to start a sprint</span>
    {/if}
  {:else}
    <!-- Active Sprint -->
    <div class="ring-container">
      <svg class="progress-ring" viewBox="0 0 200 200">
        <circle
          class="ring-bg"
          cx="100"
          cy="100"
          r={RING_RADIUS}
          fill="none"
          stroke-width="6"
        />
        <circle
          class="ring-progress"
          cx="100"
          cy="100"
          r={RING_RADIUS}
          fill="none"
          stroke-width="6"
          stroke-dasharray={RING_CIRCUMFERENCE}
          stroke-dashoffset={ringOffset}
          stroke-linecap="round"
          transform="rotate(-90 100 100)"
        />
      </svg>
      <div class="countdown">
        <span class="time" class:paused={sprintStore.isPaused}>{timeDisplay}</span>
        {#if sprintStore.isPaused}
          <span class="pause-label">Paused</span>
        {/if}
      </div>
    </div>
    <div class="controls">
      <button
        class="control-btn pause-btn"
        onclick={handlePauseResume}
      >
        {sprintStore.isPaused ? 'Resume' : 'Pause'}
      </button>
      <button
        class="control-btn stop-btn"
        onclick={handleStop}
      >
        Stop
      </button>
    </div>
  {/if}
</div>

<style>
  .sprint-timer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-xl);
  }

  /* Duration Selector */

  .duration-selector {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .selector-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-weight: var(--font-weight-medium);
  }

  .preset-buttons {
    display: flex;
    gap: var(--spacing-xs);
  }

  .preset-btn {
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-btn:hover {
    border-color: var(--accent-primary);
    color: var(--accent-primary);
  }

  .preset-btn.selected {
    background: var(--accent-primary);
    color: var(--text-inverse);
    border-color: var(--accent-primary);
  }

  .start-btn {
    padding: var(--spacing-sm) var(--spacing-xl);
    border: none;
    border-radius: var(--radius-lg);
    background: var(--accent-primary);
    color: var(--text-inverse);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .start-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .start-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  /* Progress Ring */

  .ring-container {
    position: relative;
    width: 200px;
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .progress-ring {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .ring-bg {
    stroke: var(--border-secondary);
  }

  .ring-progress {
    stroke: var(--accent-primary);
    transition: stroke-dashoffset 1s linear;
  }

  .countdown {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    z-index: 1;
  }

  .time {
    font-family: var(--font-mono);
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-bold);
    color: var(--text-primary);
    line-height: 1;
  }

  .time.paused {
    opacity: 0.5;
  }

  .pause-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Controls */

  .controls {
    display: flex;
    gap: var(--spacing-sm);
  }

  .control-btn {
    padding: var(--spacing-xs) var(--spacing-lg);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .control-btn:hover {
    background: var(--bg-tertiary);
  }

  .stop-btn {
    border-color: var(--color-error);
    color: var(--color-error);
  }

  .stop-btn:hover {
    background: var(--color-error);
    color: var(--text-inverse);
  }
</style>
