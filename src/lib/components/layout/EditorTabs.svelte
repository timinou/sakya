<script lang="ts">
  import { editorState, manuscriptStore, notesStore } from '$lib/stores';
  import { X } from 'lucide-svelte';

  function switchTab(tabId: string) {
    editorState.switchTab(tabId);
  }

  function closeTab(tabId: string, event: MouseEvent) {
    event.stopPropagation();
    const tab = editorState.tabs.find((t) => t.id === tabId);
    editorState.closeTab(tabId);
    // Clear store selection to prevent $effect from re-opening the tab
    if (tab?.documentType === 'chapter') {
      manuscriptStore.selectChapter('');
    } else if (tab?.documentType === 'note') {
      notesStore.selectNote('');
    }
  }
</script>

{#if editorState.tabs.length > 0}
  <div class="editor-tabs" role="tablist" aria-label="Open documents">
    {#each editorState.tabs as tab (tab.id)}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        class="tab"
        class:active={editorState.activeTabId === tab.id}
        onclick={() => switchTab(tab.id)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') switchTab(tab.id); }}
        role="tab"
        tabindex="0"
        aria-selected={editorState.activeTabId === tab.id}
        aria-label={tab.title}
      >
        <span class="tab-title">{tab.title}</span>
        {#if tab.isDirty}
          <span class="dirty-dot" aria-label="Unsaved changes"></span>
        {/if}
        <button
          class="tab-close"
          onclick={(e) => closeTab(tab.id, e)}
          title="Close tab"
          aria-label="Close {tab.title}"
        >
          <X size={12} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .editor-tabs {
    display: flex;
    align-items: stretch;
    gap: 1px;
    padding: 0 var(--spacing-xs);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-secondary);
    overflow-x: auto;
    scrollbar-width: none;
    user-select: none;
    flex-shrink: 0;
  }

  .editor-tabs::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    white-space: nowrap;
    transition:
      color var(--transition-fast),
      background-color var(--transition-fast),
      border-color var(--transition-fast);
    box-shadow: none;
    min-height: 32px;
  }

  .tab:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border-color: transparent;
    box-shadow: none;
  }

  .tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent-primary);
    background: var(--bg-primary);
  }

  .tab-title {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-warning);
    flex-shrink: 0;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    opacity: 0;
    transition:
      opacity var(--transition-fast),
      color var(--transition-fast),
      background-color var(--transition-fast);
    box-shadow: none;
    flex-shrink: 0;
  }

  .tab:hover .tab-close {
    opacity: 1;
  }

  .tab-close:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
    border-color: transparent;
    box-shadow: none;
  }
</style>
