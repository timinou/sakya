<script lang="ts">
  import { BookOpen, FileText } from 'lucide-svelte';
  import { manuscriptStore, projectState } from '$lib/stores';
  import type { ChapterStatus } from '$lib/types/manuscript';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';

  interface Props {
    onSelectChapter?: (slug: string) => void;
  }

  let { onSelectChapter }: Props = $props();

  let isOpen = $state(true);
  let isCreating = $state(false);
  let newChapterTitle = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

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
    <div class="placeholder-text">
      No chapters yet
    </div>
  {/if}

  {#each sortedChapters as chapter (chapter.slug)}
    <div class="chapter-row">
      <BinderItem
        label="{chapter.order + 1}. {chapter.title}"
        icon={FileText}
        isSelected={manuscriptStore.activeChapterSlug === chapter.slug}
        isActive={manuscriptStore.activeChapterSlug === chapter.slug}
        onclick={() => handleChapterClick(chapter.slug)}
        indent={1}
      />
      <span
        class="status-dot"
        style:background-color={statusColors[chapter.status]}
        title={chapter.status}
      ></span>
    </div>
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

<style>
  .placeholder-text {
    padding: var(--spacing-xs) var(--spacing-sm);
    padding-left: calc(var(--spacing-sm) + 16px + var(--spacing-xs));
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-style: italic;
  }

  .chapter-row {
    position: relative;
    display: flex;
    align-items: center;
  }

  .chapter-row :global(button) {
    flex: 1;
    min-width: 0;
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
