<script lang="ts">
  import { StickyNote, FileText } from 'lucide-svelte';
  import { notesStore, projectState } from '$lib/stores';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';

  interface Props {
    onSelectNote?: (slug: string) => void;
  }

  let { onSelectNote }: Props = $props();

  let isOpen = $state(true);
  let isCreating = $state(false);
  let newNoteTitle = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  function handleAdd(): void {
    isCreating = true;
  }

  $effect(() => {
    if (isCreating && inputEl) {
      inputEl.focus();
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
    <div class="placeholder-text">
      No notes yet
    </div>
  {/if}

  {#each notesStore.notes as note (note.slug)}
    <div class="note-row">
      <BinderItem
        label={note.title}
        icon={FileText}
        isSelected={notesStore.activeNoteSlug === note.slug}
        isActive={notesStore.activeNoteSlug === note.slug}
        onclick={() => handleNoteClick(note.slug)}
        indent={1}
      />
      {#if note.color}
        <span
          class="color-dot"
          style:background-color={note.color}
        ></span>
      {/if}
    </div>
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

<style>
  .placeholder-text {
    padding: var(--spacing-xs) var(--spacing-sm);
    padding-left: calc(var(--spacing-sm) + 16px + var(--spacing-xs));
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-style: italic;
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
</style>
