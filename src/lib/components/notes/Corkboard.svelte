<script lang="ts">
  import { StickyNote, Plus } from 'lucide-svelte';
  import type { NoteEntry, CorkboardSize } from '$lib/types';
  import { notesStore, editorState, projectState, uiState, navigationStore } from '$lib/stores';
  import NoteCard from './NoteCard.svelte';

  interface Props {
    notes: NoteEntry[];
    noteExcerpts?: Record<string, string>;
    onSelectNote?: (slug: string) => void;
  }

  let {
    notes,
    noteExcerpts = {},
    onSelectNote,
  }: Props = $props();

  // Editing state
  let editingSlug = $state<string | null>(null);
  let editingBodies = $state<Record<string, string>>({});

  // Debounce timers
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Clean up timers on unmount
  $effect(() => {
    return () => {
      if (saveTimeout) clearTimeout(saveTimeout);
      if (autoSaveTimeout) clearTimeout(autoSaveTimeout);
    };
  });

  function debouncedSave(): void {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      const path = projectState.projectPath;
      if (path) {
        notesStore.saveConfig(path).catch((e) => {
          console.error('Failed to save notes config:', e);
        });
      }
    }, 500);
  }

  function immediateSave(): void {
    if (saveTimeout) clearTimeout(saveTimeout);
    const path = projectState.projectPath;
    if (path) {
      notesStore.saveConfig(path).catch((e) => {
        console.error('Failed to save notes config:', e);
      });
    }
  }

  function handleDragEnd(slug: string, position: { x: number; y: number }): void {
    notesStore.updateCardPosition(slug, position);
    debouncedSave();
  }

  function handleColorChange(slug: string, color: string): void {
    notesStore.updateCardColor(slug, color);
    immediateSave();
  }

  function handleLabelChange(slug: string, label: string): void {
    notesStore.updateCardLabel(slug, label);
    immediateSave();
  }

  function handleSizeChange(slug: string, size: CorkboardSize): void {
    notesStore.updateCardSize(slug, size);
    debouncedSave();
  }

  function handleNoteClick(slug: string): void {
    notesStore.selectNote(slug);
    onSelectNote?.(slug);
  }

  // --- Inline editing ---

  async function handleEditStart(slug: string): Promise<void> {
    // If note is already open in a tab, switch to that tab instead
    const tabId = `note:${slug}`;
    const existingTab = editorState.tabs.find((t) => t.id === tabId);
    if (existingTab) {
      navigationStore.switchToTab(tabId);
      uiState.setViewMode('editor');
      return;
    }

    // Load content if not already cached
    const path = projectState.projectPath;
    if (!path) return;

    try {
      const content = await notesStore.loadNoteContent(path, slug);
      editingBodies[slug] = content.body;
      editingSlug = slug;
    } catch (e) {
      console.error('Failed to load note for inline editing:', e);
    }
  }

  function handleEditEnd(slug: string, body: string, isDirty: boolean): void {
    // Clear any pending auto-save
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
      autoSaveTimeout = null;
    }
    if (isDirty) {
      saveNoteContent(slug, body);
    }
    editingSlug = null;
  }

  function handleEditInput(slug: string, body: string): void {
    // Debounced auto-save (1.5s)
    if (autoSaveTimeout) clearTimeout(autoSaveTimeout);
    autoSaveTimeout = setTimeout(() => {
      saveNoteContent(slug, body);
    }, 1500);
  }

  function saveNoteContent(slug: string, body: string): void {
    const path = projectState.projectPath;
    if (!path) return;
    const note = notes.find((n) => n.slug === slug);
    if (!note) return;

    notesStore.saveNoteContent(path, slug, note.title, body).catch((e) => {
      console.error('Failed to save note content:', e);
    });
  }

  function handleOpenInTab(slug: string): void {
    // Exit edit mode first
    editingSlug = null;
    // Navigate to the note (clears cross-type selection) and switch to editor mode
    navigationStore.navigateTo({ type: 'note', slug });
    uiState.setViewMode('editor');
  }

  function handleCorkboardClick(e: MouseEvent): void {
    // If clicking on the corkboard background (not on a card), exit edit mode
    const target = e.target as HTMLElement;
    if (target.classList.contains('corkboard') && editingSlug) {
      const slug = editingSlug;
      const body = editingBodies[slug] ?? '';
      handleEditEnd(slug, body, true);
    }
  }

  async function handleCreateFirst(): Promise<void> {
    const path = projectState.projectPath;
    if (!path) return;
    try {
      await notesStore.createNote(path, 'Untitled Note');
    } catch (e) {
      console.error('Failed to create note:', e);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="corkboard" onclick={handleCorkboardClick}>
  {#if notes.length === 0}
    <div class="empty-state">
      <StickyNote size={48} strokeWidth={1} />
      <p class="empty-title">Create your first note</p>
      <p class="empty-desc">Notes appear as cards on this corkboard</p>
      <button class="create-btn" type="button" onclick={handleCreateFirst}>
        <Plus size={16} />
        <span>New Note</span>
      </button>
    </div>
  {:else}
    {#each notes as note (note.slug)}
      <NoteCard
        {note}
        excerpt={noteExcerpts[note.slug] ?? ''}
        isEditing={editingSlug === note.slug}
        editBody={editingBodies[note.slug] ?? ''}
        onDragEnd={handleDragEnd}
        onColorChange={handleColorChange}
        onLabelChange={handleLabelChange}
        onSizeChange={handleSizeChange}
        onEditStart={handleEditStart}
        onEditEnd={handleEditEnd}
        onEditInput={handleEditInput}
        onOpenInTab={handleOpenInTab}
        onclick={() => handleNoteClick(note.slug)}
      />
    {/each}
  {/if}
</div>

<style>
  .corkboard {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: var(--bg-primary);
    background-image: radial-gradient(
      circle,
      var(--border-secondary) 1px,
      transparent 1px
    );
    background-size: 24px 24px;
  }

  .empty-state {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
    color: var(--text-tertiary);
    text-align: center;
    user-select: none;
  }

  .empty-title {
    margin: 0;
    font-size: var(--font-size-lg, 1.125rem);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
  }

  .empty-desc {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-tertiary);
  }

  .create-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-md);
    border: 1px solid var(--border-primary);
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

  .create-btn:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent-primary, #7c4dbd);
  }
</style>
