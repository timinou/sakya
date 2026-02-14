# Lexical Editor: Architecture and Integration Research

## Overview

Lexical is an extensible text editor framework created by Meta (Facebook). It was
designed as the successor to Draft.js, addressing Draft.js's performance issues,
limited extensibility, and React-only coupling. Lexical powers the text editing
experience across Meta's products and is open-source under the MIT license.

For Sakya, Lexical is the primary candidate for the editor core. Its architecture ---
immutable editor states, a node-based document model, a command/listener system, and
framework agnosticism --- aligns well with Sakya's requirements for a WYSIWYG Markdown
editor with custom node types, plugin extensibility, and high performance.

This document covers Lexical's architecture, core concepts, Markdown handling, custom
node development, the svelte-lexical bindings, plugin patterns, writing-app-specific
considerations, Markdown round-trip fidelity, and performance characteristics.

---

## Lexical Architecture

### Design Principles

Lexical was built with several key principles:

1. **Framework agnostic**: The core (`lexical`) has zero framework dependencies. It
   works with vanilla JS, React, Svelte, Vue, or any other framework. Framework
   bindings are separate packages.
2. **Small core**: The `lexical` core package is approximately 22KB minified. All
   additional functionality (rich text, Markdown, lists, tables, etc.) is provided
   by separate packages.
3. **Immutable editor states**: The document model uses immutable `EditorState`
   objects. Changes are made by creating new states, not mutating existing ones.
   This enables reliable undo/redo, serialization, and collaboration.
