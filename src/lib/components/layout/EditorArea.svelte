<script lang="ts">
  import { manuscriptStore, editorState, projectState } from '$lib/stores';
  import SakyaEditor from '$lib/editor/SakyaEditor.svelte';
  import EditorTabs from './EditorTabs.svelte';
  import WelcomeCard from './WelcomeCard.svelte';
  import type { ChapterContent } from '$lib/types';

  // Track loaded content per tab
  let contentCache = $state<Record<string, ChapterContent>>({});
  let isLoadingContent = $state(false);

  let activeContent = $derived(
    editorState.activeTab ? contentCache[editorState.activeTab.id] ?? null : null
  );

  // When activeChapterSlug changes, open a tab and load content
  $effect(() => {
    const slug = manuscriptStore.activeChapterSlug;
    const path = projectState.projectPath;
    if (!slug || !path) return;

    const tabId = `chapter:${slug}`;
    const chapter = manuscriptStore.activeChapter;
    if (!chapter) return;

    // Open tab (idempotent - if already open, just switches to it)
    editorState.openDocument({
      id: tabId,
      title: chapter.title,
      documentType: 'chapter',
      documentSlug: slug,
      isDirty: false,
    });

    // Load content if not cached
    if (!contentCache[tabId]) {
      loadContent(path, slug, tabId);
    }
  });

  async function loadContent(projectPath: string, slug: string, tabId: string) {
    isLoadingContent = true;
    try {
      const content = await manuscriptStore.loadChapterContent(projectPath, slug);
      contentCache[tabId] = content;
    } catch (e) {
      console.error('[EditorArea] Failed to load chapter content:', e);
    } finally {
      isLoadingContent = false;
    }
  }

  async function handleSave(markdown: string) {
    const tab = editorState.activeTab;
    const path = projectState.projectPath;
    if (!tab || !path || tab.documentType !== 'chapter') return;

    const slug = tab.documentSlug;
    const chapter = manuscriptStore.chapters.find((c) => c.slug === slug);
    if (!chapter) return;

    await manuscriptStore.saveChapterContent(path, slug, chapter, markdown);
    editorState.setDirty(tab.id, false);

    // Update cached content
    if (contentCache[tab.id]) {
      contentCache[tab.id] = { ...contentCache[tab.id], body: markdown };
    }
  }

  function handleCountChange(counts: {
    words: number;
    characters: number;
    charactersNoSpaces: number;
  }) {
    editorState.updateWordCount(counts);
  }

  function handleDirty() {
    if (editorState.activeTab) {
      editorState.setDirty(editorState.activeTab.id, true);
    }
  }
</script>

<div class="editor-area">
  <EditorTabs />

  {#if isLoadingContent}
    <div class="editor-loading">
      <span class="loading-spinner"></span>
      <span>Loading chapter...</span>
    </div>
  {:else if activeContent}
    {#key activeContent.slug}
      <div class="editor-container" oninput={handleDirty}>
        <SakyaEditor
          content={activeContent.body}
          onSave={handleSave}
          onCountChange={handleCountChange}
        />
      </div>
    {/key}
  {:else if !editorState.activeTab}
    <WelcomeCard
      onCreateChapter={() => {
        window.dispatchEvent(new CustomEvent('sakya:create-chapter'));
      }}
      onCreateNote={() => {
        window.dispatchEvent(new CustomEvent('sakya:create-note'));
      }}
      onCreateEntity={(entityType) => {
        window.dispatchEvent(new CustomEvent('sakya:create-entity', { detail: { entityType } }));
      }}
    />
  {/if}
</div>

<style>
  .editor-area {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .editor-loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-primary);
    border-top-color: var(--accent-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
