<script lang="ts">
	import { getEditor } from 'svelte-lexical';
	import { onMount, tick } from 'svelte';
	import {
		CLICK_COMMAND,
		COMMAND_PRIORITY_LOW,
		COMMAND_PRIORITY_HIGH,
		KEY_DOWN_COMMAND,
		KEY_ESCAPE_COMMAND,
		$getSelection as getSelection,
		$isRangeSelection as isRangeSelection,
		$createTextNode as createTextNode,
		type TextNode,
	} from 'lexical';
	import { $isWikiLinkNode as isWikiLinkNode, $createWikiLinkNode as createWikiLinkNode } from '../nodes/WikiLinkNode';
	import { entityStore } from '$lib/stores';
	import { Users, MapPin, Package, Lightbulb, File } from 'lucide-svelte';
	import type { ComponentType } from 'svelte';
	import type { EntitySummary } from '$lib/types';

	interface Props {
		onNavigate?: (target: string) => void;
	}

	let { onNavigate }: Props = $props();

	const editor = getEditor();

	// --- Autocomplete state ---
	let showAutocomplete = $state(false);
	let autocompleteQuery = $state('');
	let autocompletePosition = $state({ top: 0, left: 0 });
	let selectedIndex = $state(0);
	let autocompleteAnchorOffset = $state(0);
	let dropdownEl: HTMLDivElement | undefined = $state(undefined);

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	type IconComponent = ComponentType<any>;

	const entityIcons: Record<string, IconComponent> = {
		character: Users,
		place: MapPin,
		item: Package,
		idea: Lightbulb,
	};

	const entityColors: Record<string, string> = {
		character: '#7c4dbd',
		place: '#2e8b57',
		item: '#c28a1e',
		idea: '#3a7bd5',
	};

	// Collect all entities from all types and filter by query
	let allEntities = $derived<EntitySummary[]>(
		Object.values(entityStore.entitiesByType).flat()
	);

	let filteredEntities = $derived.by<EntitySummary[]>(() => {
		if (!showAutocomplete) return [];
		const q = autocompleteQuery.toLowerCase();
		if (!q) return allEntities.slice(0, 20);
		return allEntities
			.filter((e) => e.title.toLowerCase().includes(q))
			.slice(0, 20);
	});

	function getIconForType(entityType: string): IconComponent {
		return entityIcons[entityType] ?? File;
	}

	function getColorForType(entityType: string): string | undefined {
		return entityColors[entityType];
	}

	function closeAutocomplete() {
		showAutocomplete = false;
		autocompleteQuery = '';
		selectedIndex = 0;
	}

	function getCaretRect(): { top: number; left: number } | null {
		const domSelection = window.getSelection();
		if (!domSelection || domSelection.rangeCount === 0) return null;
		const range = domSelection.getRangeAt(0);
		const rect = range.getBoundingClientRect();
		return { top: rect.bottom + 4, left: rect.left };
	}

	/**
	 * Detect `[[` pattern in the text before the cursor.
	 * Returns the partial query after `[[`, or null if no match.
	 */
	function detectWikiLinkTrigger(): { query: string; anchorOffset: number } | null {
		let result: { query: string; anchorOffset: number } | null = null;
		editor.getEditorState().read(() => {
			const selection = getSelection();
			if (!selection || !isRangeSelection(selection) || !selection.isCollapsed()) return;

			const anchor = selection.anchor;
			if (anchor.type !== 'text') return;

			const node = anchor.getNode();
			// Don't trigger inside existing wiki-link nodes
			if (isWikiLinkNode(node)) return;

			const textContent = node.getTextContent();
			const cursorOffset = anchor.offset;
			const textBeforeCursor = textContent.slice(0, cursorOffset);

			// Look for [[ that hasn't been closed yet
			const openBracketIndex = textBeforeCursor.lastIndexOf('[[');
			if (openBracketIndex === -1) return;

			// Make sure there's no ]] between [[ and cursor
			const textAfterOpen = textBeforeCursor.slice(openBracketIndex + 2);
			if (textAfterOpen.includes(']]')) return;

			result = {
				query: textAfterOpen,
				anchorOffset: openBracketIndex,
			};
		});
		return result;
	}

	function applyCompletion(entity: EntitySummary) {
		editor.update(() => {
			const selection = getSelection();
			if (!selection || !isRangeSelection(selection) || !selection.isCollapsed()) return;

			const anchor = selection.anchor;
			if (anchor.type !== 'text') return;

			const node = anchor.getNode() as TextNode;
			const textContent = node.getTextContent();
			const cursorOffset = anchor.offset;

			// Find the [[ before cursor
			const textBeforeCursor = textContent.slice(0, cursorOffset);
			const openBracketIndex = textBeforeCursor.lastIndexOf('[[');
			if (openBracketIndex === -1) return;

			const textAfterCursor = textContent.slice(cursorOffset);

			// Build the new text: everything before [[ + everything after cursor
			const before = textContent.slice(0, openBracketIndex);
			const after = textAfterCursor;

			// Replace the text node content with the before part
			if (before) {
				node.setTextContent(before);
			}

			// Create the WikiLinkNode
			const wikiLinkNode = createWikiLinkNode(entity.title);

			// Create after text node if needed
			const afterNode = after ? createTextNode(after) : null;

			if (before) {
				// Insert after the current text node
				node.insertAfter(wikiLinkNode);
				if (afterNode) {
					wikiLinkNode.insertAfter(afterNode);
				}
			} else {
				// Replace the node entirely
				node.replace(wikiLinkNode);
				if (afterNode) {
					wikiLinkNode.insertAfter(afterNode);
				}
			}

			// Move selection after the wiki link node
			if (afterNode) {
				afterNode.select(0, 0);
			} else {
				wikiLinkNode.selectNext();
			}
		});

		closeAutocomplete();
	}

	function handleItemClick(entity: EntitySummary) {
		applyCompletion(entity);
	}

	onMount(() => {
		// --- Click-to-navigate handler (existing) ---
		const removeClickListener = editor.registerCommand(
			CLICK_COMMAND,
			(event: MouseEvent) => {
				const target = event.target as HTMLElement;

				// Close autocomplete on click
				if (showAutocomplete) {
					closeAutocomplete();
				}

				// Early exit if the click is not on a wiki-link DOM element
				if (!target.closest('.editor-wiki-link')) return false;

				// Read the editor state to find the corresponding WikiLinkNode
				let handled = false;
				editor.getEditorState().read(() => {
					const selection = getSelection();
					if (!selection || !isRangeSelection(selection)) return;

					const anchorNode = selection.isCollapsed()
						? selection.anchor.getNode()
						: null;

					if (anchorNode && isWikiLinkNode(anchorNode)) {
						const linkTarget = anchorNode.getTarget();
						onNavigate?.(linkTarget);
						handled = true;
						return;
					}

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

		// --- Keyboard handling for autocomplete ---
		const removeKeyDownListener = editor.registerCommand(
			KEY_DOWN_COMMAND,
			(event: KeyboardEvent) => {
				if (!showAutocomplete) return false;

				const entities = filteredEntities;
				if (entities.length === 0) return false;

				if (event.key === 'ArrowDown') {
					event.preventDefault();
					selectedIndex = (selectedIndex + 1) % entities.length;
					scrollSelectedIntoView();
					return true;
				}

				if (event.key === 'ArrowUp') {
					event.preventDefault();
					selectedIndex = (selectedIndex - 1 + entities.length) % entities.length;
					scrollSelectedIntoView();
					return true;
				}

				if (event.key === 'Enter' || event.key === 'Tab') {
					event.preventDefault();
					const selected = entities[selectedIndex];
					if (selected) {
						applyCompletion(selected);
					}
					return true;
				}

				return false;
			},
			COMMAND_PRIORITY_HIGH,
		);

		const removeEscapeListener = editor.registerCommand(
			KEY_ESCAPE_COMMAND,
			(_event: KeyboardEvent) => {
				if (showAutocomplete) {
					closeAutocomplete();
					return true;
				}
				return false;
			},
			COMMAND_PRIORITY_HIGH,
		);

		// --- Text update listener for [[ detection ---
		const removeUpdateListener = editor.registerUpdateListener(
			({ editorState, dirtyElements, dirtyLeaves }) => {
				if (dirtyElements.size === 0 && dirtyLeaves.size === 0) return;

				const trigger = detectWikiLinkTrigger();
				if (trigger) {
					autocompleteQuery = trigger.query;
					autocompleteAnchorOffset = trigger.anchorOffset;

					const caretRect = getCaretRect();
					if (caretRect) {
						autocompletePosition = caretRect;
					}

					showAutocomplete = true;
					selectedIndex = 0;
				} else {
					if (showAutocomplete) {
						closeAutocomplete();
					}
				}
			}
		);

		return () => {
			removeClickListener();
			removeKeyDownListener();
			removeEscapeListener();
			removeUpdateListener();
		};
	});

	function scrollSelectedIntoView() {
		tick().then(() => {
			if (!dropdownEl) return;
			const selected = dropdownEl.querySelector('.wiki-autocomplete-item.selected');
			if (selected) {
				selected.scrollIntoView({ block: 'nearest' });
			}
		});
	}
</script>

{#if showAutocomplete}
	{@const entities = filteredEntities}
	{#if entities.length > 0}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="wiki-autocomplete"
			bind:this={dropdownEl}
			style="top: {autocompletePosition.top}px; left: {autocompletePosition.left}px;"
			onmousedown={(e) => e.preventDefault()}
		>
			{#each entities as entity, i (entity.slug + ':' + entity.schemaType)}
				{@const Icon = getIconForType(entity.schemaType)}
				{@const color = getColorForType(entity.schemaType)}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="wiki-autocomplete-item"
					class:selected={i === selectedIndex}
					onmousedown={(e) => { e.preventDefault(); handleItemClick(entity); }}
					onmouseenter={() => { selectedIndex = i; }}
				>
					<span class="wiki-autocomplete-icon" style:color={color}>
						<Icon size={14} />
					</span>
					<span class="wiki-autocomplete-title">{entity.title}</span>
					<span class="wiki-autocomplete-type" style:color={color}>
						{entity.schemaType}
					</span>
				</div>
			{/each}
		</div>
	{/if}
{/if}

<style>
	.wiki-autocomplete {
		position: fixed;
		z-index: 1000;
		min-width: 200px;
		max-width: 320px;
		max-height: 240px;
		overflow-y: auto;
		background: var(--bg-elevated, #fff);
		border: 1px solid var(--border-primary);
		border-radius: var(--radius-md);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		padding: var(--spacing-xs, 4px) 0;
	}

	.wiki-autocomplete-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs, 4px);
		padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
		cursor: pointer;
		font-size: var(--font-size-sm, 13px);
		transition: background-color 0.1s;
	}

	.wiki-autocomplete-item:hover,
	.wiki-autocomplete-item.selected {
		background: var(--bg-hover, #f0f0f0);
	}

	.wiki-autocomplete-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}

	.wiki-autocomplete-title {
		flex: 1;
		min-width: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		color: var(--text-primary);
		font-weight: var(--font-weight-medium, 500);
	}

	.wiki-autocomplete-type {
		flex-shrink: 0;
		font-size: var(--font-size-xs, 11px);
		opacity: 0.7;
		text-transform: capitalize;
	}
</style>
