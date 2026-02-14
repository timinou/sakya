<script lang="ts">
  import { editorState } from '$lib/stores';
</script>

<footer class="status-bar">
  <div class="status-left">
    <span class="word-count">{editorState.wordCount.words} words</span>
  </div>
  <div class="status-right">
    <span class="save-status" class:saving={editorState.isSaving}>
      {editorState.isSaving ? 'Saving...' : 'Saved'}
    </span>
  </div>
</footer>

<style>
  .status-bar {
    height: var(--statusbar-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--spacing-sm);
    border-top: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    grid-column: 1 / -1;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .save-status.saving {
    color: var(--color-warning);
  }
</style>
