<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Chapter, ChapterStatus } from '$lib/types';
  import { uiState, editorState, manuscriptStore, notesStore, projectState } from '$lib/stores';
  import { SearchPalette } from '$lib/components/common';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import PaneResizer from './PaneResizer.svelte';
  import EditorArea from './EditorArea.svelte';
  import Binder from './Binder.svelte';
  import Inspector from './Inspector.svelte';
  import ChapterInspector from '$lib/components/inspector/ChapterInspector.svelte';
  import Corkboard from '$lib/components/notes/Corkboard.svelte';

  interface Props {
    binderContent?: Snippet;
    editorContent?: Snippet;
    inspectorContent?: Snippet;
  }

  let { binderContent, editorContent, inspectorContent }: Props = $props();

  // Search palette state
  let searchOpen = $state(false);

  // --- Distraction-free mode state ---
  let peekBinder = $state(false);
  let peekInspector = $state(false);
  let peekBinderTimer: ReturnType<typeof setTimeout> | null = null;
  let peekInspectorTimer: ReturnType<typeof setTimeout> | null = null;

  const EDGE_THRESHOLD = 20; // pixels from screen edge to trigger peek
  const PEEK_HIDE_DELAY = 300; // ms delay before hiding peeked chrome

  // Determine whether binder/inspector should be shown in the DOM
  // In normal mode: only when visible. In distraction-free: always (for peek transitions).
  let showBinderInDom = $derived(
    uiState.panes.binderVisible || uiState.distractionFreeMode
  );
  let showInspectorInDom = $derived(
    uiState.panes.inspectorVisible || uiState.distractionFreeMode
  );

  let binderCol = $derived(
    uiState.distractionFreeMode
      ? '0px'
      : uiState.panes.binderVisible ? `${uiState.panes.binderWidth}px` : '0px'
  );
  let inspectorCol = $derived(
    uiState.distractionFreeMode
      ? '0px'
      : uiState.panes.inspectorVisible ? `${uiState.panes.inspectorWidth}px` : '0px'
  );
  let binderResizerCol = $derived(
    uiState.distractionFreeMode
      ? '0px'
      : uiState.panes.binderVisible ? '4px' : '0px'
  );
  let inspectorResizerCol = $derived(
    uiState.distractionFreeMode
      ? '0px'
      : uiState.panes.inspectorVisible ? '4px' : '0px'
  );

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

  // --- Chapter Inspector Handlers ---
  let metadataSaveTimer: ReturnType<typeof setTimeout> | null = null;

  function handleStatusChange(status: ChapterStatus) {
    const path = projectState.projectPath;
    const chapter = manuscriptStore.activeChapter;
    if (!path || !chapter) return;
    manuscriptStore.updateChapterMetadata(path, chapter.slug, { status });
  }

  function handleMetadataChange(updates: Partial<Chapter>) {
    const path = projectState.projectPath;
    const chapter = manuscriptStore.activeChapter;
    if (!path || !chapter) return;

    // Debounce metadata saves (1s)
    if (metadataSaveTimer) clearTimeout(metadataSaveTimer);
    metadataSaveTimer = setTimeout(() => {
      manuscriptStore.updateChapterMetadata(path, chapter.slug, updates);
      metadataSaveTimer = null;
    }, 1000);
  }

  // --- UI State Persistence ---
  let persistTimer: ReturnType<typeof setTimeout> | null = null;
  let hasRestored = $state(false);

  // Restore UI state when project opens
  $effect(() => {
    const path = projectState.projectPath;
    if (path && !hasRestored) {
      uiState.restore(path).then(() => {
        hasRestored = true;
      });
    }
    if (!path) {
      hasRestored = false;
    }
  });

  // Persist UI state on changes (debounced 1s)
  $effect(() => {
    const path = projectState.projectPath;
    if (!path || !hasRestored) return;

    // Read reactive state to track changes
    const _theme = uiState.theme;
    const _viewMode = uiState.viewMode;
    const _binderWidth = uiState.panes.binderWidth;
    const _inspectorWidth = uiState.panes.inspectorWidth;
    const _binderVisible = uiState.panes.binderVisible;
    const _inspectorVisible = uiState.panes.inspectorVisible;
    const _distractionFree = uiState.distractionFreeMode;
    const _typewriter = uiState.typewriterMode;
    const _focus = uiState.focusMode;

    if (persistTimer) clearTimeout(persistTimer);
    persistTimer = setTimeout(() => {
      uiState.persist(path);
    }, 1000);

    return () => {
      if (persistTimer) clearTimeout(persistTimer);
    };
  });

  // --- Distraction-free mouse edge detection ---
  function handleMouseMove(e: MouseEvent) {
    if (!uiState.distractionFreeMode) return;

    const x = e.clientX;
    const windowWidth = window.innerWidth;

    // Left edge: peek binder
    if (x <= EDGE_THRESHOLD) {
      if (peekBinderTimer) {
        clearTimeout(peekBinderTimer);
        peekBinderTimer = null;
      }
      peekBinder = true;
    } else if (peekBinder) {
      if (!peekBinderTimer) {
        peekBinderTimer = setTimeout(() => {
          peekBinder = false;
          peekBinderTimer = null;
        }, PEEK_HIDE_DELAY);
      }
    }

    // Right edge: peek inspector
    if (x >= windowWidth - EDGE_THRESHOLD) {
      if (peekInspectorTimer) {
        clearTimeout(peekInspectorTimer);
        peekInspectorTimer = null;
      }
      peekInspector = true;
    } else if (peekInspector) {
      if (!peekInspectorTimer) {
        peekInspectorTimer = setTimeout(() => {
          peekInspector = false;
          peekInspectorTimer = null;
        }, PEEK_HIDE_DELAY);
      }
    }
  }

  // Clean up peek state when distraction-free mode is toggled off
  $effect(() => {
    if (!uiState.distractionFreeMode) {
      peekBinder = false;
      peekInspector = false;
      if (peekBinderTimer) {
        clearTimeout(peekBinderTimer);
        peekBinderTimer = null;
      }
      if (peekInspectorTimer) {
        clearTimeout(peekInspectorTimer);
        peekInspectorTimer = null;
      }
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    // Escape: Exit distraction-free mode
    if (e.key === 'Escape' && uiState.distractionFreeMode) {
      e.preventDefault();
      uiState.distractionFreeMode = false;
      return;
    }

    // Cmd+K: Toggle search palette
    if (mod && e.key === 'k') {
      e.preventDefault();
      searchOpen = !searchOpen;
      return;
    }

    // Cmd+W: Close active tab
    if (mod && e.key === 'w') {
      const tab = editorState.activeTab;
      if (tab) {
        e.preventDefault();
        editorState.closeTab(tab.id);
        // Clear store selection to prevent $effect from re-opening the tab
        if (tab.documentType === 'chapter') {
          manuscriptStore.selectChapter('');
        } else if (tab.documentType === 'note') {
          notesStore.selectNote('');
        }
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
      return;
    }

    // Ctrl+Shift+F: Toggle distraction-free mode
    if (mod && e.shiftKey && e.key === 'F') {
      e.preventDefault();
      uiState.toggleDistractionFreeMode();
      return;
    }

    // Ctrl+Shift+T: Toggle typewriter mode
    if (mod && e.shiftKey && e.key === 'T') {
      e.preventDefault();
      uiState.toggleTypewriterMode();
      return;
    }

    // Ctrl+Shift+.: Toggle focus mode
    if (mod && e.shiftKey && (e.key === '>' || e.code === 'Period')) {
      e.preventDefault();
      uiState.toggleFocusMode();
      return;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} onmousemove={handleMouseMove} />

<div
  class="app-shell"
  class:distraction-free={uiState.distractionFreeMode}
  class:peek-binder={peekBinder}
  class:peek-inspector={peekInspector}
  style:grid-template-columns={gridTemplateColumns}
  data-theme={uiState.effectiveTheme}
>
  <Toolbar />

  {#if showBinderInDom}
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
    {:else if uiState.viewMode === 'corkboard'}
      <Corkboard notes={notesStore.notes} />
    {:else if uiState.viewMode === 'split'}
      <div class="split-view">
        <div class="split-editor">
          <EditorArea />
        </div>
        <div class="split-corkboard">
          <Corkboard notes={notesStore.notes} />
        </div>
      </div>
    {:else}
      <EditorArea />
    {/if}
  </main>

  {#if showInspectorInDom}
    <PaneResizer onResize={handleInspectorResize} />
    <div class="pane inspector-pane">
      {#if inspectorContent}
        {@render inspectorContent()}
      {:else}
        <Inspector>
          {#if editorState.activeTab?.documentType === 'chapter' && manuscriptStore.activeChapter}
            <ChapterInspector
              chapter={manuscriptStore.activeChapter}
              wordCount={editorState.wordCount.words}
              onStatusChange={handleStatusChange}
              onMetadataChange={handleMetadataChange}
            />
          {:else if editorState.activeTab}
            <div class="inspector-placeholder">
              <p>No inspector available for this document type</p>
            </div>
          {:else}
            <div class="inspector-placeholder">
              <p>No document selected</p>
            </div>
          {/if}
        </Inspector>
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
    transition: grid-template-rows 200ms ease, grid-template-columns 200ms ease;
  }

  .pane {
    overflow: hidden;
    min-width: 0;
    min-height: 0;
  }

  .binder-pane {
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-secondary);
    transition: opacity 200ms ease, transform 200ms ease;
  }

  .editor-pane {
    background: var(--bg-primary);
    transition: max-width 200ms ease, margin 200ms ease;
  }

  .inspector-pane {
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-secondary);
    transition: opacity 200ms ease, transform 200ms ease;
  }

  .split-view {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .split-editor {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .split-corkboard {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    border-left: 1px solid var(--border-secondary);
  }

  .inspector-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: var(--spacing-md);
  }

  .inspector-placeholder p {
    font-size: var(--font-size-sm);
    color: var(--text-tertiary);
    font-style: italic;
  }

  /* ==========================================================================
     Distraction-free mode
     ========================================================================== */

  /* Hide toolbar — slide up */
  .app-shell.distraction-free {
    grid-template-rows: 0px 1fr 0px;
  }

  .app-shell.distraction-free :global(.toolbar) {
    opacity: 0;
    transform: translateY(-100%);
    pointer-events: none;
    transition: opacity 200ms ease, transform 200ms ease;
    overflow: hidden;
  }

  /* Hide status bar — slide down */
  .app-shell.distraction-free :global(.status-bar) {
    opacity: 0;
    transform: translateY(100%);
    pointer-events: none;
    transition: opacity 200ms ease, transform 200ms ease;
    overflow: hidden;
  }

  /* Hide binder — fade out */
  .app-shell.distraction-free .binder-pane {
    opacity: 0;
    pointer-events: none;
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 260px;
    z-index: 100;
    border-right: 1px solid var(--border-secondary);
    transform: translateX(-100%);
  }

  /* Hide inspector — fade out */
  .app-shell.distraction-free .inspector-pane {
    opacity: 0;
    pointer-events: none;
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 300px;
    z-index: 100;
    border-left: 1px solid var(--border-secondary);
    transform: translateX(100%);
  }

  /* Hide resizers in distraction-free mode */
  .app-shell.distraction-free :global(.pane-resizer) {
    opacity: 0;
    pointer-events: none;
    width: 0;
    overflow: hidden;
  }

  /* Editor area: expand and center content for comfortable reading */
  .app-shell.distraction-free .editor-pane {
    max-width: 720px;
    margin: 0 auto;
    grid-column: 1 / -1;
  }

  /* --- Peek binder (left edge hover) --- */
  .app-shell.distraction-free.peek-binder .binder-pane {
    opacity: 1;
    pointer-events: auto;
    transform: translateX(0);
    box-shadow: 4px 0 16px rgba(0, 0, 0, 0.15);
  }

  /* --- Peek inspector (right edge hover) --- */
  .app-shell.distraction-free.peek-inspector .inspector-pane {
    opacity: 1;
    pointer-events: auto;
    transform: translateX(0);
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.15);
  }
</style>
