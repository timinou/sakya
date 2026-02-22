<script lang="ts">
  import type { EntitySchema } from '$lib/types/entity';
  import { SCHEMA_TEMPLATES } from '$lib/data/schema-templates';
  import { entityStore } from '$lib/stores';
  import { FilePlus } from 'lucide-svelte';

  interface Props {
    x: number;
    y: number;
    onSelect: (template: EntitySchema | null) => void;
    onClose: () => void;
  }

  let { x, y, onSelect, onClose }: Props = $props();

  let menuEl: HTMLElement | undefined = $state();
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  // Filter out templates whose entityType already exists in the project
  let availableTemplates = $derived.by(() => {
    const existingTypes = new Set(entityStore.schemaSummaries.map((s) => s.entityType));
    return SCHEMA_TEMPLATES.filter((t) => !existingTypes.has(t.entityType));
  });

  // Viewport adjustment (same pattern as ContextMenu)
  $effect(() => {
    if (!menuEl) return;
    const rect = menuEl.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    adjustedX = x + rect.width > vw ? vw - rect.width - 8 : x;
    adjustedY = y + rect.height > vh ? vh - rect.height - 8 : y;
  });

  // Click-outside and Escape close
  $effect(() => {
    if (!menuEl) return;
    menuEl.focus();

    function handleClickOutside(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) {
        onClose();
      }
    }

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    }

    const timeout = setTimeout(() => {
      document.addEventListener('click', handleClickOutside);
    }, 0);
    document.addEventListener('keydown', handleKeydown);

    return () => {
      clearTimeout(timeout);
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleKeydown);
    };
  });

  function selectBlank() {
    onSelect(null);
  }

  function selectTemplate(template: EntitySchema) {
    onSelect(template);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={menuEl}
  class="template-picker"
  role="menu"
  tabindex="0"
  style="left: {adjustedX}px; top: {adjustedY}px;"
>
  <div class="picker-header">New Entity Type</div>

  <button
    class="picker-item blank"
    role="menuitem"
    tabindex="0"
    onclick={selectBlank}
  >
    <span class="picker-icon blank-icon">
      <FilePlus size={16} />
    </span>
    <span class="picker-text">
      <span class="picker-name">Blank</span>
      <span class="picker-desc">Start from scratch</span>
    </span>
  </button>

  {#if availableTemplates.length > 0}
    <div class="separator" role="separator"></div>

    {#each availableTemplates as template (template.entityType)}
      <button
        class="picker-item"
        role="menuitem"
        tabindex="0"
        onclick={() => selectTemplate(template)}
      >
        <span class="picker-dot" style="background: {template.color}"></span>
        <span class="picker-text">
          <span class="picker-name">{template.name}</span>
          <span class="picker-desc">{template.description}</span>
        </span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .template-picker {
    position: fixed;
    z-index: 200;
    min-width: 260px;
    max-width: 320px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: var(--spacing-xs) 0;
    outline: none;
    animation: picker-appear var(--transition-fast) forwards;
  }

  @keyframes picker-appear {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .picker-header {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .picker-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: transparent;
    border-radius: 0;
    cursor: pointer;
    text-align: left;
    transition: background-color var(--transition-fast);
  }

  .picker-item:hover {
    background: var(--bg-tertiary);
    box-shadow: none;
    border-color: transparent;
  }

  .picker-item:focus-visible {
    background: var(--bg-tertiary);
    outline: 2px solid var(--accent-primary);
    outline-offset: -2px;
    box-shadow: none;
  }

  .picker-item.blank {
    color: var(--text-secondary);
  }

  .blank-icon {
    display: flex;
    align-items: center;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .picker-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .picker-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .picker-name {
    font-size: var(--font-size-base);
    color: var(--text-primary);
    white-space: nowrap;
  }

  .picker-desc {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .separator {
    height: 1px;
    background: var(--border-secondary);
    margin: var(--spacing-xs) 0;
  }
</style>
