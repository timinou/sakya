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

  /** Fallback: mark the first child of the editor root as active. */
  function highlightFirstBlock(): void {
    const rootEl = editor.getRootElement();
    if (rootEl?.firstElementChild) {
      const firstChild = rootEl.firstElementChild as HTMLElement;
      firstChild.classList.add('editor-focus-active');
      activeElements.add(firstChild);
    }
  }

  /** Handle selection changes: find the block-level ancestors and highlight them. */
  function updateFocusHighlight(): void {
    editor.getEditorState().read(() => {
      const selection = getSelection();

      clearActiveElements();

      if (!isRangeSelection(selection)) {
        // No selection — fall back to highlighting the first block
        highlightFirstBlock();
        return;
      }

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

          // Register the selection change listener (covers native selectionchange)
          const removeCommandListener = editor.registerCommand(
            SELECTION_CHANGE_COMMAND,
            () => {
              updateFocusHighlight();
              return false; // Don't stop command propagation
            },
            COMMAND_PRIORITY_LOW,
          );

          // Also listen to all editor state updates so the highlight follows
          // the cursor for arrow-key navigation and click-to-position moves
          // that may not fire SELECTION_CHANGE_COMMAND reliably.
          const removeUpdateListener = editor.registerUpdateListener(() => {
            updateFocusHighlight();
          });

          // Immediately highlight the current selection
          updateFocusHighlight();

          // Cleanup: runs when effect re-runs or component destroys
          return () => {
            removeCommandListener();
            removeUpdateListener();
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
