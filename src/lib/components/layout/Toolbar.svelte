<script lang="ts">
  import { uiState, projectState } from '$lib/stores';
  import { PanelLeft, PanelRight, Sun, Moon, Monitor } from 'lucide-svelte';
  import type { Theme, ViewMode } from '$lib/types';

  const themes: Theme[] = ['light', 'dark', 'system'];

  function cycleTheme() {
    const currentIndex = themes.indexOf(uiState.theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    uiState.setTheme(themes[nextIndex]);
  }

  function setViewMode(mode: ViewMode) {
    uiState.setViewMode(mode);
  }
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <span class="project-name">
      {projectState.manifest?.name ?? 'Sakya'}
    </span>
  </div>

  <div class="toolbar-center">
    <div class="view-mode-group" role="group" aria-label="View mode">
      <button
        class="view-mode-btn"
        class:active={uiState.viewMode === 'editor'}
        onclick={() => setViewMode('editor')}
        aria-pressed={uiState.viewMode === 'editor'}
      >
        Editor
      </button>
      <button
        class="view-mode-btn"
        class:active={uiState.viewMode === 'corkboard'}
        onclick={() => setViewMode('corkboard')}
        aria-pressed={uiState.viewMode === 'corkboard'}
      >
        Corkboard
      </button>
      <button
        class="view-mode-btn"
        class:active={uiState.viewMode === 'split'}
        onclick={() => setViewMode('split')}
        aria-pressed={uiState.viewMode === 'split'}
      >
        Split
      </button>
    </div>
  </div>

  <div class="toolbar-right">
    <button
      class="toolbar-btn"
      onclick={cycleTheme}
      title="Theme: {uiState.theme}"
      aria-label="Cycle theme (current: {uiState.theme})"
    >
      {#if uiState.theme === 'light'}
        <Sun size={16} />
      {:else if uiState.theme === 'dark'}
        <Moon size={16} />
      {:else}
        <Monitor size={16} />
      {/if}
    </button>
    <button
      class="toolbar-btn"
      class:active={uiState.panes.binderVisible}
      onclick={() => uiState.toggleBinder()}
      title="Toggle Binder"
      aria-label="Toggle binder panel"
      aria-pressed={uiState.panes.binderVisible}
    >
      <PanelLeft size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={uiState.panes.inspectorVisible}
      onclick={() => uiState.toggleInspector()}
      title="Toggle Inspector"
      aria-label="Toggle inspector panel"
      aria-pressed={uiState.panes.inspectorVisible}
    >
      <PanelRight size={16} />
    </button>
  </div>
</header>

<style>
  .toolbar {
    height: var(--toolbar-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
    grid-column: 1 / -1;
    user-select: none;
    -webkit-app-region: drag;
  }

  .toolbar-left,
  .toolbar-center,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    -webkit-app-region: no-drag;
  }

  .toolbar-left {
    flex: 1;
    min-width: 0;
  }

  .toolbar-center {
    flex: 0 0 auto;
  }

  .toolbar-right {
    flex: 1;
    justify-content: flex-end;
  }

  .project-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .view-mode-group {
    display: flex;
    gap: 1px;
    background: var(--border-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .view-mode-btn {
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    border-radius: 0;
    background: var(--bg-tertiary);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--transition-fast), color var(--transition-fast);
    box-shadow: none;
  }

  .view-mode-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border-color: transparent;
    box-shadow: none;
  }

  .view-mode-btn.active {
    background: var(--accent-primary);
    color: var(--text-inverse);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--transition-fast), color var(--transition-fast);
    box-shadow: none;
  }

  .toolbar-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: transparent;
    box-shadow: none;
  }

  .toolbar-btn.active {
    color: var(--accent-primary);
  }
</style>
