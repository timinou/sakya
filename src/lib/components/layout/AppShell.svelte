<script lang="ts">
  import type { Snippet } from 'svelte';
  import { uiState } from '$lib/stores';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import PaneResizer from './PaneResizer.svelte';

  interface Props {
    binderContent?: Snippet;
    editorContent?: Snippet;
    inspectorContent?: Snippet;
  }

  let { binderContent, editorContent, inspectorContent }: Props = $props();

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

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
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
      {/if}
    </div>
    <PaneResizer onResize={handleBinderResize} />
  {/if}

  <main class="pane editor-pane">
    {#if editorContent}
      {@render editorContent()}
    {/if}
  </main>

  {#if uiState.panes.inspectorVisible}
    <PaneResizer onResize={handleInspectorResize} />
    <div class="pane inspector-pane">
      {#if inspectorContent}
        {@render inspectorContent()}
      {/if}
    </div>
  {/if}

  <StatusBar />
</div>

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
