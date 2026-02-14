<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    children: Snippet;
  }

  let {
    text,
    position = 'top',
    children,
  }: Props = $props();

  let visible = $state(false);
  let showTimeout: ReturnType<typeof setTimeout> | undefined;

  function handleMouseEnter() {
    showTimeout = setTimeout(() => {
      visible = true;
    }, 200);
  }

  function handleMouseLeave() {
    if (showTimeout) {
      clearTimeout(showTimeout);
      showTimeout = undefined;
    }
    visible = false;
  }

  function handleFocusIn() {
    handleMouseEnter();
  }

  function handleFocusOut() {
    handleMouseLeave();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="tooltip-wrapper"
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
  onfocusin={handleFocusIn}
  onfocusout={handleFocusOut}
>
  {@render children()}
  {#if visible}
    <div
      class="tooltip tooltip-{position}"
      role="tooltip"
    >
      {text}
    </div>
  {/if}
</div>

<style>
  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
  }

  .tooltip {
    position: absolute;
    z-index: 150;
    background: var(--bg-inverse, #1a1614);
    color: var(--text-inverse);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
    pointer-events: none;
    animation: tooltip-fade-in var(--transition-fast) forwards;
  }

  @keyframes tooltip-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .tooltip-top {
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  .tooltip-bottom {
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  .tooltip-left {
    right: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }

  .tooltip-right {
    left: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }
</style>
