import type { EditorThemeClasses } from 'lexical';

/**
 * Sakya editor theme mapping Lexical node types to CSS class names.
 * All classes are defined in editor.css and reference CSS custom properties
 * from app.css for light/dark theme support.
 */
export const sakyaEditorTheme: EditorThemeClasses = {
	paragraph: 'editor-paragraph',
	heading: {
		h1: 'editor-heading-h1',
		h2: 'editor-heading-h2',
		h3: 'editor-heading-h3',
		h4: 'editor-heading-h4',
		h5: 'editor-heading-h5',
		h6: 'editor-heading-h6',
	},
	quote: 'editor-quote',
	list: {
		ul: 'editor-list-ul',
		ol: 'editor-list-ol',
		listitem: 'editor-listitem',
		nested: {
			listitem: 'editor-nested-listitem',
		},
	},
	code: 'editor-code',
	link: 'editor-link',
	wikiLink: 'editor-wiki-link',
	text: {
		bold: 'editor-text-bold',
		italic: 'editor-text-italic',
		strikethrough: 'editor-text-strikethrough',
		code: 'editor-text-code',
		underline: 'editor-text-underline',
	},
};
