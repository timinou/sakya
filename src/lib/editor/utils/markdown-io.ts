import { $convertFromMarkdownString, $convertToMarkdownString } from '@lexical/markdown';
import type { LexicalEditor } from 'lexical';
import { SAKYA_TRANSFORMERS } from '../transformers';

/**
 * Loads a Markdown string into the editor, replacing its current content.
 * Handles empty/null/undefined input by loading an empty paragraph.
 */
export function markdownToEditorState(
  editor: LexicalEditor,
  markdown: string | null | undefined
): void {
  editor.update(() => {
    $convertFromMarkdownString(markdown ?? '', SAKYA_TRANSFORMERS);
  });
}

/**
 * Reads the current editor state and returns it as a Markdown string.
 * Returns an empty string if the editor state cannot be read.
 */
export function editorStateToMarkdown(editor: LexicalEditor): string {
  let markdown = '';
  editor.getEditorState().read(() => {
    markdown = $convertToMarkdownString(SAKYA_TRANSFORMERS);
  });
  return markdown;
}
