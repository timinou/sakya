<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import { $getRoot as getRoot } from 'lexical';

  interface Props {
    onCountChange?: (counts: {
      words: number;
      characters: number;
      charactersNoSpaces: number;
    }) => void;
  }

  let { onCountChange }: Props = $props();

  const editor = getEditor();

  onMount(() => {
    const removeListener = editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const root = getRoot();
        const text = root.getTextContent();
        const words = text.split(/\s+/).filter(Boolean).length;
        const characters = text.length;
        const charactersNoSpaces = text.replace(/\s/g, '').length;
        onCountChange?.({ words, characters, charactersNoSpaces });
      });
    });

    return () => removeListener();
  });
</script>
