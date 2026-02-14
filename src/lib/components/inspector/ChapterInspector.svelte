<script lang="ts">
  import type { Chapter, ChapterStatus, EntitySummary } from '$lib/types';
  import { entityStore } from '$lib/stores';

  interface Props {
    chapter: Chapter;
    wordCount?: number;
    onStatusChange: (status: ChapterStatus) => void;
    onMetadataChange: (updates: Partial<Chapter>) => void;
  }

  let { chapter, wordCount = 0, onStatusChange, onMetadataChange }: Props = $props();

  // Derived: character entities for POV dropdown
  let characters = $derived<EntitySummary[]>(entityStore.entitiesByType['character'] ?? []);

  // Derived: progress percentage
  let progressPercent = $derived.by(() => {
    if (!chapter?.targetWords || chapter.targetWords <= 0) return 0;
    return Math.min(100, Math.round((wordCount / chapter.targetWords) * 100));
  });

  // Status color mapping
  const statusColors: Record<ChapterStatus, string> = {
    draft: 'var(--text-tertiary, #888)',
    revised: '#d4a017',
    final: '#22c55e',
  };

  function handleStatusChange(e: Event) {
    const status = (e.currentTarget as HTMLSelectElement).value as ChapterStatus;
    onStatusChange(status);
  }

  function handlePovChange(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    onMetadataChange({ pov: value || undefined });
  }

  function handleSynopsisInput(e: Event) {
    const value = (e.currentTarget as HTMLTextAreaElement).value;
    onMetadataChange({ synopsis: value || undefined });
  }

  function handleTargetWordsInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    if (raw === '') {
      onMetadataChange({ targetWords: undefined });
    } else {
      const parsed = parseInt(raw, 10);
      if (!isNaN(parsed) && parsed >= 0) {
        onMetadataChange({ targetWords: parsed });
      }
    }
  }
</script>

<div class="chapter-inspector">
  <!-- Status -->
  <div class="field-group">
    <label class="field-label" for="chapter-status">Status</label>
    <div class="status-field">
      <span
        class="status-dot"
        style:background-color={statusColors[chapter.status]}
      ></span>
      <select
        id="chapter-status"
        class="field-input field-select status-select"
        value={chapter.status}
        onchange={handleStatusChange}
      >
        <option value="draft">Draft</option>
        <option value="revised">Revised</option>
        <option value="final">Final</option>
      </select>
    </div>
  </div>

  <!-- POV Character -->
  <div class="field-group">
    <label class="field-label" for="chapter-pov">POV Character</label>
    <select
      id="chapter-pov"
      class="field-input field-select"
      value={chapter.pov ?? ''}
      onchange={handlePovChange}
    >
      <option value="">None</option>
      {#each characters as character (character.slug)}
        <option value={character.slug} selected={chapter.pov === character.slug}>
          {character.title}
        </option>
      {/each}
    </select>
  </div>

  <!-- Synopsis -->
  <div class="field-group">
    <label class="field-label" for="chapter-synopsis">Synopsis</label>
    <textarea
      id="chapter-synopsis"
      class="field-input field-textarea"
      placeholder="Brief chapter synopsis..."
      oninput={handleSynopsisInput}
    >{chapter.synopsis ?? ''}</textarea>
  </div>

  <!-- Word Count Target -->
  <div class="field-group">
    <label class="field-label" for="chapter-target-words">Target Words</label>
    <input
      id="chapter-target-words"
      class="field-input"
      type="number"
      value={chapter.targetWords ?? ''}
      placeholder="e.g. 3000"
      min="0"
      oninput={handleTargetWordsInput}
    />
    <div class="word-count-display">
      <span class="word-count-current">{wordCount.toLocaleString()}</span>
      {#if chapter.targetWords && chapter.targetWords > 0}
        <span class="word-count-separator">/</span>
        <span class="word-count-target">{chapter.targetWords.toLocaleString()}</span>
      {/if}
      <span class="word-count-unit">words</span>
    </div>
    {#if chapter.targetWords && chapter.targetWords > 0}
      <div class="progress-bar" role="progressbar" aria-valuenow={progressPercent} aria-valuemin={0} aria-valuemax={100}>
        <div
          class="progress-fill"
          class:progress-low={progressPercent < 50}
          class:progress-mid={progressPercent >= 50 && progressPercent < 90}
          class:progress-high={progressPercent >= 90}
          style="width: {progressPercent}%"
        ></div>
      </div>
      <span class="progress-label">{progressPercent}%</span>
    {/if}
  </div>
</div>

<style>
  .chapter-inspector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-sm);
  }

  /* -- Field Groups --------------------------------------------------------- */

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
  }

  .field-group:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .field-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .field-input {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .field-input:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px rgba(59, 111, 212, 0.15);
  }

  .field-input::placeholder {
    color: var(--text-tertiary);
  }

  .field-textarea {
    min-height: 5rem;
    resize: vertical;
    line-height: var(--line-height-normal);
    font-family: inherit;
  }

  .field-select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%235c554e' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--spacing-sm) center;
    padding-right: calc(var(--spacing-sm) + 1.25rem);
    cursor: pointer;
  }

  /* -- Status --------------------------------------------------------------- */

  .status-field {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .status-select {
    flex: 1;
  }

  /* -- Word Count ----------------------------------------------------------- */

  .word-count-display {
    display: flex;
    align-items: baseline;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
  }

  .word-count-current {
    font-weight: var(--font-weight-bold);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .word-count-separator {
    color: var(--text-tertiary);
  }

  .word-count-target {
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .word-count-unit {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  /* -- Progress Bar --------------------------------------------------------- */

  .progress-bar {
    width: 100%;
    height: 4px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width var(--transition-normal);
  }

  .progress-low {
    background: var(--accent-primary);
  }

  .progress-mid {
    background: #d4a017;
  }

  .progress-high {
    background: #22c55e;
  }

  .progress-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }
</style>
