import { TRANSFORMERS } from '@lexical/markdown';
import { WIKI_LINK_TRANSFORMER } from './wiki-link';

/**
 * Combined Markdown transformers for the Sakya editor.
 * Includes all standard Lexical transformers plus custom ones:
 * - Wiki-link: [[Target Name]]
 */
export const SAKYA_TRANSFORMERS = [...TRANSFORMERS, WIKI_LINK_TRANSFORMER];
