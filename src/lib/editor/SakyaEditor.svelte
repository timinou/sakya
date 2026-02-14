<script lang="ts">
  import {
    Composer,
    ContentEditable,
    RichTextPlugin,
    MarkdownShortcutPlugin,
    HistoryPlugin,
    ListPlugin,
    LinkPlugin,
  } from 'svelte-lexical';
  import { HeadingNode, QuoteNode } from '@lexical/rich-text';
  import { ListNode, ListItemNode } from '@lexical/list';
  import { CodeNode } from '@lexical/code';
  import { LinkNode, AutoLinkNode } from '@lexical/link';
  import { $convertFromMarkdownString as convertFromMarkdownString } from '@lexical/markdown';
  import { sakyaEditorTheme } from './theme';
  import { WikiLinkNode } from './nodes/WikiLinkNode';
  import { SAKYA_TRANSFORMERS } from './transformers';
  import ToolbarPlugin from './plugins/ToolbarPlugin.svelte';
  import AutoSavePlugin from './plugins/AutoSavePlugin.svelte';
  import WordCountPlugin from './plugins/WordCountPlugin.svelte';
  import '$lib/editor/editor.css';

  interface Props {
    content?: string;
    onSave?: (markdown: string) => void;
    onCountChange?: (counts: {
      words: number;
      characters: number;
      charactersNoSpaces: number;
    }) => void;
    readonly?: boolean;
  }

  let { content = '', onSave, onCountChange, readonly = false }: Props = $props();

  const initialConfig = {
    namespace: 'SakyaEditor',
    theme: sakyaEditorTheme,
    nodes: [
      HeadingNode,
      QuoteNode,
      ListNode,
      ListItemNode,
      CodeNode,
      LinkNode,
      AutoLinkNode,
      WikiLinkNode,
    ],
    onError: (error: Error) => {
      console.error('[SakyaEditor]', error);
    },
    editorState: content
      ? () => {
          convertFromMarkdownString(content, SAKYA_TRANSFORMERS);
        }
      : undefined,
    editable: !readonly,
  };
</script>

<div class="sakya-editor" class:readonly>
  <Composer {initialConfig}>
    {#if !readonly}
      <ToolbarPlugin />
    {/if}
    <div class="editor-scroll">
      <RichTextPlugin />
      <ContentEditable className="editor-content" />
    </div>
    <HistoryPlugin />
    <ListPlugin />
    <LinkPlugin />
    <MarkdownShortcutPlugin transformers={SAKYA_TRANSFORMERS} />
    {#if !readonly && onSave}
      <AutoSavePlugin {onSave} />
    {/if}
    {#if onCountChange}
      <WordCountPlugin {onCountChange} />
    {/if}
  </Composer>
</div>

<style>
  .sakya-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .editor-scroll {
    flex: 1;
    overflow-y: auto;
  }

  .readonly {
    opacity: 0.8;
  }

  .readonly :global(.editor-content) {
    cursor: default;
  }
</style>
