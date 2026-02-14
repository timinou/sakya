<script lang="ts">
  import type { WritingSession } from '$lib/types';

  interface Props {
    sessions: WritingSession[];
    chapterNames?: Map<string, string>;
  }

  let { sessions, chapterNames }: Props = $props();

  const PAGE_SIZE = 20;

  let visibleCount = $state(PAGE_SIZE);

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });

  let sortedSessions = $derived(
    [...sessions].sort((a, b) => new Date(b.start).getTime() - new Date(a.start).getTime()),
  );

  let visibleSessions = $derived(sortedSessions.slice(0, visibleCount));

  let hasMore = $derived(visibleCount < sortedSessions.length);

  function formatDate(iso: string): string {
    const date = new Date(iso);
    return dateFormatter.format(date);
  }

  function formatDuration(minutes: number | undefined): string {
    if (minutes == null) return '—';
    if (minutes < 1) return '< 1 min';
    return `${Math.round(minutes)} min`;
  }

  function resolveChapterName(slug: string): string {
    return chapterNames?.get(slug) ?? slug;
  }

  function goalMet(session: WritingSession): boolean | null {
    if (session.sprintGoal == null) return null;
    return session.wordsWritten >= session.sprintGoal;
  }

  function showMore(): void {
    visibleCount += PAGE_SIZE;
  }
</script>

<div class="sprint-history">
  {#if sessions.length === 0}
    <p class="empty-state">No sprints yet. Start your first writing sprint!</p>
  {:else}
    <ul class="sprint-list">
      {#each visibleSessions as session (session.id)}
        {@const met = goalMet(session)}
        <li class="sprint-entry">
          <div class="sprint-header">
            <span class="sprint-date">{formatDate(session.start)}</span>
            <span class="sprint-chapter">{resolveChapterName(session.chapterSlug)}</span>
          </div>
          <div class="sprint-details">
            <span class="sprint-duration">{formatDuration(session.durationMinutes)}</span>
            <span class="sprint-words">{session.wordsWritten.toLocaleString()} words</span>
            {#if met !== null}
              <span class="sprint-goal" class:goal-met={met} class:goal-missed={!met}>
                {#if met}
                  <svg class="goal-icon" viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z" fill="currentColor"/>
                  </svg>
                {:else}
                  <svg class="goal-icon" viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.75.75 0 111.06 1.06L9.06 8l3.22 3.22a.75.75 0 11-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 01-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 010-1.06z" fill="currentColor"/>
                  </svg>
                {/if}
                <span class="goal-target">{session.sprintGoal} goal</span>
              </span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>

    {#if hasMore}
      <button class="show-more-btn" type="button" onclick={showMore}>
        Show more
      </button>
    {/if}
  {/if}
</div>

<style>
  .sprint-history {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .empty-state {
    text-align: center;
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
    font-style: italic;
    padding: var(--spacing-xl) var(--spacing-md);
  }

  .sprint-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .sprint-entry {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    transition: background-color var(--transition-fast);
  }

  .sprint-entry:hover {
    background: var(--bg-tertiary);
  }

  .sprint-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
  }

  .sprint-date {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .sprint-chapter {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
    max-width: 40%;
    text-align: right;
  }

  .sprint-details {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
  }

  .sprint-duration {
    min-width: 4em;
  }

  .sprint-words {
    font-weight: var(--font-weight-medium);
  }

  .sprint-goal {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: auto;
  }

  .sprint-goal.goal-met {
    color: var(--color-success);
  }

  .sprint-goal.goal-missed {
    color: var(--color-error);
  }

  .goal-icon {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .goal-target {
    font-size: var(--font-size-xs);
  }

  .show-more-btn {
    align-self: center;
    padding: var(--spacing-xs) var(--spacing-lg);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      border-color var(--transition-fast),
      background-color var(--transition-fast);
  }

  .show-more-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-primary);
    background: var(--bg-tertiary);
  }
</style>
