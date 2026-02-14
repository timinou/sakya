<script lang="ts">
  import { StickyNote, FileText, Plus, Pencil, Trash2 } from 'lucide-svelte';
  import { notesStore, editorState, projectState } from '$lib/stores';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';
  import ContextMenu from '$lib/components/common/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

  interface Props {
    onSelectNote?: (slug: string) => void;
  }

  let { onSelectNote }: Props = $props();

  let isOpen = $state(true);
  let isCreating = $state(false);
  let newNoteTitle = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  // Context menu state
  let contextMenu = $state<{ x: number; y: number; slug: string; title: string } | null>(null);

  // Delete confirmation state
  let deleteTarget = $state<{ slug: string; title: string } | null>(null);

  // Rename state
  let renamingSlug = $state<string | null>(null);
  let renameValue = $state('');
  let renameInputEl = $state<HTMLInputElement | null>(null);

  function handleAdd(): void {
    isCreating = true;
  }

  $effect(() => {
    if (isCreating && inputEl) {
      inputEl.focus();
    }
  });

  // Auto-focus rename input
  $effect(() => {
    if (renamingSlug && renameInputEl) {
      renameInputEl.focus();
      renameInputEl.select();
    }
  });

  async function confirmCreate(): Promise<void> {
    const title = newNoteTitle.trim();
    if (!title || !projectState.projectPath) {
      cancelCreate();
      return;
    }
    try {
      await notesStore.createNote(projectState.projectPath, title);
    } catch (e) {
      console.error('Failed to create note:', e);
    }
    cancelCreate();
  }

  function cancelCreate(): void {
    isCreating = false;
    newNoteTitle = '';
  }

  function handleInputKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmCreate();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelCreate();
    }
  }

  function handleNoteClick(slug: string): void {
    notesStore.selectNote(slug);
    onSelectNote?.(slug);
  }

  // Context menu handlers
  function handleContextMenu(e: MouseEvent, slug: string, title: string): void {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, slug, title };
  }

  function closeContextMenu(): void {
    contextMenu = null;
  }

  // Delete handlers
  function handleDeleteRequest(slug: string, title: string): void {
    closeContextMenu();
    deleteTarget = { slug, title };
  }

  async function confirmDelete(): Promise<void> {
    if (!deleteTarget || !projectState.projectPath) return;
    const { slug } = deleteTarget;
    try {
      // Close open tab for this note
      const tabId = `note:${slug}`;
      editorState.closeTab(tabId);
      await notesStore.deleteNote(projectState.projectPath, slug);
    } catch (e) {
      console.error('Failed to delete note:', e);
    }
    deleteTarget = null;
  }

  function cancelDelete(): void {
    deleteTarget = null;
  }

  // Rename handlers
  function startRename(slug: string, title: string): void {
    closeContextMenu();
    renamingSlug = slug;
    renameValue = title;
  }

  async function confirmRename(): Promise<void> {
    const newTitle = renameValue.trim();
    if (!newTitle || !renamingSlug || !projectState.projectPath || newTitle === getNoteTitle(renamingSlug)) {
      cancelRename();
      return;
    }
    try {
      await notesStore.renameNote(projectState.projectPath, renamingSlug, newTitle);
    } catch (e) {
      console.error('Failed to rename note:', e);
    }
    cancelRename();
  }

  function cancelRename(): void {
    renamingSlug = null;
    renameValue = '';
  }

  function handleRenameKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmRename();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelRename();
    }
  }

  function getNoteTitle(slug: string): string {
    return notesStore.notes.find(n => n.slug === slug)?.title ?? '';
  }

  // Context menu items for a note
  function getContextMenuItems(slug: string, title: string) {
    return [
      { label: 'Rename', icon: Pencil, onclick: () => startRename(slug, title) },
      { label: '', separator: true },
      { label: 'Delete', icon: Trash2, onclick: () => handleDeleteRequest(slug, title) },
    ];
  }

  // Listen for sakya:create-note custom events from WelcomeCard
  $effect(() => {
    function handleCreateNoteEvent() {
      handleAdd();
    }
    window.addEventListener('sakya:create-note', handleCreateNoteEvent);
    return () => {
      window.removeEventListener('sakya:create-note', handleCreateNoteEvent);
    };
  });

  // Auto-load config once on mount (track path to avoid infinite loop with empty notes)
  let loadedPath: string | null = null;

  $effect(() => {
    const path = projectState.projectPath;
    if (path && path !== loadedPath && !notesStore.isLoading) {
      loadedPath = path;
      notesStore.loadConfig(path).catch((e) => {
        console.error('Failed to load notes config:', e);
      });
    }
  });
