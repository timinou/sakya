<script lang="ts">
	import { getEditor } from 'svelte-lexical';
	import { onMount } from 'svelte';
	import {
		CLICK_COMMAND,
		COMMAND_PRIORITY_LOW,
		$getSelection as getSelection,
		$isRangeSelection as isRangeSelection,
	} from 'lexical';
	import { $isWikiLinkNode as isWikiLinkNode } from '../nodes/WikiLinkNode';

	interface Props {
		onNavigate?: (target: string) => void;
	}

	let { onNavigate }: Props = $props();

	const editor = getEditor();

	onMount(() => {
		const removeClickListener = editor.registerCommand(
			CLICK_COMMAND,
			(event: MouseEvent) => {
				const target = event.target as HTMLElement;

				// Early exit if the click is not on a wiki-link DOM element
				if (!target.closest('.editor-wiki-link')) return false;

				// Read the editor state to find the corresponding WikiLinkNode
				let handled = false;
				editor.getEditorState().read(() => {
					const selection = getSelection();
					if (!selection || !isRangeSelection(selection)) return;

					// WikiLinkNode is a TextNode in token mode, so the
					// selection anchor node is the WikiLinkNode itself
					const anchorNode = selection.isCollapsed()
						? selection.anchor.getNode()
						: null;

					if (anchorNode && isWikiLinkNode(anchorNode)) {
						const linkTarget = anchorNode.getTarget();
						onNavigate?.(linkTarget);
						handled = true;
						return;
					}

					// Fallback: check the parent in case selection landed on
					// a child text node
					if (anchorNode) {
						const parent = anchorNode.getParent();
						if (parent && isWikiLinkNode(parent)) {
							const linkTarget = parent.getTarget();
							onNavigate?.(linkTarget);
							handled = true;
						}
					}
				});

				if (handled) {
					event.preventDefault();
					return true;
				}

				return false;
			},
			COMMAND_PRIORITY_LOW,
		);

		return () => {
			removeClickListener();
		};
	});
</script>
