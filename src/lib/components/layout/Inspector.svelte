<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

<aside class="inspector">
  <header class="inspector-header">
    <h3 class="inspector-title">Inspector</h3>
  </header>
  <div class="inspector-content">
    {#if children}
      {@render children()}
    {/if}
  </div>
</aside>

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    overflow: hidden;
  }

  .inspector-header {
    display: flex;
    align-items: center;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    flex-shrink: 0;
  }

  .inspector-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .inspector-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xs);
  }
</style>
