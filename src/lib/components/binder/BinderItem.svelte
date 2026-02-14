<script lang="ts">
  import type { ComponentType } from 'svelte';
  import { File } from 'lucide-svelte';

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = ComponentType<any>;

  interface Props {
    label: string;
    icon?: IconComponent;
    color?: string;
    isSelected?: boolean;
    isActive?: boolean;
    onclick?: () => void;
    oncontextmenu?: (e: MouseEvent) => void;
    indent?: number;
  }

  let {
    label,
    icon: Icon = File,
    color,
    isSelected = false,
    isActive = false,
    onclick,
    oncontextmenu,
    indent = 0,
  }: Props = $props();

  let paddingLeft = $derived(`${8 + indent * 16}px`);
</script>

<button
  class="item"
  class:selected={isSelected}
  class:active={isActive}
  style:padding-left={paddingLeft}
  {onclick}
  {oncontextmenu}
  type="button"
  title={label}
>
  <span class="item-icon" style:color={color}>
    {#if Icon}
      <Icon size={16} />
    {/if}
  </span>
  <span class="item-label">{label}</span>
</button>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: 3px var(--spacing-xs);
    padding-right: var(--spacing-sm);
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast);
    text-align: left;
    min-height: 28px;
  }

  .item:hover {
    background-color: var(--bg-tertiary);
  }

  .item.selected {
    background-color: var(--bg-tertiary);
    font-weight: var(--font-weight-medium);
  }

  .item.active {
    border-left-color: var(--accent-primary);
    background-color: var(--bg-tertiary);
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-tertiary);
  }

  .item-icon[style*="color"] {
    color: unset;
  }

  .item-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