4. **Predictable updates**: DOM updates happen through a reconciliation process
   (similar to React's virtual DOM). The developer works with the node tree, and
   Lexical handles DOM manipulation.
5. **Extensible node system**: The document model is defined by nodes. Developers
   can create custom node types to represent any content (wiki-links, entity
   references, embedded media, etc.).

### Core Components

```
+-------------------+
|   Editor Instance  |  (singleton per editor)
+-------------------+
         |
         v
+-------------------+
|    EditorState     |  (immutable snapshot)
|  +-------------+  |
|  |  Node Tree   |  |  (ParagraphNode, TextNode, etc.)
|  +-------------+  |
|  +-------------+  |
|  |  Selection   |  |  (RangeSelection, NodeSelection, etc.)
|  +-------------+  |
+-------------------+
         |
         v
+-------------------+
|  DOM Reconciler    |  (diffs node tree -> DOM mutations)
+-------------------+
         |
         v
+-------------------+
|   Actual DOM       |  (contenteditable element)
+-------------------+
```

### The 22KB Core

The `lexical` package contains:

- The `LexicalEditor` class (editor instance management)
- The `EditorState` class (immutable state container)
- Base node types: `RootNode`, `ElementNode`, `TextNode`, `LineBreakNode`,
  `TabNode`, `DecoratorNode`
- The selection system: `RangeSelection`, `NodeSelection`, `GridSelection`
- The command system (dispatch/register)
- The update/transform system
- The DOM reconciler
- Utility functions for node traversal and manipulation

Everything else --- rich text behavior, Markdown support, lists, tables, links, code
blocks, history (undo/redo), clipboard handling --- is provided by separate `@lexical/*`
packages.

### DOM Reconciler

Lexical uses a custom DOM reconciler (not React's). When an `EditorState` is committed,
Lexical diffs the new node tree against the previous one and applies the minimal set of
DOM mutations needed to bring the DOM into sync.

Key characteristics:

- **Batched updates**: Multiple node changes within a single `editor.update()` call
  are batched and reconciled in one pass.
- **Minimal DOM mutations**: Only the changed nodes are updated in the DOM.
- **Decorator nodes**: `DecoratorNode` instances can render arbitrary framework
  components (React components, Svelte components, etc.) within the editor. The
  reconciler manages these embeddings.
- **Controlled contenteditable**: The `contenteditable` DOM element is fully
  controlled by Lexical. Direct DOM manipulation by the developer is discouraged
  and can break the reconciler.

---

## Core Concepts in Detail

### Editor Instance

The editor instance (`LexicalEditor`) is the central coordination point. It is created
once per editor and manages:

- The current `EditorState`
- Registered commands and listeners
- Registered node types
- The DOM element binding
- The update queue

Creating an editor:

```javascript
import { createEditor } from 'lexical';

const config = {
  namespace: 'SakyaEditor',
  theme: {
    // CSS class mappings for node types
    paragraph: 'editor-paragraph',
    text: {
      bold: 'editor-bold',
      italic: 'editor-italic',
      underline: 'editor-underline',
    },
    heading: {
      h1: 'editor-h1',
      h2: 'editor-h2',
      h3: 'editor-h3',
    },
  },
  nodes: [
    // Register custom node types here
    HeadingNode,
    QuoteNode,
    ListNode,
    ListItemNode,
    CodeNode,
    LinkNode,
    // Custom nodes for Sakya
    WikiLinkNode,
    EntityReferenceNode,
  ],
  onError: (error) => {
    console.error('Lexical error:', error);
  },
};

const editor = createEditor(config);
```

### EditorState

An `EditorState` is an immutable snapshot of the editor's content and selection. It
contains:

1. **Node tree**: A hierarchy of nodes representing the document structure. The root
   is always a `RootNode`, which contains `ElementNode` children (paragraphs,
   headings, lists, etc.), which contain `TextNode` children and other inline nodes.

2. **Selection**: The current selection state. Can be:
   - `RangeSelection`: A text range (most common). Has anchor and focus points.
   - `NodeSelection`: One or more entire nodes selected (e.g., an image).
   - `GridSelection`: A range of cells in a table (deprecated in favor of
     `TableSelection`).
   - `null`: No selection (editor is not focused).

Reading the editor state:

```javascript
// Synchronous read (no mutations allowed)
editor.getEditorState().read(() => {
  const root = $getRoot();
  const text = root.getTextContent();
  console.log('Document text:', text);
});

// Or using the update callback (mutations allowed)
editor.update(() => {
  const root = $getRoot();
  const paragraph = root.getFirstChild();
  // Can read and write nodes here
});
```

### Commands

Commands are the primary communication mechanism in Lexical. Instead of directly
mutating the editor state, code dispatches commands, and registered handlers process
them.

Built-in commands include:

- `KEY_ENTER_COMMAND` (Enter key pressed)
- `KEY_BACKSPACE_COMMAND` (Backspace key pressed)
- `KEY_TAB_COMMAND` (Tab key pressed)
- `FORMAT_TEXT_COMMAND` (bold, italic, etc.)
- `INSERT_PARAGRAPH_COMMAND`
- `INSERT_LINE_BREAK_COMMAND`
- `PASTE_COMMAND`
- `COPY_COMMAND`
- `CUT_COMMAND`
- `UNDO_COMMAND`
- `REDO_COMMAND`
- `SELECTION_CHANGE_COMMAND`
- `CLICK_COMMAND`

Custom commands:

```javascript
import { createCommand } from 'lexical';

// Define a custom command
const INSERT_WIKI_LINK_COMMAND = createCommand('INSERT_WIKI_LINK');

// Register a handler
editor.registerCommand(
  INSERT_WIKI_LINK_COMMAND,
  (payload) => {
    // payload contains { target, displayText }
    const { target, displayText } = payload;
    editor.update(() => {
      const wikiLinkNode = $createWikiLinkNode(target, displayText);
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        selection.insertNodes([wikiLinkNode]);
      }
    });
    return true; // Command was handled
  },
  COMMAND_PRIORITY_NORMAL
);

// Dispatch the command from anywhere
editor.dispatchCommand(INSERT_WIKI_LINK_COMMAND, {
  target: 'character-elara',
  displayText: 'Elara',
});
```

### Command Priority System

When multiple handlers are registered for the same command, they are executed in
priority order. If a handler returns `true`, propagation stops. If it returns `false`,
the next handler is called.

Priority levels (from highest to lowest):

```javascript
import {
  COMMAND_PRIORITY_CRITICAL,  // 4 - Override everything
  COMMAND_PRIORITY_HIGH,      // 3 - Important plugins
  COMMAND_PRIORITY_NORMAL,    // 2 - Standard handlers
  COMMAND_PRIORITY_LOW,       // 1 - Default/fallback handlers
  COMMAND_PRIORITY_EDITOR,    // 0 - Built-in editor behavior
} from 'lexical';
```

This system allows plugins to intercept and override built-in behavior. For example,
a Markdown shortcuts plugin can intercept `KEY_ENTER_COMMAND` at `COMMAND_PRIORITY_HIGH`
to convert "# " at the start of a line into a heading, before the default handler
inserts a new paragraph.

### Transforms

Transforms are hooks that run before reconciliation. They observe changes to specific
node types and can modify the node tree in response.

```javascript
// Register a transform on TextNode
const removeTransform = editor.registerNodeTransform(TextNode, (textNode) => {
  // This runs every time a TextNode is created or modified
  const text = textNode.getTextContent();

  // Example: Auto-detect wiki-link syntax
  const wikiLinkRegex = /\[\[([^\]]+)\]\]/;
  const match = text.match(wikiLinkRegex);
  if (match) {
    // Split the text node and insert a WikiLinkNode
    const [before, linkText, after] = splitTextByMatch(textNode, match);
    const wikiLink = $createWikiLinkNode(linkText, linkText);
    textNode.replace(wikiLink);
    // Handle before/after text...
  }
});

// Later: clean up
removeTransform();
```

Transforms are powerful but must be used carefully:

- They run synchronously during the update cycle.
- They can trigger other transforms (Lexical handles infinite loop prevention).
- They should be fast to avoid blocking the reconciliation.

### Double-Buffering (Current + Pending States)

Lexical uses a double-buffering strategy for editor states:

1. **Current state**: The committed, immutable state that reflects the current DOM.
2. **Pending state**: A mutable clone of the current state where updates are applied.

When `editor.update()` is called:
1. The current state is cloned to create a pending state.
2. The callback executes, mutating the pending state's node tree.
3. Transforms run on the pending state.
4. The pending state is committed (becomes the new current state).
5. The DOM reconciler diffs the old and new states and updates the DOM.

This ensures that:
- Reads of the current state are always consistent.
- Updates are atomic (all changes in an `editor.update()` call apply together or
  not at all).
- The DOM is always in sync with the current state.

---

## Markdown Handling

### The `@lexical/markdown` Package

Lexical provides a dedicated package for Markdown interop. It supports:

1. **Converting editor state to Markdown string**
2. **Converting Markdown string to editor state**
3. **Registering Markdown shortcuts** (live typing shortcuts in the editor)

### Converting Editor State to Markdown

```javascript
import { $convertToMarkdownString, TRANSFORMERS } from '@lexical/markdown';

editor.update(() => {
  const markdown = $convertToMarkdownString(TRANSFORMERS);
  console.log(markdown);
  // Output:
  // # My Heading
  //
  // This is a paragraph with **bold** and *italic* text.
  //
  // - List item 1
  // - List item 2
});
```

The `TRANSFORMERS` constant is an array of all built-in transformers. You can provide
a custom subset or add custom transformers.

### Converting Markdown to Editor State

```javascript
import { $convertFromMarkdownString, TRANSFORMERS } from '@lexical/markdown';

editor.update(() => {
  $convertFromMarkdownString(markdownString, TRANSFORMERS);
});
```

This clears the current editor content and replaces it with the parsed Markdown.

### Registering Markdown Shortcuts

```javascript
import { registerMarkdownShortcuts, TRANSFORMERS } from '@lexical/markdown';

// Returns a cleanup function
const removeShortcuts = registerMarkdownShortcuts(editor, TRANSFORMERS);
```

Markdown shortcuts enable writers to type Markdown syntax and have it converted to
rich text in real-time:

- Type `# ` at the start of a line to create an H1
- Type `## ` for H2, `### ` for H3, etc.
- Type `> ` for a blockquote
- Type `- ` or `* ` for an unordered list
- Type `1. ` for an ordered list
- Type ``` ``` ``` for a code block
- Type `**text**` for bold
- Type `*text*` for italic
- Type `` `text` `` for inline code
- Type `~~text~~` for strikethrough
- Type `---` for a horizontal rule

### Transformer Types

Transformers define how nodes map to/from Markdown syntax. There are three types:

#### Element Transformers

Map block-level nodes (headings, lists, quotes, code blocks) to Markdown:

```javascript
const HEADING_TRANSFORMER = {
  dependencies: [HeadingNode],
  export: (node, exportChildren) => {
    if (!$isHeadingNode(node)) return null;
    const level = Number(node.getTag().slice(1)); // 'h1' -> 1
    return '#'.repeat(level) + ' ' + exportChildren(node);
  },
  regExp: /^(#{1,6})\s/,
  replace: (parentNode, children, match) => {
    const tag = 'h' + match[1].length;
    const headingNode = $createHeadingNode(tag);
    headingNode.append(...children);
    parentNode.replace(headingNode);
  },
  type: 'element',
};
```

#### Text Format Transformers

Map inline formatting (bold, italic, strikethrough, inline code) to Markdown:

```javascript
const BOLD_TRANSFORMER = {
  format: ['bold'],
  tag: '**',
  type: 'text-format',
};

const ITALIC_TRANSFORMER = {
  format: ['italic'],
  tag: '*',
  type: 'text-format',
};

const STRIKETHROUGH_TRANSFORMER = {
  format: ['strikethrough'],
  tag: '~~',
  type: 'text-format',
};

const INLINE_CODE_TRANSFORMER = {
  format: ['code'],
  tag: '`',
  type: 'text-format',
};
```

#### Text Match Transformers

Map inline nodes that require pattern matching (links, images) to Markdown:

```javascript
const LINK_TRANSFORMER = {
  dependencies: [LinkNode],
  export: (node) => {
    if (!$isLinkNode(node)) return null;
    const text = node.getTextContent();
    const url = node.getURL();
    return `[${text}](${url})`;
  },
  importRegExp: /\[([^\]]+)\]\(([^)]+)\)/,
  regExp: /\[([^\]]+)\]\(([^)]+)\)$/,
  replace: (textNode, match) => {
    const [, text, url] = match;
    const linkNode = $createLinkNode(url);
    const linkTextNode = $createTextNode(text);
    linkNode.append(linkTextNode);
    textNode.replace(linkNode);
  },
  trigger: ')',
  type: 'text-match',
};
```

### Custom Transformers for Sakya

Sakya will need custom transformers for:

#### Wiki-Link Transformer

```javascript
const WIKI_LINK_TRANSFORMER = {
  dependencies: [WikiLinkNode],
  export: (node) => {
    if (!$isWikiLinkNode(node)) return null;
    const target = node.getTarget();
    const displayText = node.getTextContent();
    if (target === displayText) {
      return `[[${target}]]`;
    }
    return `[[${target}|${displayText}]]`;
  },
  importRegExp: /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/,
  regExp: /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]$/,
  replace: (textNode, match) => {
    const [, target, displayText] = match;
    const wikiLink = $createWikiLinkNode(target, displayText || target);
    textNode.replace(wikiLink);
  },
  trigger: ']',
  type: 'text-match',
};
```

#### Entity Reference Transformer

```javascript
const ENTITY_REF_TRANSFORMER = {
  dependencies: [EntityReferenceNode],
  export: (node) => {
    if (!$isEntityReferenceNode(node)) return null;
    const entityType = node.getEntityType();
    const entityId = node.getEntityId();
    const displayText = node.getTextContent();
    return `{{${entityType}:${entityId}|${displayText}}}`;
  },
  importRegExp: /\{\{(\w+):([^|}]+)\|([^}]+)\}\}/,
  regExp: /\{\{(\w+):([^|}]+)\|([^}]+)\}\}$/,
  replace: (textNode, match) => {
    const [, entityType, entityId, displayText] = match;
    const entityRef = $createEntityReferenceNode(entityType, entityId, displayText);
    textNode.replace(entityRef);
  },
  trigger: '}',
  type: 'text-match',
};
```

---

## Custom Nodes

### Node Hierarchy

Lexical's built-in node types form a hierarchy:

```
LexicalNode (base)
  ├── RootNode (singleton, top of tree)
  ├── ElementNode (has children)
  │     ├── ParagraphNode
  │     ├── HeadingNode
  │     ├── QuoteNode
  │     ├── ListNode
  │     ├── ListItemNode
  │     ├── TableNode, TableRowNode, TableCellNode
  │     └── LinkNode
  ├── TextNode (leaf, contains text)
  ├── LineBreakNode (leaf)
  ├── TabNode (leaf)
  └── DecoratorNode (leaf, renders arbitrary UI)
