import type { TextMatchTransformer } from '@lexical/markdown';
import { WikiLinkNode, $createWikiLinkNode, $isWikiLinkNode } from '../nodes/WikiLinkNode';

/**
 * Markdown transformer for wiki-link syntax: [[Target Name]]
 *
 * - Export: WikiLinkNode -> [[target]]
 * - Import: [[target]] regex match -> WikiLinkNode
 * - Trigger: typing ]] completes the wiki link
 */
export const WIKI_LINK_TRANSFORMER: TextMatchTransformer = {
	dependencies: [WikiLinkNode],
	export: (node) => {
		if (!$isWikiLinkNode(node)) {
			return null;
		}
		return `[[${node.getTarget()}]]`;
	},
	importRegExp: /\[\[([^\]]+)\]\]/,
	regExp: /\[\[([^\]]+)\]\]$/,
	replace: (textNode, match) => {
		const [, target] = match;
		if (target) {
			const wikiLinkNode = $createWikiLinkNode(target);
			textNode.replace(wikiLinkNode);
		}
	},
	trigger: ']',
	type: 'text-match',
};
