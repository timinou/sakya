<script lang="ts">
  import type { Chapter, ChapterStatus, EntitySummary } from '$lib/types';
  import { editorState, entityStore } from '$lib/stores';

  interface Props {
    chapter: Chapter;
    onSave?: (chapter: Chapter) => void;
  }

  let { chapter, onSave }: Props = $props();

  // Deep clone chapter prop into local state for editing
  let chapterSnapshot = $derived(JSON.stringify(chapter));

  let localChapter = $state<Chapter>(undefined as unknown as Chapter);
  let isDirty = $state(false);
  let saveTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let lastSyncedSnapshot = $state('');

  // Derived: character entities for POV dropdown
  let characters = $derived<EntitySummary[]>(entityStore.entitiesByType['character'] ?? []);

  // Derived: word count from editor
  let currentWords = $derived(editorState.wordCount.words);

  // Derived: progress percentage
  let progressPercent = $derived.by(() => {
    if (!localChapter?.targetWords || localChapter.targetWords <= 0) return 0;
    return Math.min(100, Math.round((currentWords / localChapter.targetWords) * 100));
  });

  // Sync external chapter changes into local state (including initial mount)
  $effect(() => {
    if (chapterSnapshot !== lastSyncedSnapshot) {
      localChapter = JSON.parse(chapterSnapshot);
      lastSyncedSnapshot = chapterSnapshot;
      isDirty = false;
    }
  });

  // Debounced auto-save
  $effect(() => {
    if (!isDirty || !onSave || !localChapter) return;

    const snapshot = JSON.stringify(localChapter);
    if (snapshot === lastSyncedSnapshot) {
      isDirty = false;
      return;
    }

    if (saveTimer) clearTimeout(saveTimer);

    const chapterToSave = JSON.parse(snapshot) as Chapter;
    saveTimer = setTimeout(() => {
      onSave(chapterToSave);
      lastSyncedSnapshot = snapshot;
      isDirty = false;
      saveTimer = null;
    }, 1000);

    return () => {
      if (saveTimer) clearTimeout(saveTimer);
    };
  });

  function markDirty() {
    isDirty = true;
  }

  function handleTitleInput(e: Event) {
    localChapter.title = (e.currentTarget as HTMLInputElement).value;
    markDirty();
  }

  function handlePovChange(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    localChapter.pov = value || undefined;
    markDirty();
  }

  function handleStatusChange(e: Event) {
    localChapter.status = (e.currentTarget as HTMLSelectElement).value as ChapterStatus;
    markDirty();
  }

  function handleSynopsisInput(e: Event) {
    const value = (e.currentTarget as HTMLTextAreaElement).value;
    localChapter.synopsis = value || undefined;
    markDirty();
  }

  function handleTargetWordsInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    if (raw === '') {
      localChapter.targetWords = undefined;
    } else {
      const parsed = parseInt(raw, 10);
      if (!isNaN(parsed) && parsed >= 0) {
        localChapter.targetWords = parsed;
      }
    }
    markDirty();
  }
</script>

{#if localChapter}
<div class="chapter-inspector">
  <!-- Title -->
  <div class="field-group title-group">
    <input
      class="title-input"
      type="text"
      value={localChapter.title}
      placeholder="Chapter title..."
      oninput={handleTitleInput}
      aria-label="Chapter title"
    />
    {#if isDirty}
      <span class="unsaved-indicator" aria-label="Unsaved changes">Unsaved</span>
    {/if}
  </div>

  <!-- POV Character -->
  <div class="field-group">
    <label class="field-label" for="chapter-pov">POV Character</label>
    <select
      id="chapter-pov"
      class="field-input field-select"
      value={localChapter.pov ?? ''}
      onchange={handlePovChange}
    >
      <option value="">None</option>
      {#each characters as character (character.slug)}
        <option value={character.slug} selected={localChapter.pov === character.slug}>
          {character.title}
        </option>
      {/each}
    </select>
  </div>

  <!-- Status -->
  <div class="field-group">
    <label class="field-label" for="chapter-status">Status</label>
    <select
      id="chapter-status"
      class="field-input field-select"
      value={localChapter.status}
      onchange={handleStatusChange}
    >
      <option value="draft">Draft</option>
      <option value="revised">Revised</option>
      <option value="final">Final</option>
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
    >{localChapter.synopsis ?? ''}</textarea>
  </div>

  <!-- Word Count -->
  <div class="field-group">
    <label class="field-label">Word Count</label>
    <div class="word-count-display">
      <span class="word-count-current">{currentWords.toLocaleString()}</span>
      {#if localChapter.targetWords && localChapter.targetWords > 0}
        <span class="word-count-separator">/</span>
        <span class="word-count-target">{localChapter.targetWords.toLocaleString()}</span>
        <span class="word-count-unit">words</span>
      {:else}
        <span class="word-count-unit">words</span>
      {/if}
    </div>
    {#if localChapter.targetWords && localChapter.targetWords > 0}
      <div class="progress-bar" role="progressbar" aria-valuenow={progressPercent} aria-valuemin={0} aria-valuemax={100}>
        <div
          class="progress-fill"
          class:progress-complete={progressPercent >= 100}
          style="width: {progressPercent}%"
        ></div>
      </div>
      <span class="progress-label">{progressPercent}%</span>
    {/if}
  </div>

  <!-- Target Words -->
  <div class="field-group">
    <label class="field-label" for="chapter-target-words">Target Words</label>
    <input
      id="chapter-target-words"
      class="field-input"
      type="number"
      value={localChapter.targetWords ?? ''}
      placeholder="e.g. 3000"
      min="0"
      oninput={handleTargetWordsInput}
    />
  </div>
</div>
{/if}

<style>
  .chapter-inspector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-sm);
  }

  /* -- Title ---------------------------------------------------------------- */

  .title-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .title-input {
    flex: 1;
    padding: var(--spacing-xs) 0;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-tight);
    transition: border-color var(--transition-fast);
  }

  .title-input:focus {
    outline: none;
    border-bottom-color: var(--accent-primary);
    box-shadow: none;
  }

  .title-input::placeholder {
    color: var(--text-tertiary);
    font-weight: var(--font-weight-normal);
  }

  .unsaved-indicator {
    flex-shrink: 0;
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-full);
    background: var(--color-warning);
    color: var(--text-inverse);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* -- Field Groups --------------------------------------------------------- */

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
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
    font-size: var(--font-size-base);
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

  /* -- Word Count ----------------------------------------------------------- */

  .word-count-display {
    display: flex;
    align-items: baseline;
    gap: var(--spacing-xs);
    font-size: var(--font-size-base);
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
    background: var(--accent-primary);
    border-radius: var(--radius-full);
    transition: width var(--transition-normal);
  }

  .progress-complete {
    background: var(--color-success, var(--accent-primary));
  }

  .progress-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }
</style>
