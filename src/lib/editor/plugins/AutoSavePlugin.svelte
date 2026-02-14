<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import { $convertToMarkdownString as convertToMarkdownString } from '@lexical/markdown';
  import { SAKYA_TRANSFORMERS } from '../transformers';

  interface Props {
    onSave?: (markdown: string) => void;
    debounceMs?: number;
  }

  let { onSave, debounceMs = 800 }: Props = $props();

  const editor = getEditor();

  onMount(() => {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    const removeListener = editor.registerUpdateListener(
      ({ editorState, dirtyElements, dirtyLeaves }) => {
        if (dirtyElements.size === 0 && dirtyLeaves.size === 0) return;

        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          editorState.read(() => {
            const markdown = convertToMarkdownString(SAKYA_TRANSFORMERS);
            onSave?.(markdown);
          });
        }, debounceMs);
      }
    );

    return () => {
      if (timeoutId) clearTimeout(timeoutId);
      removeListener();
    };
  });
</script>
