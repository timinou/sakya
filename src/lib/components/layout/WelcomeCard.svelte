<script lang="ts">
  import { PenLine, Users, StickyNote } from 'lucide-svelte';
  import { entityStore } from '$lib/stores';

  interface Props {
    onCreateChapter?: () => void;
    onCreateNote?: () => void;
    onCreateEntity?: (entityType: string) => void;
  }

  let {
    onCreateChapter,
    onCreateNote,
    onCreateEntity,
  }: Props = $props();

  // Build ghost buttons from loaded entity schemas (fall back to defaults)
  let entityButtons = $derived(
    entityStore.schemaSummaries.length > 0
      ? entityStore.schemaSummaries.map((s) => ({
          type: s.entityType,
          label: s.name,
        }))
      : [
          { type: 'character', label: 'Character' },
          { type: 'place', label: 'Place' },
          { type: 'idea', label: 'Idea' },
        ]
  );

  // Use the first 3 entity types for the ghost buttons
  let visibleEntityButtons = $derived(entityButtons.slice(0, 3));
</script>

<div class="welcome-card">
  <div class="welcome-ornament" aria-hidden="true"></div>

  <h2 class="welcome-heading">Begin</h2>

  <p class="welcome-subtitle">
    Your story starts with a single chapter.<br />
    Or set the stage — sketch a character,<br />
    map a place, pin an idea.
  </p>

  <button
    class="welcome-primary-cta"
    type="button"
    onclick={() => onCreateChapter?.()}
  >
    <PenLine size={16} />
    Write first chapter
  </button>

  <div class="welcome-ghost-row">
    {#each visibleEntityButtons as btn (btn.type)}
      <button
        class="welcome-ghost-btn"
        type="button"
        data-entity-type={btn.type}
        onclick={() => onCreateEntity?.(btn.type)}
      >
        {#if btn.type === 'character'}
          <Users size={14} />
        {:else}
          <StickyNote size={14} />
        {/if}
        {btn.label}
      </button>
    {/each}
    <button
      class="welcome-ghost-btn"
      type="button"
      onclick={() => onCreateNote?.()}
    >
      <StickyNote size={14} />
      Note
    </button>
  </div>

  <div class="welcome-hints">
    <kbd>⌘K</kbd> search · <kbd>⌘\</kbd> sidebar
  </div>
</div>

<style>
  .welcome-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--spacing-lg);
    padding: var(--spacing-2xl);
    animation: fadeIn 600ms ease both;
  }

  .welcome-ornament {
    position: relative;
    width: 120px;
    height: 1px;
    background: var(--text-tertiary);
    opacity: 0.3;
    animation: fadeSlideUp 600ms ease both;
  }

  .welcome-ornament::after {
    content: '◆';
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-primary);
    padding: 0 8px;
    font-size: 8px;
    color: var(--text-tertiary);
    opacity: 0.6;
  }

  .welcome-heading {
    font-family: var(--font-serif);
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-normal);
    letter-spacing: 0.04em;
    color: var(--text-primary);
    animation: fadeSlideUp 600ms ease both;
    animation-delay: 200ms;
  }

  .welcome-subtitle {
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    text-align: center;
    line-height: 1.7;
    max-width: 320px;
    animation: fadeSlideUp 800ms ease both;
    animation-delay: 400ms;
  }

  .welcome-primary-cta {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 10px 24px;
    background: var(--accent-primary);
    color: var(--text-inverse);
    border: none;
    border-radius: var(--radius-lg);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    box-shadow: var(--shadow-sm);
    transition:
      transform var(--transition-spring),
      box-shadow var(--transition-fast);
    animation: fadeSlideUp 600ms ease both;
    animation-delay: 600ms;
  }

  .welcome-primary-cta:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .welcome-primary-cta:active {
    transform: translateY(0);
  }

  .welcome-ghost-row {
    display: flex;
    gap: var(--spacing-sm);
    animation: fadeSlideUp 600ms ease both;
    animation-delay: 800ms;
  }

  .welcome-ghost-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      border-color var(--transition-fast),
      color var(--transition-fast),
      background-color var(--transition-fast);
  }

  .welcome-ghost-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .welcome-ghost-btn[data-entity-type="character"]:hover {
    border-color: var(--color-entity-character);
    color: var(--color-entity-character);
    background: color-mix(in srgb, var(--color-entity-character) 6%, transparent);
  }

  .welcome-ghost-btn[data-entity-type="place"]:hover {
    border-color: var(--color-entity-place);
    color: var(--color-entity-place);
    background: color-mix(in srgb, var(--color-entity-place) 6%, transparent);
  }

  .welcome-ghost-btn[data-entity-type="idea"]:hover {
    border-color: var(--color-entity-idea);
    color: var(--color-entity-idea);
    background: color-mix(in srgb, var(--color-entity-idea) 6%, transparent);
  }

  .welcome-hints {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    animation: fadeSlideUp 600ms ease both;
    animation-delay: 1000ms;
  }

  .welcome-hints kbd {
    display: inline-block;
    padding: 1px 5px;
    font-size: 0.6875rem;
    font-family: var(--font-sans);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    line-height: 1.4;
  }

  @keyframes fadeSlideUp {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
