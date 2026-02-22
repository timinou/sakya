<script lang="ts">
  import { untrack } from 'svelte';
  import { onMount } from 'svelte';
  import { ArrowLeft, FileText, Plus, BookOpen, Layout } from 'lucide-svelte';
  import { notebookStore } from '$lib/stores';
  import SakyaEditor from '$lib/editor/SakyaEditor.svelte';
  import Corkboard from '$lib/components/notes/Corkboard.svelte';

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  // View mode — editor (two-column) or corkboard
  type ViewMode = 'editor' | 'corkboard';
  let viewMode = $state<ViewMode>('editor');

  // Selected note and its loaded content
  let selectedSlug = $state<string | null>(null);
  let loadedBody = $state<string | null>(null);
  let isLoadingContent = $state(false);

  // Inline note creation
  let isCreating = $state(false);
  let newNoteTitle = $state('');
  let createInputEl = $state<HTMLInputElement | null>(null);

  // Derived: the currently selected NoteEntry
  let selectedNote = $derived(
    selectedSlug ? notebookStore.notes.find((n) => n.slug === selectedSlug) ?? null : null
  );

  // Load notebook on mount (guard: only if not already loaded)
  onMount(() => {
    if (!notebookStore.isLoaded && !notebookStore.isLoading) {
      notebookStore.loadConfig().catch((e) => {
        console.error('[NotebookView] Failed to load notebook config:', e);
      });
    }
  });

  // Auto-focus the create input when creation mode opens
  $effect(() => {
    if (isCreating && createInputEl) {
      createInputEl.focus();
    }
  });

  // Load content when selectedSlug changes, using path-based guard to avoid loops
  $effect(() => {
    const slug = selectedSlug;
    if (!slug) {
      loadedBody = null;
      return;
    }
    // Use untrack so that reading loadedBody does not create a dependency cycle
    untrack(() => loadNoteContent(slug));
  });

  async function loadNoteContent(slug: string): Promise<void> {
    isLoadingContent = true;
    try {
      const content = await notebookStore.loadNoteContent(slug);
      // Only apply if selection has not changed while we awaited
      if (selectedSlug === slug) {
        loadedBody = content.body;
      }
    } catch (e) {
      console.error('[NotebookView] Failed to load note content:', e);
    } finally {
      if (selectedSlug === slug) {
        isLoadingContent = false;
      }
    }
  }

  function handleNoteClick(slug: string): void {
    if (selectedSlug === slug) return;
    selectedSlug = slug;
    loadedBody = null;
  }

  // --- Create note ---

  function handleAddNote(): void {
    isCreating = true;
  }

  async function confirmCreate(): Promise<void> {
    const title = newNoteTitle.trim();
    if (!title) {
      cancelCreate();
      return;
    }
    try {
      await notebookStore.createNote(title);
      // Select the newly created note (last in list after reload)
      const last = notebookStore.notes[notebookStore.notes.length - 1];
      if (last) {
        selectedSlug = last.slug;
        loadedBody = null;
      }
    } catch (e) {
      console.error('[NotebookView] Failed to create note:', e);
    }
    cancelCreate();
  }

  function cancelCreate(): void {
    isCreating = false;
    newNoteTitle = '';
  }

  function handleCreateKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmCreate();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelCreate();
    }
  }

  // --- Save ---

  async function handleSave(markdown: string): Promise<void> {
    if (!selectedSlug || !selectedNote) return;
    try {
      await notebookStore.saveNoteContent(selectedSlug, selectedNote.title, markdown);
      // Keep local cache up to date
      loadedBody = markdown;
    } catch (e) {
      console.error('[NotebookView] Failed to save note:', e);
    }
  }

  // Ctrl+S forwarded from the window (matches the app-wide convention)
  function handleWindowKeydown(e: KeyboardEvent): void {
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      // The SakyaEditor AutoSavePlugin handles Ctrl+S internally and calls onSave.
      // We also dispatch the custom event as a belt-and-suspenders signal.
      window.dispatchEvent(new CustomEvent('sakya:save'));
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="notebook-view">
  <!-- Toolbar -->
  <header class="notebook-toolbar">
    <div class="toolbar-left">
      <button class="back-btn" onclick={onBack} title="Back to launcher" aria-label="Back to launcher">
        <ArrowLeft size={16} />
      </button>
      <div class="title-group">
        <BookOpen size={15} class="title-icon" />
        <span class="title">Notebook</span>
      </div>
    </div>

    <div class="toolbar-center">
      <div class="view-mode-group" role="group" aria-label="View mode">
        <button
          class="view-mode-btn"
          class:active={viewMode === 'editor'}
          onclick={() => (viewMode = 'editor')}
          aria-pressed={viewMode === 'editor'}
          title="Editor view"
        >
          <FileText size={13} />
          Editor
        </button>
        <button
          class="view-mode-btn"
          class:active={viewMode === 'corkboard'}
          onclick={() => (viewMode = 'corkboard')}
          aria-pressed={viewMode === 'corkboard'}
          title="Corkboard view"
        >
          <Layout size={13} />
          Corkboard
        </button>
      </div>
    </div>

    <div class="toolbar-right">
      <!-- Spacer to keep center balanced -->
    </div>
  </header>

  <!-- Content area -->
  <div class="notebook-content">
    {#if viewMode === 'editor'}
      <!-- Two-column layout: note list + editor -->
      <aside class="note-list" aria-label="Note list">
        <div class="note-list-scroll">
          {#if notebookStore.isLoading && !notebookStore.isLoaded}
            <div class="list-loading">
              <span class="spinner" aria-hidden="true"></span>
              <span>Loading notes…</span>
            </div>
          {:else if notebookStore.notes.length === 0 && !isCreating}
            <button class="empty-list-cta" type="button" onclick={handleAddNote}>
              <Plus size={12} />
              Add first note
            </button>
          {:else}
            {#each notebookStore.notes as note (note.slug)}
              <button
                class="note-item"
                class:selected={note.slug === selectedSlug}
                type="button"
                onclick={() => handleNoteClick(note.slug)}
                title={note.title}
              >
                <FileText size={13} class="note-item-icon" />
                <span class="note-item-label">{note.title}</span>
                {#if note.color}
                  <span
                    class="note-color-dot"
                    style:background-color={note.color}
                    aria-hidden="true"
                  ></span>
                {/if}
              </button>
            {/each}
          {/if}

          {#if isCreating}
            <div class="inline-input-wrapper">
              <input
                bind:this={createInputEl}
                bind:value={newNoteTitle}
                class="inline-input"
                type="text"
                placeholder="Note title…"
                onkeydown={handleCreateKeydown}
                onblur={confirmCreate}
                aria-label="New note title"
              />
            </div>
          {/if}
        </div>

        <div class="note-list-footer">
          <button
            class="add-note-btn"
            type="button"
            onclick={handleAddNote}
            disabled={isCreating}
            title="Add note"
            aria-label="Add note"
          >
            <Plus size={14} />
            Add Note
          </button>
        </div>
      </aside>

      <!-- Editor panel -->
      <div class="editor-panel">
        {#if isLoadingContent}
          <div class="editor-placeholder">
            <span class="spinner" aria-hidden="true"></span>
            <span>Loading…</span>
          </div>
        {:else if selectedSlug && loadedBody !== null}
          {#key selectedSlug}
            <div class="editor-wrap">
              <SakyaEditor
                content={loadedBody}
                onSave={handleSave}
              />
            </div>
          {/key}
        {:else if selectedSlug && loadedBody === null}
          <!-- Content fetch in flight but isLoadingContent may race — show nothing -->
        {:else}
          <div class="editor-placeholder">
            <FileText size={40} strokeWidth={1} />
            <p class="placeholder-title">Select a note to begin writing</p>
            <p class="placeholder-hint">
              {#if notebookStore.notes.length === 0}
                Create your first note using the "Add Note" button.
              {:else}
                Choose a note from the list on the left.
              {/if}
            </p>
          </div>
        {/if}
      </div>
    {:else}
      <!-- Corkboard mode -->
      <div class="corkboard-wrap">
        <Corkboard notes={notebookStore.notes} />
      </div>
    {/if}
  </div>
</div>

<style>
  /* ── Full-screen shell ─────────────────────────────────────────── */
  .notebook-view {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
    background: var(--bg-primary);
    overflow: hidden;
  }

  /* ── Toolbar ───────────────────────────────────────────────────── */
  .notebook-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--toolbar-height, 40px);
    padding: 0 var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
    flex-shrink: 0;
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

  .back-btn {
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
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
    flex-shrink: 0;
  }

  .back-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    color: var(--text-primary);
  }

  .title-group :global(svg) {
    color: var(--accent-primary);
    flex-shrink: 0;
  }

  .title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    white-space: nowrap;
  }

  /* View mode pill */
  .view-mode-group {
    display: flex;
    gap: 1px;
    background: var(--border-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .view-mode-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    border-radius: 0;
    background: var(--bg-tertiary);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .view-mode-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .view-mode-btn.active {
    background: var(--accent-primary);
    color: var(--text-inverse, #fff);
  }

  /* ── Content area ──────────────────────────────────────────────── */
  .notebook-content {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Note list sidebar ─────────────────────────────────────────── */
  .note-list {
    display: flex;
    flex-direction: column;
    width: 250px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
    overflow: hidden;
  }

  .note-list-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xs) 0;
  }

  .note-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: 6px var(--spacing-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
    overflow: hidden;
  }

  .note-item:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .note-item.selected {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .note-item.selected :global(svg) {
    color: var(--accent-primary);
  }

  .note-item :global(svg) {
    flex-shrink: 0;
    color: var(--text-tertiary);
    transition: color var(--transition-fast);
  }

  .note-item-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-color-dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  /* Inline create input */
  .inline-input-wrapper {
    padding: 2px var(--spacing-xs);
  }

  .inline-input {
    width: 100%;
    padding: 4px var(--spacing-xs);
    border: 1px solid var(--accent-primary, #7c4dbd);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    outline: none;
    box-sizing: border-box;
  }

  .inline-input:focus {
    box-shadow: 0 0 0 1px var(--accent-primary, #7c4dbd);
  }

  .inline-input::placeholder {
    color: var(--text-tertiary);
  }

  /* Empty state CTA inside list */
  .empty-list-cta {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    background: transparent;
    font-size: var(--font-size-xs);
    font-style: italic;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background-color var(--transition-fast);
  }

  .empty-list-cta:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  /* List footer with Add Note button */
  .note-list-footer {
    flex-shrink: 0;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-top: 1px solid var(--border-secondary);
  }

  .add-note-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    font-family: inherit;
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast),
      border-color var(--transition-fast);
  }

  .add-note-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: var(--accent-primary);
  }

  .add-note-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Loading state inside list */
  .list-loading {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm);
    color: var(--text-tertiary);
    font-size: var(--font-size-xs);
  }

  /* ── Editor panel ──────────────────────────────────────────────── */
  .editor-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .editor-wrap {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Placeholder (empty selection) */
  .editor-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    color: var(--text-tertiary);
    text-align: center;
    padding: var(--spacing-xl);
    user-select: none;
  }

  .editor-placeholder :global(svg) {
    opacity: 0.35;
  }

  .placeholder-title {
    margin: 0;
    font-size: var(--font-size-lg, 1.125rem);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
  }

  .placeholder-hint {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-tertiary);
    max-width: 280px;
  }

  /* ── Corkboard mode ────────────────────────────────────────────── */
  .corkboard-wrap {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Spinner ───────────────────────────────────────────────────── */
  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-primary);
    border-top-color: var(--accent-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
