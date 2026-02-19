<script lang="ts">
  import type { NoteEntry } from '$lib/types';

  interface Props {
    note: NoteEntry;
    bodyPreview?: string;
    onOpenInTab?: (slug: string) => void;
  }

  let { note, bodyPreview = '', onOpenInTab }: Props = $props();

  let truncatedPreview = $derived(
    bodyPreview.length > 200 ? bodyPreview.slice(0, 200) + '...' : bodyPreview
  );
</script>

<div class="note-inspector">
  <!-- Color -->
  {#if note.color}
    <div class="field-group">
      <span class="field-label">Color</span>
      <div class="color-display">
        <span class="color-dot" style:background-color={note.color}></span>
        <span class="color-value">{note.color}</span>
      </div>
    </div>
  {/if}

  <!-- Label -->
  {#if note.label}
    <div class="field-group">
      <span class="field-label">Label</span>
      <span class="label-value">{note.label}</span>
    </div>
  {/if}

  <!-- Body preview -->
  {#if truncatedPreview}
    <div class="field-group">
      <span class="field-label">Preview</span>
      <p class="body-preview">{truncatedPreview}</p>
    </div>
  {/if}

  <!-- Open in Editor button -->
  <div class="actions">
    <button
      class="open-btn"
      type="button"
      onclick={() => onOpenInTab?.(note.slug)}
    >
      Open in Editor
    </button>
  </div>
</div>

<style>
  .note-inspector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .field-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .color-display {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .color-dot {
    display: block;
    width: 12px;
    height: 12px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .color-value {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
  }

  .label-value {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .body-preview {
    margin: 0;
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .actions {
    margin-top: var(--spacing-xs);
  }

  .open-btn {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary, #555);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast);
  }

  .open-btn:hover {
    background: var(--accent-primary, #7c4dbd);
    color: var(--text-inverse, #fff);
    border-color: var(--accent-primary, #7c4dbd);
  }
</style>
