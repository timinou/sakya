<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import {
    SELECTION_CHANGE_COMMAND,
    COMMAND_PRIORITY_LOW,
    $getSelection as getSelection,
    $isRangeSelection as isRangeSelection,
  } from 'lexical';
  import { uiState } from '$lib/stores/ui.svelte';

  const editor = getEditor();

  /** Track the DOM elements currently marked as active so we can clear them. */
  let activeElements: Set<HTMLElement> = new Set();

  /** Remove the editor-focus-active class from all tracked elements. */
  function clearActiveElements(): void {
    for (const el of activeElements) {
      el.classList.remove('editor-focus-active');
    }
    activeElements.clear();
  }

  /** Add editor-focus-active class to the DOM element for a given node key. */
  function markElementActive(key: string): void {
    const domElement = editor.getElementByKey(key);
    if (domElement) {
      domElement.classList.add('editor-focus-active');
      activeElements.add(domElement);
    }
  }

  /** Handle selection changes: find the block-level ancestors and highlight them. */
  function updateFocusHighlight(): void {
    editor.getEditorState().read(() => {
      const selection = getSelection();
      if (!isRangeSelection(selection)) return;

      clearActiveElements();

      const anchorNode = selection.anchor.getNode();
      const focusNode = selection.focus.getNode();

      // Find top-level block ancestors for both anchor and focus
      const anchorTopLevel =
        anchorNode.getKey() === 'root'
          ? null
          : anchorNode.getTopLevelElementOrThrow();
      const focusTopLevel =
        focusNode.getKey() === 'root'
          ? null
          : focusNode.getTopLevelElementOrThrow();

      if (anchorTopLevel) {
        markElementActive(anchorTopLevel.getKey());
      }
      if (focusTopLevel && focusTopLevel !== anchorTopLevel) {
        markElementActive(focusTopLevel.getKey());
      }
    });
  }

  /** Remove all focus-mode classes from the editor. */
  function cleanupFocusMode(): void {
    const rootElement = editor.getRootElement();
    if (rootElement) {
      rootElement.classList.remove('editor-focus-enabled');
    }
    clearActiveElements();
  }

  onMount(() => {
    // Use $effect.root to create a reactive scope inside onMount
    const stopEffect = $effect.root(() => {
      $effect(() => {
        const enabled = uiState.focusMode;
        const rootElement = editor.getRootElement();

        if (enabled) {
          if (rootElement) {
            rootElement.classList.add('editor-focus-enabled');
          }

          // Register the selection change listener
          const removeCommandListener = editor.registerCommand(
            SELECTION_CHANGE_COMMAND,
            () => {
              updateFocusHighlight();
              return false; // Don't stop command propagation
            },
            COMMAND_PRIORITY_LOW,
          );

          // Immediately highlight the current selection
          updateFocusHighlight();

          // Cleanup: runs when effect re-runs or component destroys
          return () => {
            removeCommandListener();
            cleanupFocusMode();
          };
        } else {
          cleanupFocusMode();
        }
      });
    });

    // Component destroy cleanup
    return () => {
      stopEffect();
      cleanupFocusMode();
    };
  });
</script>

<style>
  :global(.editor-focus-enabled) > :global(*) {
    opacity: 0.3;
    transition: opacity 150ms ease;
  }
  :global(.editor-focus-enabled) > :global(.editor-focus-active) {
    opacity: 1;
  }
</style>
