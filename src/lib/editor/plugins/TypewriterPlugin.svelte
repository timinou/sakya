<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import {
    $getSelection as getSelection,
    $isRangeSelection as isRangeSelection,
  } from 'lexical';
  import { uiState } from '$lib/stores/ui.svelte';

  const editor = getEditor();

  let cleanupFn: (() => void) | null = null;
  let rafId: number | null = null;

  function getScrollContainer(): Element | null {
    const rootElement = editor.getRootElement();
    if (!rootElement) return null;
    return rootElement.closest('.editor-scroll');
  }

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

      const scrollContainer = getScrollContainer();
      if (!scrollContainer) return;

      // Cancel any pending scroll
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }

      rafId = requestAnimationFrame(() => {
        rafId = null;

        // Use absolute scroll position: element's offset from scroll container top
        const elementOffsetTop = anchorElement.offsetTop;
        const containerHeight = scrollContainer.clientHeight;
        const elementHeight = anchorElement.offsetHeight;

        const targetScroll = elementOffsetTop - containerHeight / 2 + elementHeight / 2;

        scrollContainer.scrollTo({
          top: targetScroll,
          behavior: 'smooth',
        });
      });
    });
  }

  function enable(): void {
    // Add padding class for scroll space above/below content
    const scrollContainer = getScrollContainer();
    if (scrollContainer) {
      scrollContainer.classList.add('typewriter-active');
    }

    // Use registerUpdateListener for complete event coverage
    // (typing, paste, delete, undo/redo, formatting, etc.)
    cleanupFn = editor.registerUpdateListener(() => {
      scrollCaretToCenter();
    });

    // Immediate scroll to center on enable
    scrollCaretToCenter();
  }

  function disable(): void {
    if (cleanupFn) {
      cleanupFn();
      cleanupFn = null;
    }
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }

    // Remove padding class
    const scrollContainer = getScrollContainer();
    if (scrollContainer) {
      scrollContainer.classList.remove('typewriter-active');
    }
  }

  onMount(() => {
    // Use $effect to reactively register/deregister based on typewriterMode
    const stopEffect = $effect.root(() => {
      $effect(() => {
        if (uiState.typewriterMode) {
          enable();
        } else {
          disable();
        }
      });
    });

    return () => {
      stopEffect();
      disable();
    };
  });
</script>
