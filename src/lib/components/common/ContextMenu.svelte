<script lang="ts">
  import type { ComponentType } from 'svelte';

  interface MenuItem {
    label: string;
    icon?: ComponentType<any>;
    shortcut?: string;
    disabled?: boolean;
    separator?: boolean;
    onclick?: () => void;
  }

  interface Props {
    items: MenuItem[];
    x: number;
    y: number;
    onClose: () => void;
  }

  let { items, x, y, onClose }: Props = $props();

  let menuEl: HTMLElement | undefined = $state();

  // Adjust position to stay within viewport
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  $effect(() => {
    if (!menuEl) return;
    const rect = menuEl.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    adjustedX = x + rect.width > vw ? vw - rect.width - 8 : x;
    adjustedY = y + rect.height > vh ? vh - rect.height - 8 : y;
  });

  $effect(() => {
    if (!menuEl) return;

    // Focus the menu for keyboard navigation
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

    // Delay to avoid closing immediately from the same click event
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

  function handleItemClick(item: MenuItem) {
    if (item.disabled) return;
    item.onclick?.();
    onClose();
  }

  function handleItemKeydown(e: KeyboardEvent, item: MenuItem) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleItemClick(item);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={menuEl}
  class="context-menu"
  role="menu"
  tabindex="0"
  style="left: {adjustedX}px; top: {adjustedY}px;"
>
  {#each items as item}
    {#if item.separator}
      <div class="separator" role="separator"></div>
    {:else}
      <button
        class="menu-item"
        class:disabled={item.disabled}
        role="menuitem"
        aria-disabled={item.disabled ?? false}
        tabindex={item.disabled ? -1 : 0}
        onclick={() => handleItemClick(item)}
        onkeydown={(e) => handleItemKeydown(e, item)}
      >
        {#if item.icon}
          {@const Icon = item.icon}
          <span class="menu-icon">
            <Icon size={16} />
          </span>
        {/if}
        <span class="menu-label">{item.label}</span>
        {#if item.shortcut}
          <span class="menu-shortcut">{item.shortcut}</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 200;
    min-width: 180px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: var(--spacing-xs) 0;
    outline: none;
    animation: menu-appear var(--transition-fast) forwards;
  }

  @keyframes menu-appear {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: transparent;
    border-radius: 0;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: background-color var(--transition-fast);
  }

  .menu-item:hover:not(.disabled) {
    background: var(--bg-tertiary);
    box-shadow: none;
    border-color: transparent;
  }

  .menu-item:focus-visible {
    background: var(--bg-tertiary);
    outline: 2px solid var(--accent-primary);
    outline-offset: -2px;
    box-shadow: none;
  }

  .menu-item.disabled {
    color: var(--text-tertiary);
    cursor: default;
  }

  .menu-icon {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .menu-label {
    flex: 1;
  }

  .menu-shortcut {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    margin-left: var(--spacing-lg);
  }

  .separator {
    height: 1px;
    background: var(--border-secondary);
    margin: var(--spacing-xs) 0;
  }
</style>