```

Custom nodes extend one of the base types:

- **Extend `ElementNode`**: For block-level containers (e.g., custom callout boxes,
  collapsible sections).
- **Extend `TextNode`**: For inline text with special behavior (e.g., mentions,
  hashtags).
- **Extend `DecoratorNode`**: For non-text inline or block content (e.g., embedded
  images, interactive widgets, character profile cards).

### Creating a Custom Node: WikiLinkNode

Here is a complete example of a custom inline node for wiki-links:

```javascript
import {
  $applyNodeReplacement,
  TextNode,
  type NodeKey,
  type SerializedTextNode,
  type Spread,
} from 'lexical';

// Serialized format for JSON persistence
type SerializedWikiLinkNode = Spread<
  {
    target: string;
    type: 'wiki-link';
    version: 1;
  },
  SerializedTextNode
>;

export class WikiLinkNode extends TextNode {
  __target: string;

  // Required: unique type identifier
  static getType(): string {
    return 'wiki-link';
  }

  // Required: clone for immutable state updates
  static clone(node: WikiLinkNode): WikiLinkNode {
    return new WikiLinkNode(node.__target, node.__text, node.__key);
  }

  constructor(target: string, text?: string, key?: NodeKey) {
    super(text ?? target, key);
    this.__target = target;
  }

