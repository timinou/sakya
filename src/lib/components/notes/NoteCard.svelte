<script lang="ts">
  import type { NoteEntry, CorkboardPosition } from '$lib/types';

  interface Props {
    note: NoteEntry;
    excerpt?: string;
    onDragEnd?: (slug: string, position: CorkboardPosition) => void;
    onColorChange?: (slug: string, color: string) => void;
    onLabelChange?: (slug: string, label: string) => void;
    onclick?: () => void;
  }

  let {
    note,
    excerpt = '',
    onDragEnd,
    onColorChange,
    onLabelChange,
    onclick,
  }: Props = $props();

  const DEFAULT_COLORS = ['#ef4444', '#f59e0b', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899', '#6b7280'];

  let isDragging = $state(false);
  let showColorPicker = $state(false);
  let showLabelEdit = $state(false);
  let labelInput = $state('');
  let labelInputEl = $state<HTMLInputElement | null>(null);

  // Drag state: track the offset from the pointer to the card's percentage position
  let dragStartPointerX = $state(0);
  let dragStartPointerY = $state(0);
  let dragStartPosX = $state(0);
  let dragStartPosY = $state(0);
  let currentX = $state(50);
  let currentY = $state(50);

  // Keep local position in sync with prop changes when not dragging
  $effect(() => {
    if (!isDragging) {
      currentX = note.position?.x ?? 50;
      currentY = note.position?.y ?? 50;
    }
  });

  $effect(() => {
    if (showLabelEdit && labelInputEl) {
      labelInputEl.focus();
      labelInputEl.select();
    }
  });

  function handlePointerDown(e: PointerEvent): void {
    // Ignore if clicking on interactive children (color picker, label editor)
    const target = e.target as HTMLElement;
    if (target.closest('.color-picker') || target.closest('.label-editor') || target.closest('.label-badge')) {
      return;
    }

    isDragging = true;
    dragStartPointerX = e.clientX;
    dragStartPointerY = e.clientY;
    dragStartPosX = currentX;
    dragStartPosY = currentY;

    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function handlePointerMove(e: PointerEvent): void {
    if (!isDragging) return;

    const card = (e.currentTarget as HTMLElement);
    const container = card.parentElement;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();
    const deltaX = e.clientX - dragStartPointerX;
    const deltaY = e.clientY - dragStartPointerY;

    // Convert pixel delta to percentage of container
    const deltaPercentX = (deltaX / containerRect.width) * 100;
    const deltaPercentY = (deltaY / containerRect.height) * 100;

    currentX = Math.max(0, Math.min(100, dragStartPosX + deltaPercentX));
    currentY = Math.max(0, Math.min(100, dragStartPosY + deltaPercentY));
  }

  function handlePointerUp(e: PointerEvent): void {
    if (!isDragging) return;

    isDragging = false;

    // Only fire callback if position actually changed
    const startX = note.position?.x ?? 50;
    const startY = note.position?.y ?? 50;
    if (Math.abs(currentX - startX) > 0.1 || Math.abs(currentY - startY) > 0.1) {
      onDragEnd?.(note.slug, { x: currentX, y: currentY });
    }
  }

  function handleClick(e: MouseEvent): void {
    // Only fire click if not dragging (prevent click after drag)
    const target = e.target as HTMLElement;
    if (target.closest('.color-picker') || target.closest('.label-editor') || target.closest('.label-badge')) {
      return;
    }
    onclick?.();
  }

  function handleColorSelect(color: string): void {
    onColorChange?.(note.slug, color);
    showColorPicker = false;
  }

  function toggleColorPicker(e: MouseEvent): void {
    e.stopPropagation();
    showColorPicker = !showColorPicker;
    showLabelEdit = false;
  }

  function handleLabelClick(e: MouseEvent): void {
    e.stopPropagation();
    labelInput = note.label ?? '';
    showLabelEdit = true;
    showColorPicker = false;
  }

  function confirmLabel(): void {
    const trimmed = labelInput.trim();
    onLabelChange?.(note.slug, trimmed);
    showLabelEdit = false;
  }

  function handleLabelKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmLabel();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      showLabelEdit = false;
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onclick?.();
    }
  }
</script>

<div
  class="note-card"
  class:dragging={isDragging}
  style:left="{currentX}%"
  style:top="{currentY}%"
  style:--card-color={note.color ?? 'var(--border-secondary)'}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="button"
  tabindex="0"
