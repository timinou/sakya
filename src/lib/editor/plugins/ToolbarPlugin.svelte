<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import {
    $getSelection as getSelection,
    $isRangeSelection as isRangeSelection,
    FORMAT_TEXT_COMMAND,
  } from 'lexical';
  import { $isHeadingNode as isHeadingNode } from '@lexical/rich-text';
  import { $isListNode as isListNode } from '@lexical/list';
  import {
    Bold,
    Italic,
    Strikethrough,
    Code,
    Heading1,
    Heading2,
    Heading3,
    List,
    ListOrdered,
    Quote,
    Undo2,
    Redo2,
  } from 'lucide-svelte';
  import {
    toggleBold,
    toggleItalic,
    toggleStrikethrough,
    formatHeading,
    formatBulletList,
    formatNumberedList,
    formatQuote,
    formatCode,
    formatParagraph,
    undo,
    redo,
  } from 'svelte-lexical';

  type BlockType =
    | 'paragraph'
    | 'h1'
    | 'h2'
    | 'h3'
    | 'bullet'
    | 'number'
    | 'quote'
    | 'code';

  let isBold = $state(false);
  let isItalic = $state(false);
  let isStrikethrough = $state(false);
  let isCode = $state(false);
  let blockType: BlockType = $state('paragraph');

  const editor = getEditor();

  onMount(() => {
    const removeListener = editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const selection = getSelection();
        if (!isRangeSelection(selection)) return;

        isBold = selection.hasFormat('bold');
        isItalic = selection.hasFormat('italic');
        isStrikethrough = selection.hasFormat('strikethrough');
        isCode = selection.hasFormat('code');

        const anchorNode = selection.anchor.getNode();
        const topLevelElement =
          anchorNode.getKey() === 'root'
            ? anchorNode
            : anchorNode.getTopLevelElementOrThrow();

        if (isHeadingNode(topLevelElement)) {
          blockType = topLevelElement.getTag() as BlockType;
        } else if (isListNode(topLevelElement)) {
          const listType = topLevelElement.getListType();
          blockType = listType === 'bullet' ? 'bullet' : 'number';
        } else {
          const type = topLevelElement.getType();
          if (type === 'quote') {
            blockType = 'quote';
          } else if (type === 'code') {
            blockType = 'code';
          } else {
            blockType = 'paragraph';
          }
        }
      });
    });

    return () => removeListener();
  });

  function toggleBlockType(type: BlockType) {
    if (blockType === type) {
      formatParagraph(editor);
    } else {
      switch (type) {
        case 'h1':
          formatHeading(editor, blockType, 'h1');
          break;
        case 'h2':
          formatHeading(editor, blockType, 'h2');
          break;
        case 'h3':
          formatHeading(editor, blockType, 'h3');
          break;
        case 'bullet':
          formatBulletList(editor, blockType);
          break;
        case 'number':
          formatNumberedList(editor, blockType);
          break;
        case 'quote':
          formatQuote(editor, blockType);
          break;
        case 'code':
          formatCode(editor, blockType);
          break;
      }
    }
  }
</script>

<div class="editor-toolbar" role="toolbar" aria-label="Text formatting">
  <div class="toolbar-group">
    <button
      class="toolbar-btn"
      class:active={isBold}
      onclick={() => toggleBold(editor)}
      title="Bold (Ctrl+B)"
      aria-pressed={isBold}
    >
      <Bold size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isItalic}
      onclick={() => toggleItalic(editor)}
      title="Italic (Ctrl+I)"
      aria-pressed={isItalic}
    >
      <Italic size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isStrikethrough}
      onclick={() => toggleStrikethrough(editor)}
      title="Strikethrough"
      aria-pressed={isStrikethrough}
    >
      <Strikethrough size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isCode}
      onclick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'code')}
      title="Inline Code"
      aria-pressed={isCode}
    >
      <Code size={16} />
    </button>
  </div>

  <span class="toolbar-divider"></span>

  <div class="toolbar-group">
    <button
      class="toolbar-btn"
      class:active={blockType === 'h1'}
      onclick={() => toggleBlockType('h1')}
      title="Heading 1"
      aria-pressed={blockType === 'h1'}
    >
      <Heading1 size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={blockType === 'h2'}
      onclick={() => toggleBlockType('h2')}
      title="Heading 2"
      aria-pressed={blockType === 'h2'}
    >
      <Heading2 size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={blockType === 'h3'}
      onclick={() => toggleBlockType('h3')}
      title="Heading 3"
      aria-pressed={blockType === 'h3'}
    >
      <Heading3 size={16} />
    </button>
  </div>

  <span class="toolbar-divider"></span>

  <div class="toolbar-group">
    <button
      class="toolbar-btn"
      class:active={blockType === 'bullet'}
      onclick={() => toggleBlockType('bullet')}
      title="Bullet List"
      aria-pressed={blockType === 'bullet'}
    >
      <List size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={blockType === 'number'}
      onclick={() => toggleBlockType('number')}
      title="Numbered List"
      aria-pressed={blockType === 'number'}
    >
      <ListOrdered size={16} />
    </button>
    <button
      class="toolbar-btn"
      class:active={blockType === 'quote'}
      onclick={() => toggleBlockType('quote')}
      title="Blockquote"
      aria-pressed={blockType === 'quote'}
    >
      <Quote size={16} />
    </button>
  </div>

  <span class="toolbar-divider"></span>

  <div class="toolbar-group">
    <button
      class="toolbar-btn"
      onclick={() => undo(editor)}
      title="Undo (Ctrl+Z)"
    >
      <Undo2 size={16} />
    </button>
    <button
      class="toolbar-btn"
      onclick={() => redo(editor)}
      title="Redo (Ctrl+Shift+Z)"
    >
      <Redo2 size={16} />
    </button>
  </div>
</div>

<style>
  .editor-toolbar {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    background: var(--bg-secondary);
    user-select: none;
    flex-wrap: wrap;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--border-secondary);
    margin: 0 var(--spacing-xs);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
    box-shadow: none;
  }

  .toolbar-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: transparent;
    box-shadow: none;
  }

  .toolbar-btn.active {
    background: var(--bg-tertiary);
    color: var(--accent-primary);
  }
</style>