  // Required: JSON deserialization
  static importJSON(serializedNode: SerializedWikiLinkNode): WikiLinkNode {
    const node = $createWikiLinkNode(
      serializedNode.target,
      serializedNode.text
    );
    node.setFormat(serializedNode.format);
    node.setDetail(serializedNode.detail);
    node.setMode(serializedNode.mode);
    node.setStyle(serializedNode.style);
    return node;
  }

  // Required: JSON serialization
  exportJSON(): SerializedWikiLinkNode {
    return {
      ...super.exportJSON(),
      target: this.__target,
      type: 'wiki-link',
      version: 1,
    };
  }

  // Required: DOM creation
  createDOM(config: EditorConfig): HTMLElement {
    const dom = super.createDOM(config);
    dom.classList.add('wiki-link');
    dom.setAttribute('data-wiki-target', this.__target);
    dom.style.color = '#5b21b6';
    dom.style.textDecoration = 'underline';
    dom.style.cursor = 'pointer';
    return dom;
  }

  // Required: DOM update (for efficiency)
  updateDOM(prevNode: WikiLinkNode, dom: HTMLElement, config: EditorConfig): boolean {
    const isUpdated = super.updateDOM(prevNode, dom, config);
    if (prevNode.__target !== this.__target) {
      dom.setAttribute('data-wiki-target', this.__target);
    }
    return isUpdated;
  }

  // Custom getter
  getTarget(): string {
    return this.__target;
  }

  // Custom setter (creates a writable clone)
  setTarget(target: string): this {
    const self = this.getWritable();
    self.__target = target;
    return self;
  }

  // Prevent text modification from breaking the link
  canInsertTextBefore(): boolean {
    return false;
  }

  canInsertTextAfter(): boolean {
    return false;
  }

  isTextEntity(): boolean {
    return true;
  }
}

// Factory function (required pattern in Lexical)
export function $createWikiLinkNode(target: string, text?: string): WikiLinkNode {
  return $applyNodeReplacement(new WikiLinkNode(target, text));
}

// Type guard
export function $isWikiLinkNode(node: LexicalNode | null | undefined): node is WikiLinkNode {
  return node instanceof WikiLinkNode;
}
```

### Required Methods for Custom Nodes

Every custom node must implement:

| Method | Purpose |
|---|---|
| `static getType()` | Returns a unique string identifier for the node type. Used for serialization and registration. |
| `static clone(node)` | Creates a shallow copy. Called during immutable state updates (the "writable" pattern). |
| `static importJSON(json)` | Deserializes a node from its JSON representation. |
| `exportJSON()` | Serializes the node to JSON. Must include `type` and `version` fields. |
| `createDOM(config)` | Creates and returns the DOM element for this node. Called once when the node first appears. |
| `updateDOM(prev, dom, config)` | Updates an existing DOM element. Returns `true` if the DOM needs to be recreated (rare). |

Optional but commonly implemented:

| Method | Purpose |
|---|---|
| `decorate()` | (DecoratorNode only) Returns a framework component to render. |
| `isInline()` | Returns `true` for inline nodes. |
| `canInsertTextBefore()` | Whether text can be typed before this node. |
| `canInsertTextAfter()` | Whether text can be typed after this node. |
| `isTextEntity()` | Whether the node represents a text entity (affects selection behavior). |
| `canMergeWith(node)` | Whether this node can merge with an adjacent node of the same type. |
| `exportDOM()` | Custom HTML export (for clipboard, etc.). |
| `importDOM()` | Custom HTML import (for paste, etc.). |

### DecoratorNode for Rich Embeds

`DecoratorNode` is the most powerful custom node type. It renders arbitrary UI
components within the editor. This is how you embed interactive widgets, images with
controls, character profile cards, or any non-text content.

```javascript
import { DecoratorNode } from 'lexical';

export class CharacterCardNode extends DecoratorNode {
  __characterId: string;

  static getType() {
    return 'character-card';
  }

  static clone(node) {
    return new CharacterCardNode(node.__characterId, node.__key);
  }

  constructor(characterId, key) {
    super(key);
    this.__characterId = characterId;
  }

  createDOM() {
    const div = document.createElement('div');
    div.classList.add('character-card-container');
    return div;
  }

  updateDOM() {
    return false;
  }

  // This is where the magic happens: return a Svelte component
  decorate() {
    return {
      component: CharacterCard,
      props: {
        characterId: this.__characterId,
      },
    };
  }