>
  <!-- Color strip -->
  <div class="color-strip"></div>

  <div class="card-body">
    <!-- Header with title and label -->
    <div class="card-header">
      <span class="card-title">{note.title}</span>
      {#if note.label && !showLabelEdit}
        <button
          class="label-badge"
          type="button"
          onclick={handleLabelClick}
          title="Edit label"
        >
          {note.label}
        </button>
      {/if}
    </div>

    <!-- Excerpt -->
    {#if excerpt}
      <p class="card-excerpt">{excerpt}</p>
    {/if}

    <!-- Footer actions -->
    <div class="card-footer">
      <button
        class="action-btn"
        type="button"
        onclick={toggleColorPicker}
        title="Change color"
      >
        <span class="color-indicator" style:background-color={note.color ?? 'var(--text-tertiary)'}></span>
      </button>
      <button
        class="action-btn"
        type="button"
        onclick={handleLabelClick}
        title={note.label ? 'Edit label' : 'Add label'}
      >
        <span class="label-icon">T</span>
      </button>
    </div>

    <!-- Color picker dropdown -->
    {#if showColorPicker}
      <div class="color-picker" role="listbox" aria-label="Choose card color">
        {#each DEFAULT_COLORS as color}
          <button
            class="color-swatch"
            class:selected={note.color === color}
            style:background-color={color}
            type="button"
            onclick={() => handleColorSelect(color)}
            role="option"
            aria-selected={note.color === color}
            aria-label="Color {color}"
          ></button>
        {/each}
      </div>
    {/if}

    <!-- Label editor -->
    {#if showLabelEdit}
      <div class="label-editor">
        <input
          bind:this={labelInputEl}
          bind:value={labelInput}
          class="label-input"
          type="text"
          placeholder="Label..."
          onkeydown={handleLabelKeydown}
          onblur={confirmLabel}
        />
      </div>
    {/if}
  </div>
</div>

<style>
  .note-card {
    position: absolute;
    transform: translate(-50%, -50%);
    min-width: 180px;
    max-width: 260px;
    min-height: 120px;
    display: flex;
    background: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    cursor: grab;
    user-select: none;
    transition: box-shadow var(--transition-fast);
    touch-action: none;
  }

  .note-card:hover {
    box-shadow: var(--shadow-lg, 0 4px 12px rgba(0, 0, 0, 0.2));
  }

  .note-card.dragging {
    cursor: grabbing;
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0, 0, 0, 0.3));
    z-index: 10;
  }

  .color-strip {
    width: 4px;
    flex-shrink: 0;
    background-color: var(--card-color);
    border-radius: var(--radius-md) 0 0 var(--radius-md);
  }

  .card-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-sm);
    min-width: 0;
    position: relative;
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-xs);
  }

  .card-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .label-badge {
    flex-shrink: 0;
    padding: 1px 6px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-full);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition: background-color var(--transition-fast);
  }

  .label-badge:hover {
    background: var(--bg-secondary, var(--bg-tertiary));
  }

  .card-excerpt {
    flex: 1;
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    line-height: 1.4;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    margin: 0;
  }

  .card-footer {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-xs);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .note-card:hover .card-footer {
    opacity: 1;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .color-indicator {
    display: block;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
  }

  .label-icon {
    font-size: 11px;
    font-weight: var(--font-weight-semibold);
  }

  .color-picker {
    position: absolute;
    bottom: -4px;
    left: var(--spacing-sm);
    transform: translateY(100%);
    display: flex;
    gap: 4px;
    padding: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: 20;
  }

  .color-swatch {
    width: 20px;
    height: 20px;
    border: 2px solid transparent;
    border-radius: var(--radius-full);
    cursor: pointer;
    transition: border-color var(--transition-fast), transform var(--transition-fast);
  }

  .color-swatch:hover {
    transform: scale(1.15);
  }

  .color-swatch.selected {
    border-color: var(--text-primary);
  }

  .label-editor {
    position: absolute;
    bottom: -4px;
    left: var(--spacing-sm);
    right: var(--spacing-sm);
    transform: translateY(100%);
    z-index: 20;
  }

  .label-input {
    width: 100%;
    padding: 4px var(--spacing-xs);
    border: 1px solid var(--border-primary, #555);
    border-radius: var(--radius-sm);
    background: var(--bg-primary, #1e1e1e);
    color: var(--text-primary);
    font-size: var(--font-size-xs);
    font-family: inherit;
    outline: none;
    box-shadow: var(--shadow-md);
  }

  .label-input:focus {
    border-color: var(--accent-primary, #7c4dbd);
  }

  .label-input::placeholder {
    color: var(--text-tertiary);
  }
</style>
