import {
	$applyNodeReplacement,
	TextNode,
	type EditorConfig,
	type LexicalNode,
	type LexicalUpdateJSON,
	type NodeKey,
	type SerializedTextNode,
	type Spread,
} from 'lexical';

export type SerializedWikiLinkNode = Spread<
	{
		target: string;
		type: 'wiki-link';
		version: 1;
	},
	SerializedTextNode
>;

/**
 * Custom Lexical node for wiki-style [[links]].
 * Extends TextNode to represent internal cross-references between documents.
 * Renders as a styled <span> with the editor-wiki-link theme class.
 */
export class WikiLinkNode extends TextNode {
	__target: string;

	static getType(): string {
		return 'wiki-link';
	}

	static clone(node: WikiLinkNode): WikiLinkNode {
		return new WikiLinkNode(node.__target, node.__text, node.__key);
	}

	constructor(target: string, text?: string, key?: NodeKey) {
		super(text ?? target, key);
		this.__target = target;
	}

	afterCloneFrom(prevNode: this): void {
		super.afterCloneFrom(prevNode);
		this.__target = prevNode.__target;
	}

	static importJSON(serializedNode: SerializedWikiLinkNode): WikiLinkNode {
		const node = $createWikiLinkNode(serializedNode.target, serializedNode.text);
		node.setFormat(serializedNode.format);
		node.setDetail(serializedNode.detail);
		node.setMode(serializedNode.mode);
		node.setStyle(serializedNode.style);
		return node;
	}

	updateFromJSON(serializedNode: LexicalUpdateJSON<SerializedWikiLinkNode>): this {
		return super.updateFromJSON(serializedNode).setTarget(serializedNode.target);
	}

	exportJSON(): SerializedWikiLinkNode {
		return {
			...super.exportJSON(),
			target: this.__target,
			type: 'wiki-link',
			version: 1,
		};
	}

	createDOM(config: EditorConfig): HTMLElement {
		const dom = super.createDOM(config);
		const wikiLinkClass = config.theme.wikiLink;
		if (typeof wikiLinkClass === 'string') {
			dom.className = wikiLinkClass;
		}
		return dom;
	}

	updateDOM(prevNode: this, dom: HTMLElement, config: EditorConfig): boolean {
		const isUpdated = super.updateDOM(prevNode, dom, config);
		if (prevNode.__target !== this.__target) {
			return true;
		}
		return isUpdated;
	}

	getTarget(): string {
		const self = this.getLatest();
		return self.__target;
	}

	setTarget(target: string): this {
		const self = this.getWritable();
		self.__target = target;
		return self;
	}

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

/**
 * Creates a new WikiLinkNode.
 *
 * @param target - The link target (document/entity name).
 * @param text - Optional display text. Defaults to the target.
 * @returns A new WikiLinkNode instance.
 */
export function $createWikiLinkNode(target: string, text?: string): WikiLinkNode {
	const node = new WikiLinkNode(target, text);
	node.setMode('token');
	return $applyNodeReplacement(node);
}

/**
 * Type guard for WikiLinkNode.
 */
export function $isWikiLinkNode(
	node: LexicalNode | null | undefined,
): node is WikiLinkNode {
	return node instanceof WikiLinkNode;
}
