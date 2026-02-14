<script lang="ts">
  import type { Snippet, ComponentType } from 'svelte';
  import { ChevronRight, Plus } from 'lucide-svelte';

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = ComponentType<any>;

  interface Props {
    title: string;
    icon: IconComponent;
    color?: string;
    count?: number;
    isOpen: boolean;
    onAdd?: () => void;
    ontoggle?: () => void;
    children: Snippet;
  }

  let {
    title,
    icon: Icon,
    color,
    count,
    isOpen = $bindable(),
    onAdd,
    ontoggle,
    children,
  }: Props = $props();

  function toggle() {
    if (ontoggle) {
      ontoggle();
    } else {
      isOpen = !isOpen;
    }
  }
</script>

<div class="section" class:open={isOpen}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="section-header" onclick={toggle} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggle(); } }} role="button" tabindex={0}>
    <span class="chevron" class:rotated={isOpen}>
      <ChevronRight size={14} />
    </span>
    <span class="section-icon" style:color={color}>
      <Icon size={14} />
    </span>
    <span class="section-title">{title}</span>
    {#if count !== undefined}
      <span class="section-count">{count}</span>
    {/if}
    {#if onAdd}
      <button
        class="section-add"
        type="button"
        onclick={(e) => { e.stopPropagation(); onAdd?.(); }}
        aria-label="Add {title}"
      >
        <Plus size={14} />
      </button>
    {/if}
  </div>
  {#if isOpen}
    <div class="section-content">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .section {
    margin-bottom: 2px;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-xs);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    cursor: pointer;
    transition: background-color var(--transition-fast);
    text-align: left;
  }

  .section-header:hover {
    background-color: var(--bg-tertiary);
  }

  .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: transform var(--transition-fast);
    color: var(--text-tertiary);
  }

  .chevron.rotated {
    transform: rotate(90deg);
  }

  .section-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .section-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .section-count {
    flex-shrink: 0;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-normal);
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    padding: 0 6px;
    min-width: 18px;
    text-align: center;
    line-height: 18px;
  }

  .section-add {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    opacity: 0;
    transition:
      opacity var(--transition-fast),
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .section-header:hover .section-add {
    opacity: 1;
  }

  .section-add:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .section-content {
    overflow: hidden;
  }
</style>
