<script lang="ts">
  import type { ComponentType } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    Search,
    BookOpen,
    StickyNote,
    Users,
    MapPin,
    Package,
    Lightbulb,
    File,
    Loader2,
  } from 'lucide-svelte';
  import { projectState } from '$lib/stores';

  interface SearchResult {
    title: string;
    slug: string;
    fileType: string;
    entityType?: string;
    matchingLine: string;
    lineNumber: number;
    contextBefore: string;
    contextAfter: string;
  }

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onNavigate?: (fileType: string, slug: string, entityType?: string) => void;
  }

  let { isOpen, onClose, onNavigate }: Props = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let selectedIndex = $state(0);
  let searchInputEl = $state<HTMLInputElement | null>(null);
  let resultsContainerEl = $state<HTMLDivElement | null>(null);
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = ComponentType<any>;

  // Entity type icon mapping
  const entityIcons: Record<string, IconComponent> = {
    character: Users,
    place: MapPin,
    item: Package,
    idea: Lightbulb,
  };

  // Entity type color mapping
  const entityColors: Record<string, string> = {
    character: 'var(--color-entity-character)',
    place: 'var(--color-entity-place)',
    item: 'var(--color-entity-item)',
    idea: 'var(--color-entity-idea)',
  };

  function getIconForResult(result: SearchResult): IconComponent {
    if (result.fileType === 'chapter') return BookOpen;
    if (result.fileType === 'note') return StickyNote;
    if (result.fileType === 'entity' && result.entityType) {
      return entityIcons[result.entityType] ?? File;
    }
    return File;
  }

  function getColorForResult(result: SearchResult): string | undefined {
    if (result.fileType === 'entity' && result.entityType) {
      return entityColors[result.entityType];
    }
    return undefined;
  }

  // Group results by file type
  interface ResultGroup {
    label: string;
    results: SearchResult[];
    startIndex: number;
  }

  let groupedResults = $derived.by(() => {
    const groups: ResultGroup[] = [];
    const byType: Record<string, SearchResult[]> = {};

    for (const r of results) {
      const key = r.fileType;
      if (!byType[key]) byType[key] = [];
      byType[key].push(r);
    }

    const typeLabels: Record<string, string> = {
      chapter: 'Chapters',
      entity: 'Entities',
      note: 'Notes',
    };

    const typeOrder = ['chapter', 'entity', 'note'];
    let runningIndex = 0;

    for (const type of typeOrder) {
      if (byType[type] && byType[type].length > 0) {
        groups.push({
          label: typeLabels[type] ?? type,
          results: byType[type],
          startIndex: runningIndex,
        });
        runningIndex += byType[type].length;
      }
    }

    return groups;
  });

  let flatResults = $derived(groupedResults.flatMap((g) => g.results));

  // Debounced search
  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);

    if (!query.trim()) {
      results = [];
      selectedIndex = 0;
      isSearching = false;
      return;
    }

    isSearching = true;
    debounceTimer = setTimeout(async () => {
      await performSearch(query.trim());
    }, 300);
  }

  async function performSearch(searchQuery: string) {
    const projectPath = projectState.projectPath;
    if (!projectPath || !searchQuery) {
      results = [];
      isSearching = false;
      return;
    }

    try {
      const searchResults = await invoke<SearchResult[]>('search_project', {
        projectPath,
        query: searchQuery,
      });
      results = searchResults;
      selectedIndex = 0;
    } catch (err) {
      console.error('[SearchPalette] Search failed:', err);
      results = [];
    } finally {
      isSearching = false;
    }
  }

  function handleSelect(result: SearchResult) {
    onNavigate?.(result.fileType, result.slug, result.entityType);
    handleClose();
  }

  function handleClose() {
    query = '';
    results = [];
    selectedIndex = 0;
    isSearching = false;
    if (debounceTimer) clearTimeout(debounceTimer);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (flatResults.length > 0) {
        selectedIndex = (selectedIndex + 1) % flatResults.length;
        scrollSelectedIntoView();
      }
      return;
    }

    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (flatResults.length > 0) {
        selectedIndex = (selectedIndex - 1 + flatResults.length) % flatResults.length;
        scrollSelectedIntoView();
      }
      return;
    }

    if (e.key === 'Enter') {
      e.preventDefault();
      if (flatResults.length > 0 && flatResults[selectedIndex]) {
        handleSelect(flatResults[selectedIndex]);
      }
      return;
    }
  }

  function scrollSelectedIntoView() {
    // Use requestAnimationFrame to let DOM update first
    requestAnimationFrame(() => {
      const container = resultsContainerEl;
      if (!container) return;

      const selectedEl = container.querySelector('[data-selected="true"]');
      if (selectedEl) {
        selectedEl.scrollIntoView({ block: 'nearest' });
      }
    });
  }

  // Focus the input when modal opens
  $effect(() => {
    if (isOpen && searchInputEl) {
      // Use microtask to ensure the element is rendered
      queueMicrotask(() => {
        searchInputEl?.focus();
      });
    }
  });

  // Handle backdrop click
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  }

  // Focus trap: keep focus within the modal
  function handleFocusTrap(e: KeyboardEvent) {
    if (e.key !== 'Tab') return;

    // In a search palette, we really only need focus on the input
    // so just prevent tab from leaving
    e.preventDefault();
    searchInputEl?.focus();
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="search-backdrop"
    onmousedown={handleBackdropClick}
    onkeydown={handleFocusTrap}
    role="dialog"
    aria-modal="true"
    aria-label="Search project"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="search-modal" onkeydown={handleKeydown}>
      <div class="search-input-wrapper">
        <Search size={18} strokeWidth={2} />
        <input
          bind:this={searchInputEl}
          type="text"
          class="search-input"
          placeholder="Search chapters, entities, notes..."
          bind:value={query}
          oninput={handleInput}
          aria-label="Search query"
          aria-autocomplete="list"
          aria-controls="search-results"
          aria-activedescendant={flatResults.length > 0 ? `search-result-${selectedIndex}` : undefined}
        />
        {#if isSearching}
          <div class="search-spinner">
            <Loader2 size={16} strokeWidth={2} />
          </div>
        {/if}
        <kbd class="search-shortcut">ESC</kbd>
      </div>

      <div class="search-results" id="search-results" bind:this={resultsContainerEl} role="listbox">
        {#if query.trim() && !isSearching && flatResults.length === 0}
          <div class="search-empty">
            No results for &ldquo;{query}&rdquo;
          </div>
        {:else}
          {#each groupedResults as group}
            <div class="result-group">
              <div class="result-group-header">{group.label}</div>
              {#each group.results as result, i}
                {@const globalIndex = group.startIndex + i}
                {@const isSelected = globalIndex === selectedIndex}
                {@const Icon = getIconForResult(result)}
                {@const color = getColorForResult(result)}
                <button
                  id="search-result-{globalIndex}"
                  class="result-item"
                  class:selected={isSelected}
                  data-selected={isSelected}
                  role="option"
                  aria-selected={isSelected}
                  onmouseenter={() => { selectedIndex = globalIndex; }}
                  onclick={() => handleSelect(result)}
                >
                  <span class="result-icon" style:color={color}>
                    <Icon size={16} strokeWidth={2} />
                  </span>
                  <span class="result-content">
                    <span class="result-title">{result.title}</span>
                    {#if result.entityType}
                      <span
                        class="result-entity-type"
                        style:color={color}
                      >{result.entityType}</span>
                    {/if}
                    {#if result.matchingLine}
                      <span class="result-match">{result.matchingLine.trim()}</span>
                    {/if}
                  </span>
                  {#if result.lineNumber > 0}
                    <span class="result-line">L{result.lineNumber}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/each}
        {/if}
      </div>

      {#if flatResults.length > 0}
        <div class="search-footer">
          <span class="footer-hint">
            <kbd>&uarr;</kbd><kbd>&darr;</kbd> navigate
          </span>
          <span class="footer-hint">
            <kbd>&crarr;</kbd> open
          </span>
          <span class="footer-hint">
            <kbd>esc</kbd> close
          </span>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    inset: 0;
    background: var(--bg-overlay);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: var(--z-modal);
    animation: fadeIn 150ms ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .search-modal {
    max-width: 600px;
    width: 90%;
    background: var(--bg-elevated);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-secondary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideIn 150ms ease;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-secondary);
    color: var(--text-tertiary);
  }

  .search-input {
    flex: 1;
    border: none;
    background: none;
    font-size: var(--font-size-lg);
    color: var(--text-primary);
    outline: none;
    padding: 0;
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--text-tertiary);
  }

  .search-input:focus {
    border: none;
    box-shadow: none;
  }

  .search-spinner {
    display: flex;
    align-items: center;
    color: var(--accent-primary);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .search-shortcut {
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-results {
    max-height: 400px;
    overflow-y: auto;
    padding: var(--spacing-xs) 0;
  }

  .search-empty {
    padding: var(--spacing-xl) var(--spacing-md);
    text-align: center;
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
  }

  .result-group {
    padding: var(--spacing-xs) 0;
  }

  .result-group-header {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: none;
    cursor: pointer;
    text-align: left;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    border-radius: 0;
    transition: background-color var(--transition-fast);
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--bg-tertiary);
  }

  .result-item:focus-visible {
    outline: none;
    background: var(--bg-tertiary);
  }

  .result-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    color: var(--text-secondary);
  }

  .result-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .result-title {
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-entity-type {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    text-transform: capitalize;
  }

  .result-match {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-line {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  .search-footer {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    border-top: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
  }

  .footer-hint {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .footer-hint kbd {
    font-family: var(--font-sans);
    font-size: 10px;
    padding: 1px 4px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: 3px;
    min-width: 18px;
    text-align: center;
  }
</style>