  static importJSON(json) {
    return new CharacterCardNode(json.characterId);
  }

  exportJSON() {
    return {
      type: 'character-card',
      version: 1,
      characterId: this.__characterId,
    };
  }

  // Block-level node
  isInline() {
    return false;
  }
}
```

### NodeState API

The NodeState API (introduced in later Lexical versions) reduces boilerplate for
custom nodes. Instead of manually implementing getters, setters, `clone`,
`importJSON`, and `exportJSON` for each property, you declare the state shape:

```javascript
import { ElementNode, NodeState } from 'lexical';

const calloutState = NodeState.define({
  type: NodeState.string('info'), // 'info' | 'warning' | 'error'
  collapsed: NodeState.boolean(false),
});

export class CalloutNode extends ElementNode {
  static getType() {
    return 'callout';
  }

  // NodeState auto-generates clone, importJSON, exportJSON,
  // and typed getters/setters for 'type' and 'collapsed'

  createDOM(config) {
    const div = document.createElement('div');
    div.classList.add('callout', `callout-${this.getState().type}`);
    return div;
  }

  updateDOM(prevNode, dom) {
    if (prevNode.getState().type !== this.getState().type) {
      dom.className = `callout callout-${this.getState().type}`;
    }
    return false;
  }
}
```

The NodeState API is particularly valuable for Sakya because custom entity nodes
(characters, locations, items) will have many properties, and manually implementing
serialization for each would be tedious and error-prone.

---

## svelte-lexical

### Overview

`svelte-lexical` (version 0.6.4 as of research) provides Svelte bindings for Lexical.
It wraps Lexical's core functionality in Svelte components and stores, enabling
idiomatic Svelte development.

### Architecture

svelte-lexical uses a Composer pattern:

```svelte
<script>
  import { Composer, ContentEditable, RichTextPlugin } from 'svelte-lexical';
  import { HeadingNode, QuoteNode } from '@lexical/rich-text';
  import { ListNode, ListItemNode } from '@lexical/list';
  import { CodeNode } from '@lexical/code';
  import { LinkNode } from '@lexical/link';

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
      WikiLinkNode,
    ],
    onError: (error) => console.error(error),
  };
</script>

<Composer {initialConfig}>
  <div class="editor-container">
    <RichTextPlugin>
      <ContentEditable class="editor-content" />
    </RichTextPlugin>
    <!-- Plugins are children of Composer -->
    <MarkdownShortcutsPlugin />
    <HistoryPlugin />
    <AutoFocusPlugin />
    <WikiLinkPlugin />
    <WordCountPlugin />
  </div>
</Composer>
```

### Accessing the Editor Instance

Inside the `Composer`, any child component can access the editor instance:

```svelte
<script>
  import { getContext } from 'svelte';

  // svelte-lexical stores the editor in Svelte context
  const editor = getContext('editor');

  // Now you can use the editor directly
  function insertWikiLink(target) {
    editor.dispatchCommand(INSERT_WIKI_LINK_COMMAND, { target });
  }

  // Or register listeners
  import { onMount, onDestroy } from 'svelte';

  let removeListener;

  onMount(() => {
    removeListener = editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const root = $getRoot();
        wordCount = root.getTextContent().split(/\s+/).filter(Boolean).length;
      });
    });
  });

  onDestroy(() => {
    removeListener?.();
  });
</script>
```

### Plugin Architecture in svelte-lexical

Plugins in svelte-lexical are Svelte components that register listeners, commands, or
transforms on mount and clean up on destroy:

```svelte
<!-- WordCountPlugin.svelte -->
<script>
  import { getContext, onMount, onDestroy } from 'svelte';
  import { $getRoot } from 'lexical';

  const editor = getContext('editor');

  export let onWordCountChange = (count) => {};

  let removeListener;

  onMount(() => {
    removeListener = editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const root = $getRoot();
        const text = root.getTextContent();
        const count = text.split(/\s+/).filter(Boolean).length;
        onWordCountChange(count);
      });
    });
  });

  onDestroy(() => {
    removeListener?.();
  });
</script>

<!-- No DOM output needed; this is a "headless" plugin -->
```

Usage:

```svelte
<Composer {initialConfig}>
  <RichTextPlugin>
    <ContentEditable />
  </RichTextPlugin>
  <WordCountPlugin onWordCountChange={(count) => wordCount = count} />
</Composer>

<div class="status-bar">Words: {wordCount}</div>
```

---

## Plugin Patterns

### Registering Listeners

Lexical provides several listener types:

```javascript
// Update listener: fires after every state change
const remove1 = editor.registerUpdateListener(({ editorState, prevEditorState, tags }) => {
  // Compare states, update external UI, trigger auto-save, etc.
});

// Text content listener: fires when text content changes
const remove2 = editor.registerTextContentListener((textContent) => {
  // Simple text change notification
});

// Mutation listener: fires when specific node types are created/updated/destroyed
const remove3 = editor.registerMutationListener(WikiLinkNode, (mutations) => {
  // mutations is a Map<NodeKey, 'created' | 'updated' | 'destroyed'>
  for (const [key, type] of mutations) {
    if (type === 'created') {
      // A new wiki-link was created
    }
  }
});