</script>

<BinderSection
  title="Notes"
  icon={StickyNote}
  count={notesStore.noteCount}
  bind:isOpen={isOpen}
  onAdd={handleAdd}
>
  {#if notesStore.notes.length === 0 && !isCreating}
    <button class="placeholder-cta" type="button" onclick={handleAdd}>
      <Plus size={12} /> Add first note
    </button>
  {/if}

  {#each notesStore.notes as note (note.slug)}
    {#if renamingSlug === note.slug}
      <div class="inline-input-wrapper">
        <input
          bind:this={renameInputEl}
          bind:value={renameValue}
          class="inline-input rename-input"
          type="text"
          placeholder="Note title..."
          onkeydown={handleRenameKeydown}
          onblur={confirmRename}
        />
      </div>
    {:else}
      <div class="note-row">
        <BinderItem
          label={note.title}
          icon={FileText}
          isSelected={notesStore.activeNoteSlug === note.slug}
          isActive={notesStore.activeNoteSlug === note.slug}
          onclick={() => handleNoteClick(note.slug)}
          oncontextmenu={(e) => handleContextMenu(e, note.slug, note.title)}
          indent={1}
        />
        {#if note.color}
          <span
            class="color-dot"
            style:background-color={note.color}
          ></span>
        {/if}
      </div>
    {/if}
  {/each}

  {#if isCreating}
    <div class="inline-input-wrapper">
      <input
        bind:this={inputEl}
        bind:value={newNoteTitle}
        class="inline-input"
        type="text"
        placeholder="Note title..."
        onkeydown={handleInputKeydown}
        onblur={confirmCreate}
      />
    </div>
  {/if}
</BinderSection>

{#if contextMenu}
  <ContextMenu
    items={getContextMenuItems(contextMenu.slug, contextMenu.title)}
    x={contextMenu.x}
    y={contextMenu.y}
    onClose={closeContextMenu}
  />
{/if}

<ConfirmDialog
  isOpen={deleteTarget !== null}
  title="Delete Note"
  message={deleteTarget ? `Are you sure you want to delete "${deleteTarget.title}"? This action cannot be undone.` : ''}
  confirmLabel="Delete"
  destructive={true}
  onConfirm={confirmDelete}
  onCancel={cancelDelete}
/>

<style>
  .placeholder-cta {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    padding-left: calc(var(--spacing-sm) + 16px + var(--spacing-xs));
    border: none;
    background: transparent;
    font-size: var(--font-size-xs);
    font-style: italic;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background-color var(--transition-fast),
      transform var(--transition-fast);
  }

  .placeholder-cta:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    transform: translateX(2px);
  }

  .placeholder-cta :global(svg) {
    opacity: 0.6;
    transition: opacity var(--transition-fast);
  }

  .placeholder-cta:hover :global(svg) {
    opacity: 1;
  }

  .note-row {
    position: relative;
    display: flex;
    align-items: center;
  }

  .note-row :global(button) {
    flex: 1;
    min-width: 0;
  }

  .color-dot {
    position: absolute;
    left: 14px;
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    pointer-events: none;
  }

  .inline-input-wrapper {
    padding: 2px var(--spacing-xs);
    padding-left: calc(8px + 1 * 16px);
  }

  .inline-input {
    width: 100%;
    padding: 3px var(--spacing-xs);
    border: 1px solid var(--border-primary, #555);
    border-radius: var(--radius-sm);
    background: var(--bg-primary, #1e1e1e);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    outline: none;
  }

  .inline-input:focus {
    border-color: var(--accent-primary, #7c4dbd);
  }

  .inline-input::placeholder {
    color: var(--text-tertiary);
  }

  .rename-input {
    border-color: var(--accent-primary, #7c4dbd);
    box-shadow: 0 0 0 1px var(--accent-primary, #7c4dbd);
  }
</style>
