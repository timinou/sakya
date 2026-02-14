<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import {
    SELECTION_CHANGE_COMMAND,
    KEY_ENTER_COMMAND,
    COMMAND_PRIORITY_LOW,
    $getSelection as getSelection,
    $isRangeSelection as isRangeSelection,
  } from 'lexical';
  import { uiState } from '$lib/stores/ui.svelte';

  const editor = getEditor();

  let cleanupFns: (() => void)[] = [];
  let rafId: number | null = null;

  function scrollCaretToCenter(): void {
    const rootElement = editor.getRootElement();
    if (!rootElement) return;

    // Don't scroll when the editor is not focused
    if (!rootElement.ownerDocument.activeElement?.closest('[contenteditable]')) return;

    editor.getEditorState().read(() => {
      const selection = getSelection();
      if (!isRangeSelection(selection)) return;

      const anchorKey = selection.anchor.key;
      const anchorElement = editor.getElementByKey(anchorKey);
      if (!anchorElement) return;

      const scrollContainer = rootElement.closest('.editor-scroll');
      if (!scrollContainer) return;

      // Cancel any pending scroll
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }

      rafId = requestAnimationFrame(() => {
        rafId = null;

        const containerRect = scrollContainer.getBoundingClientRect();
        const elementRect = anchorElement.getBoundingClientRect();

        // Calculate where the element center is relative to the container
        const elementCenter = elementRect.top + elementRect.height / 2;
        const containerCenter = containerRect.top + containerRect.height / 2;
        const offset = elementCenter - containerCenter;

        // Only scroll if the element is not already near the center
        if (Math.abs(offset) > 10) {
          scrollContainer.scrollBy({
            top: offset,
            behavior: 'smooth',
          });
        }
      });
    });
  }

  function registerListeners(): void {
    cleanupFns.push(
      editor.registerCommand(
        SELECTION_CHANGE_COMMAND,
        () => {
          scrollCaretToCenter();
          return false; // Don't prevent other handlers
        },
        COMMAND_PRIORITY_LOW,
      ),
    );

    cleanupFns.push(
      editor.registerCommand(
        KEY_ENTER_COMMAND,
        () => {
          // Delay slightly so the new line is rendered before we scroll
          requestAnimationFrame(() => scrollCaretToCenter());
          return false;
        },
        COMMAND_PRIORITY_LOW,
      ),
    );
  }

  function deregisterListeners(): void {
    for (const cleanup of cleanupFns) {
      cleanup();
    }
    cleanupFns = [];
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  }

  onMount(() => {
    // Use $effect to reactively register/deregister based on typewriterMode
    const stopEffect = $effect.root(() => {
      $effect(() => {
        if (uiState.typewriterMode) {
          registerListeners();
        } else {
          deregisterListeners();
        }
      });
    });

    return () => {
      stopEffect();
      deregisterListeners();
    };
  });
</script>
