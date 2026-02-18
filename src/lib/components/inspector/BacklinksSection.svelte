<script lang="ts">
  import { untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Users, MapPin, Package, Lightbulb, File, Link } from 'lucide-svelte';
  import type { ComponentType } from 'svelte';

  interface BacklinkResult {
    title: string;
    slug: string;
    fileType: string;
    entityType?: string;
    matchingLine: string;
    lineNumber: number;
  }

  interface Props {
    title: string;
    projectPath: string;
    onNavigate?: (slug: string, fileType: string, entityType?: string) => void;
  }

  let { title, projectPath, onNavigate }: Props = $props();

  let backlinks = $state<BacklinkResult[]>([]);
  let isLoading = $state(false);
  let hasLoaded = $state(false);

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

  function getIcon(result: BacklinkResult): IconComponent {
    if (result.entityType && entityIcons[result.entityType]) {
      return entityIcons[result.entityType];
    }
    return File;
  }

  function getColor(result: BacklinkResult): string | undefined {
    if (result.entityType && entityColors[result.entityType]) {
      return entityColors[result.entityType];
    }
    return undefined;
  }

  function getTypeLabel(result: BacklinkResult): string {
    if (result.entityType) {
      return result.entityType.charAt(0).toUpperCase() + result.entityType.slice(1);
    }
    return result.fileType.charAt(0).toUpperCase() + result.fileType.slice(1);
  }

  async function fetchBacklinks(searchTitle: string, path: string) {
    if (!searchTitle || !path) {
      backlinks = [];
      hasLoaded = true;
      return;
    }

    isLoading = true;
    try {
      backlinks = await invoke<BacklinkResult[]>('find_backlinks', {
        projectPath: path,
        title: searchTitle,
      });
    } catch {
      // On error, show empty state gracefully
      backlinks = [];
    } finally {
      isLoading = false;
      hasLoaded = true;
    }
  }

  function handleClick(result: BacklinkResult) {
    onNavigate?.(result.slug, result.fileType, result.entityType);
  }

  // Fetch backlinks when title or projectPath changes (Bug 5 fix: untrack async call)
  $effect(() => {
    const t = title;
    const p = projectPath;
    untrack(() => fetchBacklinks(t, p));
  });
</script>

<section class="backlinks-section">
  <header class="backlinks-header">
    <Link size={14} />
    <h4 class="backlinks-title">Referenced By</h4>
    {#if hasLoaded && !isLoading}
      <span class="backlinks-count">{backlinks.length}</span>
    {/if}
  </header>

  <div class="backlinks-body">
    {#if isLoading}
      <div class="backlinks-loading">
        <span class="loading-spinner"></span>
        <span>Searching references...</span>
      </div>
    {:else if backlinks.length === 0}
      <p class="backlinks-empty">No references found</p>
    {:else}
      <ul class="backlinks-list">
        {#each backlinks as result (result.slug + ':' + result.lineNumber)}
          {@const Icon = getIcon(result)}
          {@const color = getColor(result)}
          <li>
            <button
              class="backlink-item"
              onclick={() => handleClick(result)}
              title="{result.title} (line {result.lineNumber})"
            >
              <span class="backlink-icon" style:color={color}>
                <Icon size={14} />
              </span>
              <span class="backlink-info">
                <span class="backlink-title">{result.title}</span>
                <span class="backlink-type">{getTypeLabel(result)}</span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .backlinks-section {
    border-top: 1px solid var(--border-secondary);
    padding-top: var(--spacing-sm);
  }

  .backlinks-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 0 var(--spacing-sm) var(--spacing-xs);
  }

  .backlinks-header :global(svg) {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .backlinks-title {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    flex: 1;
  }

  .backlinks-count {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
    padding: 0 var(--spacing-xs);
    border-radius: var(--radius-full);
    font-variant-numeric: tabular-nums;
  }

  .backlinks-body {
    padding: 0 var(--spacing-xs);
  }

  .backlinks-loading {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm);
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .loading-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border-primary);
    border-top-color: var(--accent-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .backlinks-empty {
    padding: var(--spacing-sm);
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-style: italic;
  }

  .backlinks-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .backlink-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    text-align: left;
    transition: background-color var(--transition-fast);
  }

  .backlink-item:hover {
    background: var(--bg-hover);
  }

  .backlink-item:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: -2px;
  }

  .backlink-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .backlink-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .backlink-title {
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .backlink-type {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }
</style>
