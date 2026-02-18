<script lang="ts">
  import { getEditor } from 'svelte-lexical';
  import { onMount } from 'svelte';
  import {
    $convertFromMarkdownString as convertFromMarkdownString,
    $convertToMarkdownString as convertToMarkdownString,
  } from '@lexical/markdown';
  import { $getRoot as getRoot } from 'lexical';
  import { SAKYA_TRANSFORMERS } from '../transformers';
  import type { LoroDoc, LoroText, Subscription } from 'loro-crdt';

  interface Props {
    /** The Loro CRDT document to sync with. */
    loroDoc: LoroDoc;
    /** Container ID for this document's text (e.g., the chapter slug). */
    containerId: string;
    /** Callback with binary updates for sync transport. */
    onLocalUpdate?: (update: Uint8Array) => void;
    /** Debounce interval for Lexical → Loro sync (ms). */
    debounceMs?: number;
  }

  let {
    loroDoc,
    containerId,
    onLocalUpdate,
    debounceMs = 300,
  }: Props = $props();

  const editor = getEditor();

  onMount(() => {
    const text: LoroText = loroDoc.getText(containerId);
    let isApplyingRemote = false;
    let isApplyingLocal = false;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let lastLocalMarkdown = '';

    // ─── Lexical → Loro ──────────────────────────────────────
    // When the editor changes locally, export markdown and update
    // the Loro text container. Debounced to batch rapid keystrokes.
    const removeUpdateListener = editor.registerUpdateListener(
      ({ dirtyElements, dirtyLeaves, editorState }) => {
        // Skip if this update was triggered by applying remote changes
        if (isApplyingRemote) return;
        // Skip no-op updates
        if (dirtyElements.size === 0 && dirtyLeaves.size === 0) return;

        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          editorState.read(() => {
            const markdown = convertToMarkdownString(SAKYA_TRANSFORMERS);

            // Only sync if content actually changed
            if (markdown === lastLocalMarkdown) return;
            lastLocalMarkdown = markdown;

            // Replace the entire Loro text with the new markdown
            isApplyingLocal = true;
            try {
              const currentLen = text.length;
              if (currentLen > 0) {
                text.delete(0, currentLen);
              }
              if (markdown.length > 0) {
                text.insert(0, markdown);
              }
              loroDoc.commit({ origin: 'lexical' });
            } finally {
              isApplyingLocal = false;
            }
          });
        }, debounceMs);
      },
    );

    // ─── Binary update transport ─────────────────────────────
    // Forward binary Loro updates to the sync transport layer.
    let unsubscribeLocalUpdates: (() => void) | null = null;
    if (onLocalUpdate) {
      unsubscribeLocalUpdates = loroDoc.subscribeLocalUpdates((update) => {
        onLocalUpdate(update);
      });
    }

    // ─── Loro → Lexical ──────────────────────────────────────
    // When remote changes arrive (via import), update the editor.
    const unsubscribeDoc: Subscription = loroDoc.subscribe((eventBatch) => {
      // Skip events triggered by our own local edits
      if (isApplyingLocal) return;
      if (eventBatch.by === 'local') return;

      // Find text events for our container
      const hasTextChange = eventBatch.events.some(
        (ev) => ev.target === text.id,
      );
      if (!hasTextChange) return;

      // Read the updated Loro text content
      const remoteMarkdown = text.toString();

      // Skip if content is the same (avoids unnecessary editor updates)
      if (remoteMarkdown === lastLocalMarkdown) return;
      lastLocalMarkdown = remoteMarkdown;

      // Apply to Lexical editor
      isApplyingRemote = true;
      editor.update(
        () => {
          // Save current selection for restoration
          const root = getRoot();
          root.clear();
          convertFromMarkdownString(remoteMarkdown, SAKYA_TRANSFORMERS);
        },
        {
          discrete: true,
          tag: 'loro-sync',
        },
      );
      // Reset the flag after a microtask to ensure the update listener fires first
      queueMicrotask(() => {
        isApplyingRemote = false;
      });
    });

    // ─── Initial sync ────────────────────────────────────────
    // If Loro already has content (reconnection scenario), apply it.
    // Otherwise, push the current editor content to Loro.
    const existingContent = text.toString();
    if (existingContent.length > 0) {
      // Loro has content — apply to editor
      lastLocalMarkdown = existingContent;
      isApplyingRemote = true;
      editor.update(
        () => {
          const root = getRoot();
          root.clear();
          convertFromMarkdownString(existingContent, SAKYA_TRANSFORMERS);
        },
        {
          discrete: true,
          tag: 'loro-sync-init',
        },
      );
      queueMicrotask(() => {
        isApplyingRemote = false;
      });
    } else {
      // Editor has content — push to Loro
      editor.getEditorState().read(() => {
        const markdown = convertToMarkdownString(SAKYA_TRANSFORMERS);
        if (markdown.length > 0) {
          lastLocalMarkdown = markdown;
          isApplyingLocal = true;
          try {
            text.insert(0, markdown);
            loroDoc.commit({ origin: 'lexical-init' });
          } finally {
            isApplyingLocal = false;
          }
        }
      });
    }

    // ─── Cleanup ─────────────────────────────────────────────
    return () => {
      if (timeoutId) clearTimeout(timeoutId);
      removeUpdateListener();
      unsubscribeDoc();
      unsubscribeLocalUpdates?.();
    };
  });
</script>
