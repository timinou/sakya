<script lang="ts">
  import { untrack } from 'svelte';
  import { sessionsStore, projectState } from '$lib/stores';
  import CalendarHeatmap from './CalendarHeatmap.svelte';
  import SprintHistory from './SprintHistory.svelte';
  import { Flame, Award, PenTool, Clock, Calendar, TrendingUp, Trophy, Star } from 'lucide-svelte';

  // Load sessions when the component mounts.
  // IMPORTANT: untrack the loadSessions call to avoid tracking sessionsStore.$state
  // (isLoading), which would create an infinite $effect loop as isLoading toggles.
  $effect(() => {
    const path = projectState.projectPath;
    if (!path) return;
    untrack(() => sessionsStore.loadSessions(path));
  });

  /** Format total minutes as "Xh Ym" or just "Xm" */
  function formatTime(minutes: number): string {
    if (minutes < 1) return '0m';
    const hours = Math.floor(minutes / 60);
    const mins = Math.round(minutes % 60);
    if (hours === 0) return `${mins}m`;
    if (mins === 0) return `${hours}h`;
    return `${hours}h ${mins}m`;
  }

  /** Format a number with locale separators */
  function formatNumber(n: number): string {
    return Math.round(n).toLocaleString();
  }

  /** Format a date string as "Feb 14, 2026" */
  function formatDate(dateStr: string | undefined): string {
    if (!dateStr) return '--';
    const date = new Date(dateStr + 'T00:00:00');
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }
</script>

<div class="writing-stats">
  {#if sessionsStore.isLoading}
    <div class="stats-loading">
      <span class="loading-spinner"></span>
      <span>Loading statistics...</span>
    </div>
  {:else}
    <!-- Calendar Heatmap -->
    <section class="stats-section">
      <h2 class="section-title">Writing Activity</h2>
      <CalendarHeatmap dailyWordCounts={sessionsStore.dailyWordCounts} />
    </section>

    <!-- Stats Summary -->
    {#if sessionsStore.stats}
      {@const s = sessionsStore.stats}
      <section class="stats-section">
        <h2 class="section-title">Overview</h2>
        <div class="stats-grid">
          <div class="stat-card stat-card--streak">
            <div class="stat-icon"><Flame size={20} /></div>
            <div class="stat-value">{s.currentStreak}</div>
            <div class="stat-label">Current Streak</div>
            <div class="stat-unit">{s.currentStreak === 1 ? 'day' : 'days'}</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><Award size={20} /></div>
            <div class="stat-value">{s.longestStreak}</div>
            <div class="stat-label">Longest Streak</div>
            <div class="stat-unit">{s.longestStreak === 1 ? 'day' : 'days'}</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><PenTool size={20} /></div>
            <div class="stat-value">{formatNumber(s.totalWords)}</div>
            <div class="stat-label">Total Words</div>
            <div class="stat-unit">words</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><Clock size={20} /></div>
            <div class="stat-value">{formatTime(s.totalMinutes)}</div>
            <div class="stat-label">Total Time</div>
            <div class="stat-unit">spent writing</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><Calendar size={20} /></div>
            <div class="stat-value">{s.totalSessions}</div>
            <div class="stat-label">Sessions</div>
            <div class="stat-unit">{s.totalSessions === 1 ? 'session' : 'sessions'}</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><TrendingUp size={20} /></div>
            <div class="stat-value">{formatNumber(s.dailyAverage)}</div>
            <div class="stat-label">Daily Average</div>
            <div class="stat-unit">words / day</div>
          </div>

          <div class="stat-card">
            <div class="stat-icon"><Star size={20} /></div>
            <div class="stat-value">{formatNumber(s.weeklyAverage)}</div>
            <div class="stat-label">Weekly Average</div>
            <div class="stat-unit">words / week</div>
          </div>

          <div class="stat-card stat-card--best">
            <div class="stat-icon"><Trophy size={20} /></div>
            <div class="stat-value">{formatNumber(s.bestDayWords)}</div>
            <div class="stat-label">Best Day</div>
            <div class="stat-unit">{formatDate(s.bestDayDate)}</div>
          </div>
        </div>
      </section>
    {/if}

    <!-- Sprint History -->
    <section class="stats-section">
      <h2 class="section-title">Sprint History</h2>
      <SprintHistory sessions={sessionsStore.sessions} />
    </section>
  {/if}
</div>

<style>
  .writing-stats {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg);
    overflow-y: auto;
    height: 100%;
    max-width: 900px;
    margin: 0 auto;
  }

  .stats-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .section-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
    padding-bottom: var(--spacing-xs);
    border-bottom: 1px solid var(--border-secondary);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--spacing-md);
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-lg);
    text-align: center;
    transition: border-color var(--transition-fast), background-color var(--transition-fast);
  }

  .stat-card:hover {
    border-color: var(--border-primary);
    background: var(--bg-tertiary);
  }

  .stat-card--streak .stat-icon {
    color: var(--color-error);
  }

  .stat-card--best .stat-icon {
    color: var(--accent-primary);
  }

  .stat-icon {
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stat-value {
    font-size: var(--font-size-2xl, 1.5rem);
    font-weight: var(--font-weight-bold);
    color: var(--text-primary);
    line-height: 1.1;
  }

  .stat-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .stat-unit {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .stats-loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
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
</style>