// Decorator listener: fires when decorator nodes change
const remove4 = editor.registerDecoratorListener((decorators) => {
  // decorators is a Record<NodeKey, DecoratorValue>
});
```

### Registering Commands

```javascript
// Handle a built-in command
const remove = editor.registerCommand(
  KEY_ENTER_COMMAND,
  (event) => {
    // Custom Enter key behavior
    // Return true to prevent other handlers from running
    // Return false to let the default handler run
    return false;
  },
  COMMAND_PRIORITY_NORMAL
);
```

### Registering Node Transforms

```javascript
const remove = editor.registerNodeTransform(ParagraphNode, (node) => {
  // Run every time a ParagraphNode is created or modified
  const text = node.getTextContent();

  // Example: auto-detect and convert scene separators
  if (text.trim() === '***' || text.trim() === '---') {
    const separator = $createSceneSeparatorNode();
    node.replace(separator);
  }
});
```

### Cleanup Pattern

All registration functions return cleanup functions. This is critical for preventing
memory leaks:

```javascript
// In a Svelte component
onMount(() => {
  const cleanups = [
    editor.registerCommand(MY_COMMAND, handler, COMMAND_PRIORITY_NORMAL),
    editor.registerUpdateListener(updateHandler),
    editor.registerNodeTransform(TextNode, transformHandler),
    editor.registerMutationListener(WikiLinkNode, mutationHandler),
  ];

  return () => {
    cleanups.forEach((cleanup) => cleanup());
  };
});
```

Or using a helper:

```javascript
function mergeCleanups(...cleanups) {
  return () => cleanups.forEach((fn) => fn());
}

onMount(() => {
  const cleanup = mergeCleanups(
    editor.registerCommand(...),
    editor.registerUpdateListener(...),
    editor.registerNodeTransform(...),
  );

  return cleanup;
});
```

---

## Writing App Considerations

### Focus Mode (Dim Non-Active Paragraphs)

Focus mode dims all paragraphs except the one currently being edited, reducing
visual distraction.

Implementation approach:

```javascript
// FocusModePlugin
const remove = editor.registerUpdateListener(({ editorState }) => {
  editorState.read(() => {
    const selection = $getSelection();
    if (!$isRangeSelection(selection)) return;

    const anchorNode = selection.anchor.getNode();
    const activeParagraph = $findMatchingParent(anchorNode, $isParagraphNode)
      ?? anchorNode;

    // Apply CSS classes
    const root = $getRoot();
    for (const child of root.getChildren()) {
      const dom = editor.getElementByKey(child.getKey());
      if (dom) {
        if (child.getKey() === activeParagraph.getKey()) {
          dom.classList.remove('dimmed');
          dom.classList.add('focused');
        } else {
          dom.classList.add('dimmed');
          dom.classList.remove('focused');
        }
      }
    }
  });
});
```

CSS:

```css
.editor-content .dimmed {
  opacity: 0.3;
  transition: opacity 0.3s ease;
}

.editor-content .focused {
  opacity: 1;
  transition: opacity 0.15s ease;
}
```

### Word Count

Word count is essential for any writing app. Implementation:

```javascript
editor.registerUpdateListener(({ editorState }) => {
  editorState.read(() => {
    const root = $getRoot();
    const text = root.getTextContent();
    const words = text.split(/\s+/).filter(Boolean).length;
    const characters = text.length;
    const paragraphs = root.getChildren().length;

    updateWordCount({ words, characters, paragraphs });
  });
});
```

For large documents, debounce the word count calculation:

```javascript
let wordCountTimeout;

editor.registerUpdateListener(({ editorState }) => {
  clearTimeout(wordCountTimeout);
  wordCountTimeout = setTimeout(() => {
    editorState.read(() => {
      // ... word count calculation
    });
  }, 300);
});
```

### Auto-Save

Auto-save should serialize the editor state and persist it:

```javascript
let saveTimeout;
const SAVE_DELAY = 2000; // 2 seconds after last edit

