<script lang="ts">
  import { BookOpen, FileText, Plus, Pencil, Trash2, EllipsisVertical, FolderOutput, FolderInput } from 'lucide-svelte';
  import { notebookStore, notesStore, editorState, projectState, navigationStore } from '$lib/stores';
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

  // Load notebook on mount if not yet loaded
  $effect(() => {
    if (!notebookStore.isLoaded && !notebookStore.isLoading) {
      notebookStore.loadConfig().catch((e) => {
        console.error('Failed to load notebook config:', e);
      });
    }
  });

  async function confirmCreate(): Promise<void> {
    const title = newNoteTitle.trim();
    if (!title) {
      cancelCreate();
      return;
    }
    try {
      await notebookStore.createNote(title);
    } catch (e) {
      console.error('Failed to create notebook note:', e);
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
    navigationStore.navigateTo({ type: 'notebook-note', slug });
    onSelectNote?.(slug);
  }

  // Context menu handlers
  function handleContextMenu(e: MouseEvent, slug: string, title: string): void {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, slug, title };
  }

  function handleMenuButtonClick(e: MouseEvent, slug: string, title: string): void {
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    contextMenu = { x: rect.right, y: rect.top, slug, title };
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
    if (!deleteTarget) return;
    const { slug } = deleteTarget;
    try {
      // Close open tab for this note
      const tabId = `notebook-note:${slug}`;
      editorState.closeTab(tabId);
      await notebookStore.deleteNote(slug);
    } catch (e) {
      console.error('Failed to delete notebook note:', e);
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
    if (!newTitle || !renamingSlug || newTitle === getNoteTitle(renamingSlug)) {
      cancelRename();
      return;
    }
    try {
      await notebookStore.renameNote(renamingSlug, newTitle);
    } catch (e) {
      console.error('Failed to rename notebook note:', e);
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
    return notebookStore.notes.find((n) => n.slug === slug)?.title ?? '';
  }

  // Copy/move to project handlers
  async function handleCopyToProject(slug: string): Promise<void> {
    closeContextMenu();
    const path = projectState.projectPath;
    if (!path) return;
    try {
      await notebookStore.copyToProject(slug, path);
      await notesStore.loadConfig(path);
    } catch (e) {
      console.error('Failed to copy notebook note to project:', e);
    }
  }

  async function handleMoveToProject(slug: string): Promise<void> {
    closeContextMenu();
    const path = projectState.projectPath;
    if (!path) return;
    try {
      // Close open tab for this note since it will be removed from notebook
      const tabId = `notebook-note:${slug}`;
      editorState.closeTab(tabId);
      await notebookStore.moveToProject(slug, path);
      await notesStore.loadConfig(path);
    } catch (e) {
      console.error('Failed to move notebook note to project:', e);
    }
  }

  // Context menu items for a note
  function getContextMenuItems(slug: string, title: string) {
    return [
      { label: 'Rename', icon: Pencil, onclick: () => startRename(slug, title) },
      { label: '', separator: true },
      {
        label: 'Copy to Project',
        icon: FolderOutput,
        onclick: () => handleCopyToProject(slug),
        disabled: !projectState.projectPath,
      },
      {
        label: 'Move to Project',
        icon: FolderInput,
        onclick: () => handleMoveToProject(slug),
        disabled: !projectState.projectPath,
      },
      { label: '', separator: true },
      { label: 'Delete', icon: Trash2, onclick: () => handleDeleteRequest(slug, title) },
    ];
  }
</script>

<BinderSection
  title="Notebook"
  icon={BookOpen}
  count={notebookStore.noteCount}
  bind:isOpen={isOpen}
  onAdd={handleAdd}
>
  {#if notebookStore.notes.length === 0 && !isCreating}
    <button class="placeholder-cta" type="button" onclick={handleAdd}>
      <Plus size={12} /> Add first note
    </button>
  {/if}

  {#each notebookStore.notes as note (note.slug)}
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
          isSelected={notebookStore.activeNoteSlug === note.slug}
          isActive={notebookStore.activeNoteSlug === note.slug}
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
        <button
          class="item-action-btn"
          type="button"
          title="More actions"
          onclick={(e) => handleMenuButtonClick(e, note.slug, note.title)}
        >
          <EllipsisVertical size={14} />
        </button>
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
  title="Delete Notebook Note"
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

  .item-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    opacity: 0;
    transition:
      opacity var(--transition-fast),
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .note-row:hover .item-action-btn,
  .note-row:focus-within .item-action-btn {
    opacity: 1;
  }

  .item-action-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
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
