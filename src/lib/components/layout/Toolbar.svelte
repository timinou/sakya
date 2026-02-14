<script lang="ts">
  import { uiState, projectState, sprintStore, editorState } from '$lib/stores';
  import { PanelLeft, PanelRight, Sun, Moon, Monitor, Eye, Check, Timer, BarChart3 } from 'lucide-svelte';
  import type { Theme, ViewMode } from '$lib/types';

  const themes: Theme[] = ['light', 'dark', 'system'];

  let focusDropdownOpen = $state(false);
  let dropdownRef = $state<HTMLDivElement | null>(null);
  let triggerRef = $state<HTMLButtonElement | null>(null);

  // Whether any focus mode is active (to highlight the trigger button)
  let anyFocusModeActive = $derived(
    uiState.typewriterMode || uiState.focusMode || uiState.distractionFreeMode
  );

  function cycleTheme() {
    const currentIndex = themes.indexOf(uiState.theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    uiState.setTheme(themes[nextIndex]);
  }

  function setViewMode(mode: ViewMode) {
    uiState.setViewMode(mode);
  }

  function toggleFocusDropdown() {
    focusDropdownOpen = !focusDropdownOpen;
  }

  function handleDropdownKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      focusDropdownOpen = false;
      triggerRef?.focus();
    }
  }

  function toggleSprintPanel() {
    window.dispatchEvent(new CustomEvent('sakya:toggle-sprint'));
  }

  function openStats() {
    editorState.openDocument({
      id: 'stats:writing',
      title: 'Writing Stats',
      documentType: 'stats',
      documentSlug: 'writing',
      isDirty: false,
    });
  }

  function handleClickOutside(e: MouseEvent) {
    if (!focusDropdownOpen) return;
    const target = e.target as Node;
    if (
      dropdownRef && !dropdownRef.contains(target) &&
      triggerRef && !triggerRef.contains(target)
    ) {
      focusDropdownOpen = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

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
    <!-- Sprint timer toggle -->
    <button
      class="toolbar-btn"
      class:active={sprintStore.isActive}
      onclick={toggleSprintPanel}
      title={sprintStore.isActive ? 'Sprint in progress' : 'Start Sprint'}
      aria-label={sprintStore.isActive ? 'Sprint in progress — click to show timer' : 'Open sprint timer'}
    >
      <Timer size={16} />
    </button>

    <!-- Writing Stats -->
    <button
      class="toolbar-btn"
      class:active={editorState.activeTab?.documentType === 'stats'}
      onclick={openStats}
      title="Writing Stats"
      aria-label="Open writing statistics"
    >
      <BarChart3 size={16} />
    </button>

    <!-- Focus modes dropdown -->
    <div class="focus-dropdown-wrapper">
      <button
        bind:this={triggerRef}
        class="toolbar-btn"
        class:active={anyFocusModeActive}
        onclick={toggleFocusDropdown}
        title="Focus Modes"
        aria-label="Focus modes menu"
        aria-haspopup="true"
        aria-expanded={focusDropdownOpen}
      >
        <Eye size={16} />
      </button>

      {#if focusDropdownOpen}
        <div
          bind:this={dropdownRef}
          class="focus-dropdown"
          role="menu"
          aria-label="Focus modes"
          tabindex="-1"
          onkeydown={handleDropdownKeydown}
        >
          <button
            class="focus-dropdown-item"
            role="menuitemcheckbox"
            aria-checked={uiState.typewriterMode}
            onclick={() => { uiState.toggleTypewriterMode(); }}
          >
            <span class="focus-dropdown-check">
              {#if uiState.typewriterMode}<Check size={14} />{/if}
            </span>
            <span class="focus-dropdown-label">Typewriter Mode</span>
            <span class="focus-dropdown-shortcut">Ctrl+Shift+T</span>
          </button>
          <button
            class="focus-dropdown-item"
            role="menuitemcheckbox"
            aria-checked={uiState.focusMode}
            onclick={() => { uiState.toggleFocusMode(); }}
          >
            <span class="focus-dropdown-check">
              {#if uiState.focusMode}<Check size={14} />{/if}
            </span>
            <span class="focus-dropdown-label">Focus Mode</span>
            <span class="focus-dropdown-shortcut">Ctrl+Shift+.</span>
          </button>
          <button
            class="focus-dropdown-item"
            role="menuitemcheckbox"
            aria-checked={uiState.distractionFreeMode}
            onclick={() => { uiState.toggleDistractionFreeMode(); }}
          >
            <span class="focus-dropdown-check">
              {#if uiState.distractionFreeMode}<Check size={14} />{/if}
            </span>
            <span class="focus-dropdown-label">Distraction-Free</span>
            <span class="focus-dropdown-shortcut">Ctrl+Shift+F</span>
          </button>
        </div>
      {/if}
    </div>

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

  @keyframes sprint-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  /* --- Focus dropdown --- */
  .focus-dropdown-wrapper {
    position: relative;
  }

  .focus-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 220px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    z-index: 200;
    padding: var(--spacing-xs) 0;
    -webkit-app-region: no-drag;
  }

  .focus-dropdown-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    text-align: left;
    box-shadow: none;
    transition: background-color var(--transition-fast);
  }

  .focus-dropdown-item:hover {
    background: var(--bg-tertiary);
    border-color: transparent;
    box-shadow: none;
  }

  .focus-dropdown-check {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    color: var(--accent-primary);
  }

  .focus-dropdown-label {
    flex: 1;
  }

  .focus-dropdown-shortcut {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    margin-left: auto;
    padding-left: var(--spacing-sm);
  }
</style>
