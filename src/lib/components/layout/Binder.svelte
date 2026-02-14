<script lang="ts">
  import type { Snippet } from 'svelte';
  import { BinderTree } from '$lib/components/binder';

  interface Props {
    onSelectEntity?: (schemaType: string, slug: string) => void;
    onCreateEntity?: (schemaType: string) => void;
    onSelectChapter?: (slug: string) => void;
    children?: Snippet;
  }

  let {
    onSelectEntity,
    onCreateEntity,
    onSelectChapter,
    children,
  }: Props = $props();
</script>

<aside class="binder">
  <header class="binder-header">
    <h3 class="binder-title">Binder</h3>
  </header>
  <div class="binder-content">
    <BinderTree
      {onSelectEntity}
      {onCreateEntity}
      {onSelectChapter}
    />
    {#if children}
      {@render children()}
    {/if}
  </div>
</aside>

<style>
  .binder {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    overflow: hidden;
  }

  .binder-header {
    display: flex;
    align-items: center;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    flex-shrink: 0;
  }

  .binder-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .binder-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xs);
  }
</style>