editor.registerUpdateListener(({ editorState, dirtyElements, dirtyLeaves }) => {
  // Only save if something actually changed
  if (dirtyElements.size === 0 && dirtyLeaves.size === 0) return;

  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    editorState.read(() => {
      // Option 1: Save as Markdown
      const markdown = $convertToMarkdownString(SAKYA_TRANSFORMERS);
      saveToFile(currentDocumentPath, markdown);

      // Option 2: Save as Lexical JSON (for lossless round-trip)
      const json = editorState.toJSON();
      saveToFile(currentDocumentPath + '.lexical', JSON.stringify(json));
    });
  }, SAVE_DELAY);
});
```

### Wiki-Links as Custom Nodes

Wiki-links (`[[Target]]` or `[[Target|Display Text]]`) are a core Sakya feature.
They should be custom nodes that:

1. **Render as styled inline elements** with the display text visible and the target
   accessible on hover or click.
2. **Navigate on click**: Clicking a wiki-link opens the linked document in the
   editor or in a split pane.
3. **Show hover previews**: Hovering over a wiki-link shows a tooltip with the
   target document's synopsis.
4. **Auto-complete**: Typing `[[` triggers an autocomplete popup showing available
   link targets (documents, characters, locations, etc.).
5. **Serialize to Markdown**: Wiki-links serialize to `[[target]]` or
   `[[target|display text]]` syntax.
6. **Validate**: Invalid links (pointing to non-existent documents) are visually
   distinguished (e.g., red underline).

### Distraction-Free Mode

Full-screen, minimal UI writing mode:

- Hide binder, inspector, toolbar, and status bar
- Center the editor content with generous margins
- Dim or hide the scrollbar
- Optional: dark background with light text
- Optional: typewriter scrolling (keep cursor vertically centered)
- Keyboard shortcut to toggle (e.g., F11 or Cmd+Shift+F)

Implementation: This is primarily a UI concern (CSS and component visibility), not
an editor concern. The editor itself does not change; only its container styling does.

### Typewriter Scrolling

Keep the active line vertically centered in the viewport:

```javascript
editor.registerUpdateListener(() => {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0) return;

  const range = selection.getRangeAt(0);
  const rect = range.getBoundingClientRect();
  const editorRect = editorElement.getBoundingClientRect();

  const targetY = editorRect.top + editorRect.height / 2;
  const currentY = rect.top;
  const scrollDelta = currentY - targetY;

  editorElement.scrollBy({
    top: scrollDelta,
    behavior: 'smooth',
  });
});
```

---

## Markdown Round-Trip Fidelity

### The Canonical State Problem

A critical architectural decision: **what is the canonical representation of a
document?**

Option A: **Markdown is canonical**. The editor imports Markdown on open, and exports
Markdown on save. The Lexical EditorState is a transient, in-memory representation.

Option B: **EditorState (JSON) is canonical**. The Lexical JSON is the source of
truth. Markdown is a serialization format for export, file storage, and Git
compatibility.

Option C: **Markdown is canonical for storage, EditorState for editing**. Markdown
files are what is stored on disk and committed to Git. When a file is opened, it is
parsed into an EditorState. When saved, the EditorState is serialized back to Markdown.
The round-trip must be lossless for supported syntax.

**Recommendation for Sakya: Option C.**

Rationale:
- Markdown files on disk are human-readable, Git-friendly, and editor-agnostic.
- The EditorState provides rich editing capabilities that plain text cannot.
- The round-trip requirement ensures that opening and saving a file without editing
  produces identical output (no "phantom diffs" in Git).

### Round-Trip Challenges

Markdown to EditorState to Markdown is not inherently lossless. Common issues:

1. **Whitespace normalization**: Lexical may normalize whitespace (collapsing multiple
   spaces, trimming trailing spaces) that was significant in the original Markdown.
2. **Emphasis ambiguity**: `*italic*` and `_italic_` both produce italic text.
   Lexical cannot preserve which syntax was used.
3. **List markers**: `-`, `*`, and `+` are all valid unordered list markers. Lexical
   normalizes to one.
4. **Heading syntax**: `# Heading` and `Heading\n===` both produce H1. Lexical
   normalizes to `#` syntax.
5. **Link syntax**: `[text](url)` and `[text][ref]` (reference-style links) produce
   the same LinkNode. Reference definitions are lost.
6. **HTML in Markdown**: Raw HTML in Markdown may not survive the round-trip.

### Mitigation Strategies

1. **Standardize on a single Markdown dialect**: Use a strict subset of CommonMark.
   Document which syntax forms are canonical. Always output the same form.
2. **Custom transformers for custom syntax**: Wiki-links, entity references, and
   other Sakya-specific syntax must have dedicated transformers that preserve
   the syntax exactly.
3. **Preserve frontmatter**: YAML frontmatter (`---\ntitle: ...\n---`) must be
   preserved through the round-trip. Lexical does not natively handle frontmatter,
   so Sakya must extract it before parsing and re-attach it on save.
4. **Test the round-trip**: Automated tests that parse Markdown, convert to
   EditorState, convert back to Markdown, and compare. Any differences are bugs.

### Frontmatter Handling

Sakya documents use YAML frontmatter for metadata:

```markdown
---
title: "Chapter 3: The Storm"
label: revised
status: in-progress
synopsis: "Elara confronts the guardian of the Northern Gate."
word_target: 3000
custom:
  pov: "Elara"
  timeline: "Day 14"
---

The wind hit her like a wall when she stepped through the gate...
```

Handling strategy:

```javascript
function parseDocument(markdownWithFrontmatter) {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---\n/;
  const match = markdownWithFrontmatter.match(frontmatterRegex);

  let frontmatter = null;
  let markdown = markdownWithFrontmatter;

  if (match) {
    frontmatter = parseYAML(match[1]);
    markdown = markdownWithFrontmatter.slice(match[0].length);
  }

  return { frontmatter, markdown };
}

function serializeDocument(editorState, frontmatter) {
  let output = '';

  if (frontmatter) {
    output += '---\n' + serializeYAML(frontmatter) + '\n---\n\n';
  }

  editorState.read(() => {
    output += $convertToMarkdownString(SAKYA_TRANSFORMERS);
  });

  return output;
}
```

---

## Performance Considerations

### Large Document Handling

Lexical was designed for social media posts and comments (typically under 10KB).
For a writing application handling novel chapters, performance must be considered:

| Document Size | Typical Content | Expected Performance |
|---|---|---|
| < 10KB | Short story, single scene | Excellent, no issues |
| 10-50KB | Novel chapter (3,000-15,000 words) | Good, minimal latency |
| 50-100KB | Long chapter or multiple scenes | Acceptable, slight lag on updates |
| 100-500KB | Entire short novel | Noticeable lag, may need mitigation |
| > 500KB | Full novel in single document | Slow, not recommended |

### Performance Bottlenecks

1. **Reconciliation**: Each `editor.update()` triggers a full diff of the node tree
   and DOM reconciliation. For large documents, this diff can take 10-50ms, causing
   perceptible input lag.

2. **Transform cascades**: Node transforms that trigger other transforms can cause
   cascading updates. In a large document with many wiki-links or entity references,
   this can be expensive.

3. **Selection listeners**: Update listeners that run on every keystroke (word count,
   auto-save) can accumulate. Debouncing is essential.

4. **Memory**: Each node in the tree is a JavaScript object. A 100KB document might
   have 5,000+ nodes, each consuming memory.

### Mitigation Strategies

1. **Document splitting**: Encourage writers to split manuscripts into chapters/scenes
   (which aligns with Scrivener's model). Each binder item is a separate Lexical
   editor instance. This keeps individual documents under 50KB.

2. **Lazy loading**: Only instantiate editor instances for documents that are
   currently visible. Documents in the binder but not open in the editor do not need
   active Lexical instances.

3. **Debounced listeners**: Debounce word count, auto-save, and other non-critical
   listeners to avoid running on every keystroke.

4. **Virtualized rendering**: For very long documents, consider virtualizing the
   rendering so that only visible paragraphs have DOM nodes. This is complex to
   implement with Lexical but possible using DecoratorNodes as placeholders for
   off-screen content. (This is a future optimization, not needed for MVP.)

5. **Web Workers**: Offload expensive operations (Markdown serialization, word count
   for large documents, search) to Web Workers to avoid blocking the main thread.

6. **Batch transforms**: Group node transforms to minimize reconciliation passes.

### Benchmarking Recommendations

Before committing to Lexical, benchmark:

1. **Typing latency**: Measure input-to-screen latency for documents of 10KB, 50KB,
   and 100KB. Target: < 16ms (60fps).
2. **Paste performance**: Measure time to paste 10KB of text. Target: < 100ms.
3. **Markdown export**: Measure time to serialize a 50KB document to Markdown.
   Target: < 50ms.
4. **Markdown import**: Measure time to parse a 50KB Markdown file into EditorState.
   Target: < 100ms.
5. **Memory usage**: Measure heap size for documents of various sizes.
6. **Mount time**: Measure time to create an editor instance and render a document.
   Target: < 200ms for 50KB.

---

## Integration with Sakya's Architecture

### Editor as a Managed Component

The Lexical editor should be managed by Sakya's application layer, not the other way
around. The application controls:

- Which document is loaded into the editor
- When saves occur
- How the editor state maps to the binder tree
- How wiki-links resolve to binder items
- How entity references resolve to entity data

The editor is a view component that receives content and emits changes. It does not
own the document model.

### State Flow

```
Binder (click document)
  -> App loads Markdown file from disk
  -> App parses frontmatter + body
  -> App initializes EditorState from Markdown body
  -> Editor renders EditorState
  -> User types
  -> Editor dispatches updates
  -> App debounces auto-save
  -> App serializes EditorState to Markdown
  -> App writes frontmatter + Markdown to disk
```

### Multi-Document Considerations

Sakya may have multiple documents open (split-screen, tabs). Each needs its own
Lexical editor instance. Considerations:

- **Instance management**: Create and destroy editor instances as documents are
  opened and closed. Do not keep inactive instances alive.
- **Shared configuration**: All instances share the same node types, theme, and
  custom transformers. Create a factory function.
- **Cross-document commands**: Wiki-link navigation triggers loading a different
  document. This is an application-level action, not an editor action.
- **Undo scope**: Each editor instance has its own undo history. Undo does not
  cross document boundaries.

### Toolbar and Formatting Controls

The toolbar (bold, italic, heading, list, etc.) communicates with the editor via
commands:

```svelte
<script>
  import { getContext } from 'svelte';
  import { FORMAT_TEXT_COMMAND } from 'lexical';

  const editor = getContext('editor');

  function toggleBold() {
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold');
  }

  function toggleItalic() {
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic');
  }

  function insertHeading(level) {
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createHeadingNode(`h${level}`));
      }
    });
  }
</script>

<div class="toolbar">
  <button on:click={toggleBold}><strong>B</strong></button>
  <button on:click={toggleItalic}><em>I</em></button>
  <button on:click={() => insertHeading(1)}>H1</button>
  <button on:click={() => insertHeading(2)}>H2</button>
  <button on:click={() => insertHeading(3)}>H3</button>
</div>
```

---

## Risks and Open Questions

1. **svelte-lexical maturity**: svelte-lexical (v0.6.4) is not as mature as
   @lexical/react. It may have bugs, missing features, or performance issues that
   the React bindings do not. Mitigation: be prepared to contribute upstream or
   fork if necessary.

2. **Markdown round-trip fidelity**: Achieving true lossless round-trip for all
   supported Markdown syntax is non-trivial. It will require extensive testing and
   possibly custom modifications to Lexical's Markdown package.

3. **Large document performance**: While document splitting mitigates this, some
   writers will insist on single-document workflows. We need to understand the
   performance ceiling and communicate it clearly.

4. **Custom node complexity**: Each custom node type (wiki-link, entity reference,
   inline metadata, etc.) adds complexity to the Markdown transformer layer. The
   transformer code must be rigorously tested.

5. **Lexical versioning**: Lexical is actively developed and has breaking changes
   between versions. Sakya should pin a specific version and upgrade deliberately.

6. **Accessibility**: Lexical's contenteditable approach has known accessibility
   challenges. Screen reader support, keyboard navigation, and ARIA attributes
   must be tested thoroughly.

7. **Mobile support**: Lexical's mobile (touch) editing support is less mature than
   desktop. If Sakya targets mobile platforms, this is a significant concern.

---

## Summary

Lexical provides a strong foundation for Sakya's editor:

- **Small core** with a composable plugin architecture matches Sakya's need for a
  lean, extensible editor.
- **Custom nodes** enable wiki-links, entity references, and other Sakya-specific
  inline elements.
- **Markdown interop** via `@lexical/markdown` provides the import/export pipeline
  needed for Markdown-canonical storage.
- **Command/listener system** enables clean separation between the editor and the
  application layer.
- **svelte-lexical** provides idiomatic Svelte bindings, though maturity is a
  consideration.

The main challenges are Markdown round-trip fidelity, large document performance,
and svelte-lexical maturity. These are manageable with deliberate architecture
(document splitting, debounced listeners, comprehensive round-trip tests) and a
willingness to contribute upstream.
