<script lang="ts">
  import { BookOpen, FileText, Plus, Pencil, Trash2, ArrowUp, ArrowDown, EllipsisVertical } from 'lucide-svelte';
  import { manuscriptStore, editorState, projectState } from '$lib/stores';
  import type { ChapterStatus } from '$lib/types/manuscript';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';
  import ContextMenu from '$lib/components/common/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

  interface Props {
    onSelectChapter?: (slug: string) => void;
  }

  let { onSelectChapter }: Props = $props();

  let isOpen = $state(true);
  let isCreating = $state(false);
  let newChapterTitle = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  // Context menu state
  let contextMenu = $state<{ x: number; y: number; slug: string; title: string; status: ChapterStatus; index: number } | null>(null);

  // Delete confirmation state
  let deleteTarget = $state<{ slug: string; title: string } | null>(null);

  // Rename state
  let renamingSlug = $state<string | null>(null);
  let renameValue = $state('');
  let renameInputEl = $state<HTMLInputElement | null>(null);

  // Drag state
  let dragSlug = $state<string | null>(null);
  let dropIndex = $state<number | null>(null);

  const statusColors: Record<ChapterStatus, string> = {
    draft: 'var(--text-tertiary, #888)',
    revised: '#d4a017',
    final: '#22c55e',
  };

  let sortedChapters = $derived(
    [...manuscriptStore.chapters].sort((a, b) => a.order - b.order),
  );

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
    const title = newChapterTitle.trim();
    if (!title || !projectState.projectPath) {
      cancelCreate();
      return;
    }
    try {
      await manuscriptStore.createChapter(projectState.projectPath, title);
    } catch (e) {
      console.error('Failed to create chapter:', e);
    }
    cancelCreate();
  }

  function cancelCreate(): void {
    isCreating = false;
    newChapterTitle = '';
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

  function handleChapterClick(slug: string): void {
    manuscriptStore.selectChapter(slug);
    onSelectChapter?.(slug);
  }

  // Context menu handlers
  function handleContextMenu(e: MouseEvent, slug: string, title: string, status: ChapterStatus, index: number): void {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, slug, title, status, index };
  }

  function handleMenuButtonClick(e: MouseEvent, slug: string, title: string, status: ChapterStatus, index: number): void {
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    contextMenu = { x: rect.right, y: rect.top, slug, title, status, index };
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
      // Close open tab for this chapter
      const tabId = `chapter:${slug}`;
      editorState.closeTab(tabId);
      await manuscriptStore.deleteChapter(projectState.projectPath, slug);
    } catch (e) {
      console.error('Failed to delete chapter:', e);
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
    if (!newTitle || !renamingSlug || !projectState.projectPath || newTitle === getChapterTitle(renamingSlug)) {
      cancelRename();
      return;
    }
    try {
      await manuscriptStore.renameChapter(projectState.projectPath, renamingSlug, newTitle);
    } catch (e) {
      console.error('Failed to rename chapter:', e);
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

  function getChapterTitle(slug: string): string {
    return manuscriptStore.chapters.find(c => c.slug === slug)?.title ?? '';
  }

  // Status change handler
  async function handleStatusChange(slug: string, newStatus: ChapterStatus): Promise<void> {
    closeContextMenu();
    if (!projectState.projectPath) return;
    const chapter = manuscriptStore.chapters.find(c => c.slug === slug);
    if (!chapter) return;
    try {
      await manuscriptStore.saveChapterContent(
        projectState.projectPath,
        slug,
        { ...chapter, status: newStatus },
        manuscriptStore.chapterContent[slug]?.body ?? '',
      );
    } catch (e) {
      console.error('Failed to change chapter status:', e);
    }
  }

  // Move handlers (keyboard-accessible reorder)
  async function handleMoveUp(slug: string, index: number): Promise<void> {
    closeContextMenu();
    if (index <= 0 || !projectState.projectPath) return;
    const slugs = sortedChapters.map(c => c.slug);
    [slugs[index - 1], slugs[index]] = [slugs[index], slugs[index - 1]];
    try {
      await manuscriptStore.reorderChapters(projectState.projectPath, slugs);
    } catch (e) {
      console.error('Failed to reorder chapters:', e);
    }
  }

  async function handleMoveDown(slug: string, index: number): Promise<void> {
    closeContextMenu();
    if (index >= sortedChapters.length - 1 || !projectState.projectPath) return;
    const slugs = sortedChapters.map(c => c.slug);
    [slugs[index], slugs[index + 1]] = [slugs[index + 1], slugs[index]];
    try {
      await manuscriptStore.reorderChapters(projectState.projectPath, slugs);
    } catch (e) {
      console.error('Failed to reorder chapters:', e);
    }
  }

  // Drag-and-drop handlers
  function handleDragStart(e: DragEvent, slug: string): void {
    dragSlug = slug;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', slug);
    }
  }

  function handleDragOver(e: DragEvent, index: number): void {
    if (!dragSlug) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dropIndex = index;
  }

  function handleDragLeave(): void {
    dropIndex = null;
  }

  function handleDragEnd(): void {
    dragSlug = null;
    dropIndex = null;
  }

  async function handleDrop(e: DragEvent, targetIndex: number): Promise<void> {
    e.preventDefault();
    if (!dragSlug || !projectState.projectPath) {
      handleDragEnd();
      return;
    }
    const currentIndex = sortedChapters.findIndex(c => c.slug === dragSlug);
    if (currentIndex === -1 || currentIndex === targetIndex) {
      handleDragEnd();
      return;
    }
    const slugs = sortedChapters.map(c => c.slug);
    const [moved] = slugs.splice(currentIndex, 1);
    slugs.splice(targetIndex, 0, moved);
    try {
      await manuscriptStore.reorderChapters(projectState.projectPath, slugs);
    } catch (e) {
      console.error('Failed to reorder chapters:', e);
    }
    handleDragEnd();
  }

  // Context menu items for a chapter
  function getContextMenuItems(slug: string, title: string, status: ChapterStatus, index: number) {
    return [
      { label: 'Rename', icon: Pencil, onclick: () => startRename(slug, title) },
      { label: '', separator: true },
      { label: 'Status: Draft', onclick: () => handleStatusChange(slug, 'draft'), disabled: status === 'draft' },
      { label: 'Status: Revised', onclick: () => handleStatusChange(slug, 'revised'), disabled: status === 'revised' },
      { label: 'Status: Final', onclick: () => handleStatusChange(slug, 'final'), disabled: status === 'final' },
      { label: '', separator: true },
      { label: 'Move Up', icon: ArrowUp, onclick: () => handleMoveUp(slug, index), disabled: index === 0 },
      { label: 'Move Down', icon: ArrowDown, onclick: () => handleMoveDown(slug, index), disabled: index === sortedChapters.length - 1 },
      { label: '', separator: true },
      { label: 'Delete', icon: Trash2, onclick: () => handleDeleteRequest(slug, title) },
    ];
  }

  // Listen for sakya:create-chapter custom events from WelcomeCard
  $effect(() => {
    function handleCreateChapterEvent() {
      handleAdd();
    }
    window.addEventListener('sakya:create-chapter', handleCreateChapterEvent);
    return () => {
      window.removeEventListener('sakya:create-chapter', handleCreateChapterEvent);
    };
  });

  // Load chapters once when mounted (track path to avoid infinite loop with empty chapters)
  let loadedPath: string | null = null;

  $effect(() => {
    const path = projectState.projectPath;
    if (path && path !== loadedPath && !manuscriptStore.isLoading) {
      loadedPath = path;
      manuscriptStore.loadConfig(path).catch((e) => {
        console.error('Failed to load manuscript config:', e);
      });
    }
  });
</script>

<BinderSection
  title="Manuscript"
  icon={BookOpen}
  count={manuscriptStore.chapterCount}
  bind:isOpen={isOpen}
  onAdd={handleAdd}
>
  {#if sortedChapters.length === 0 && !isCreating}
    <button class="placeholder-cta" type="button" onclick={handleAdd}>
      <Plus size={12} /> Add first chapter
    </button>
  {/if}

  {#each sortedChapters as chapter, idx (chapter.slug)}
    {#if renamingSlug === chapter.slug}
      <div class="inline-input-wrapper">
        <input
          bind:this={renameInputEl}
          bind:value={renameValue}
          class="inline-input rename-input"
          type="text"
          placeholder="Chapter title..."
          onkeydown={handleRenameKeydown}
          onblur={confirmRename}
        />
      </div>
    {:else}
      <div
        class="chapter-row"
        class:drag-over={dropIndex === idx && dragSlug !== chapter.slug}
        class:dragging={dragSlug === chapter.slug}
        draggable="true"
        ondragstart={(e) => handleDragStart(e, chapter.slug)}
        ondragover={(e) => handleDragOver(e, idx)}
        ondragleave={handleDragLeave}
        ondragend={handleDragEnd}
        ondrop={(e) => handleDrop(e, idx)}
        role="listitem"
      >
        <BinderItem
          label="{chapter.order + 1}. {chapter.title}"
          icon={FileText}
          isSelected={manuscriptStore.activeChapterSlug === chapter.slug}
          isActive={manuscriptStore.activeChapterSlug === chapter.slug}
          onclick={() => handleChapterClick(chapter.slug)}
          oncontextmenu={(e) => handleContextMenu(e, chapter.slug, chapter.title, chapter.status, idx)}
          indent={1}
        />
        <span
          class="status-dot"
          style:background-color={statusColors[chapter.status]}
          title={chapter.status}
        ></span>
        <button
          class="item-action-btn"
          type="button"
          title="More actions"
          onclick={(e) => handleMenuButtonClick(e, chapter.slug, chapter.title, chapter.status, idx)}
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
        bind:value={newChapterTitle}
        class="inline-input"
        type="text"
        placeholder="Chapter title..."
        onkeydown={handleInputKeydown}
        onblur={confirmCreate}
      />
    </div>
  {/if}
</BinderSection>

{#if contextMenu}
  <ContextMenu
    items={getContextMenuItems(contextMenu.slug, contextMenu.title, contextMenu.status, contextMenu.index)}
    x={contextMenu.x}
    y={contextMenu.y}
    onClose={closeContextMenu}
  />
{/if}

<ConfirmDialog
  isOpen={deleteTarget !== null}
  title="Delete Chapter"
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

  .chapter-row {
    position: relative;
    display: flex;
    align-items: center;
    transition: opacity var(--transition-fast);
  }

  .chapter-row :global(button) {
    flex: 1;
    min-width: 0;
  }

  .chapter-row.dragging {
    opacity: 0.4;
  }

  .chapter-row.drag-over {
    border-top: 2px solid var(--accent-primary);
  }

  .status-dot {
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

  .chapter-row:hover .item-action-btn,
  .chapter-row:focus-within .item-action-btn {
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
