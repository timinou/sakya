<script lang="ts">
  import type { Snippet } from 'svelte';
  import { uiState, editorState, manuscriptStore, notesStore } from '$lib/stores';
  import { SearchPalette } from '$lib/components/common';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import PaneResizer from './PaneResizer.svelte';
  import EditorArea from './EditorArea.svelte';
  import Binder from './Binder.svelte';
  import Inspector from './Inspector.svelte';

  interface Props {
    binderContent?: Snippet;
    editorContent?: Snippet;
    inspectorContent?: Snippet;
  }

  let { binderContent, editorContent, inspectorContent }: Props = $props();

  // Search palette state
  let searchOpen = $state(false);

  let binderCol = $derived(
    uiState.panes.binderVisible ? `${uiState.panes.binderWidth}px` : '0px'
  );
  let inspectorCol = $derived(
    uiState.panes.inspectorVisible ? `${uiState.panes.inspectorWidth}px` : '0px'
  );
  let binderResizerCol = $derived(uiState.panes.binderVisible ? '4px' : '0px');
  let inspectorResizerCol = $derived(uiState.panes.inspectorVisible ? '4px' : '0px');

  let gridTemplateColumns = $derived(
    `${binderCol} ${binderResizerCol} 1fr ${inspectorResizerCol} ${inspectorCol}`
  );

  function handleBinderResize(delta: number) {
    uiState.setBinderWidth(uiState.panes.binderWidth + delta);
  }

  function handleInspectorResize(delta: number) {
    uiState.setInspectorWidth(uiState.panes.inspectorWidth - delta);
  }

  function handleSearchNavigate(fileType: string, slug: string, entityType?: string) {
    switch (fileType) {
      case 'chapter':
        manuscriptStore.selectChapter(slug);
        break;
      case 'entity':
        // Open entity by selecting it via the entity store flow
        // The entity type is the schema type for entity navigation
        if (entityType) {
          editorState.openDocument({
            id: `entity:${entityType}:${slug}`,
            title: slug,
            documentType: 'entity',
            documentSlug: slug,
            isDirty: false,
          });
        }
        break;
      case 'note':
        notesStore.selectNote(slug);
        break;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    // Cmd+K: Toggle search palette
    if (mod && e.key === 'k') {
      e.preventDefault();
      searchOpen = !searchOpen;
      return;
    }

    // Cmd+W: Close active tab
    if (mod && e.key === 'w') {
      if (editorState.activeTabId) {
        e.preventDefault();
        editorState.closeTab(editorState.activeTabId);
      }
      return;
    }

    // Cmd+S: Trigger immediate save
    if (mod && e.key === 's') {
      if (editorState.activeTab) {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent('sakya:save'));
      }
      // If no active tab, don't prevent default — let browser/Tauri handle
      return;
    }

    // Cmd+\ / Cmd+Shift+\: Toggle binder / inspector
    if (mod && e.key === '\\') {
      e.preventDefault();
      if (e.shiftKey) {
        uiState.toggleInspector();
      } else {
        uiState.toggleBinder();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="app-shell"
  style:grid-template-columns={gridTemplateColumns}
  data-theme={uiState.effectiveTheme}
>
  <Toolbar />

  {#if uiState.panes.binderVisible}
    <div class="pane binder-pane">
      {#if binderContent}
        {@render binderContent()}
      {:else}
        <Binder />
      {/if}
    </div>
    <PaneResizer onResize={handleBinderResize} />
  {/if}

  <main class="pane editor-pane">
    {#if editorContent}
      {@render editorContent()}
    {:else}
      <EditorArea />
    {/if}
  </main>

  {#if uiState.panes.inspectorVisible}
    <PaneResizer onResize={handleInspectorResize} />
    <div class="pane inspector-pane">
      {#if inspectorContent}
        {@render inspectorContent()}
      {:else}
        <Inspector />
      {/if}
    </div>
  {/if}

  <StatusBar />
</div>

<SearchPalette
  isOpen={searchOpen}
  onClose={() => { searchOpen = false; }}
  onNavigate={handleSearchNavigate}
/>

<style>
  .app-shell {
    display: grid;
    grid-template-rows: var(--toolbar-height) 1fr var(--statusbar-height);
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .pane {
    overflow: hidden;
    min-width: 0;
    min-height: 0;
  }

  .binder-pane {
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-secondary);
  }

  .editor-pane {
    background: var(--bg-primary);
  }

  .inspector-pane {
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-secondary);
  }
</style>
