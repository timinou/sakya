<script lang="ts">
  interface Props {
    dailyWordCounts: Map<string, number>; // "YYYY-MM-DD" -> word count
  }

  let { dailyWordCounts }: Props = $props();

  // Tooltip state
  let tooltipVisible = $state(false);
  let tooltipText = $state('');
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  // Cell sizing constants
  const cellSize = 13;
  const cellGap = 3;
  const step = cellSize + cellGap;
  const dayLabelWidth = 32;
  const monthLabelHeight = 20;
  const weeks = 52;
  const daysPerWeek = 7;

  // Day labels shown on the left (Mon, Wed, Fri — rows 0,2,4 which map to Mon, Wed, Fri)
  const dayLabels: { label: string; row: number }[] = [
    { label: 'Mon', row: 0 },
    { label: 'Wed', row: 2 },
    { label: 'Fri', row: 4 },
  ];

  // SVG dimensions
  let svgWidth = $derived(dayLabelWidth + weeks * step + cellGap);
  let svgHeight = $derived(monthLabelHeight + daysPerWeek * step + cellGap);

  // Intensity level: 0-4
  function getIntensityLevel(count: number): number {
    if (count === 0) return 0;
    if (count < 250) return 1;
    if (count < 500) return 2;
    if (count < 1000) return 3;
    return 4;
  }

  // Format a date for the tooltip: "Feb 14, 2026"
  function formatDate(date: Date): string {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }

  // Format a date to "YYYY-MM-DD" for map lookup
  function toDateKey(date: Date): string {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, '0');
    const d = String(date.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
  }

  // Build grid data: 52 columns x 7 rows, going back from today
  interface CellData {
    date: Date;
    dateKey: string;
    count: number;
    level: number;
    col: number;
    row: number;
    x: number;
    y: number;
  }

  let gridCells = $derived.by(() => {
    const cells: CellData[] = [];
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    // Find the Monday of the current week
    const todayDow = today.getDay(); // 0=Sun, 1=Mon, ...
    // Convert to Mon=0, Tue=1, ..., Sun=6
    const mondayOffset = todayDow === 0 ? 6 : todayDow - 1;

    // The last day in our grid is the end of the current week (Sunday)
    const endOfWeek = new Date(today);
    endOfWeek.setDate(today.getDate() + (6 - mondayOffset));

    // Start date is 51 weeks before the Monday of the current week
    const startDate = new Date(endOfWeek);
    startDate.setDate(endOfWeek.getDate() - (weeks * 7 - 1));

    for (let col = 0; col < weeks; col++) {
      for (let row = 0; row < daysPerWeek; row++) {
        const dayOffset = col * 7 + row;
        const cellDate = new Date(startDate);
        cellDate.setDate(startDate.getDate() + dayOffset);

        // Skip future dates
        if (cellDate > today) continue;

        const dateKey = toDateKey(cellDate);
        const count = dailyWordCounts.get(dateKey) ?? 0;

        cells.push({
          date: new Date(cellDate),
          dateKey,
          count,
          level: getIntensityLevel(count),
          col,
          row,
          x: dayLabelWidth + col * step,
          y: monthLabelHeight + row * step,
        });
      }
    }
    return cells;
  });

  // Month labels: compute which columns correspond to the start of each month
  interface MonthLabel {
    label: string;
    x: number;
  }

  let monthLabels = $derived.by(() => {
    const labels: MonthLabel[] = [];
    const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
                        'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

    if (gridCells.length === 0) return labels;

    let lastMonth = -1;

    for (const cell of gridCells) {
      if (cell.row !== 0) continue; // Only check first row (Monday)
      const month = cell.date.getMonth();
      if (month !== lastMonth) {
        labels.push({
          label: monthNames[month],
          x: cell.x,
        });
        lastMonth = month;
      }
    }

    return labels;
  });

  function handleCellEnter(e: MouseEvent, cell: CellData): void {
    const formatted = formatDate(cell.date);
    const wordText = cell.count === 1 ? 'word' : 'words';
    tooltipText = `${formatted}: ${cell.count.toLocaleString()} ${wordText}`;
    tooltipVisible = true;

    // Position tooltip relative to the container
    const container = (e.currentTarget as HTMLElement).closest('.calendar-heatmap');
    if (container) {
      const rect = container.getBoundingClientRect();
      tooltipX = e.clientX - rect.left;
      tooltipY = e.clientY - rect.top - 32;
    }
  }

  function handleCellMove(e: MouseEvent): void {
    if (!tooltipVisible) return;
    const container = (e.currentTarget as HTMLElement).closest('.calendar-heatmap');
    if (container) {
      const rect = container.getBoundingClientRect();
      tooltipX = e.clientX - rect.left;
      tooltipY = e.clientY - rect.top - 32;
    }
  }

  function handleCellLeave(): void {
    tooltipVisible = false;
  }
</script>

<div class="calendar-heatmap">
  <div class="heatmap-scroll">
    <svg
      width={svgWidth}
      height={svgHeight}
      viewBox="0 0 {svgWidth} {svgHeight}"
      role="img"
      aria-label="Calendar heatmap showing daily word counts over the past year"
    >
      <!-- Month labels along the top -->
      {#each monthLabels as month}
        <text
          x={month.x}
          y={12}
          class="month-label"
        >{month.label}</text>
      {/each}

      <!-- Day-of-week labels on the left -->
      {#each dayLabels as day}
        <text
          x={dayLabelWidth - 6}
          y={monthLabelHeight + day.row * step + cellSize / 2 + 1}
          class="day-label"
        >{day.label}</text>
      {/each}

      <!-- Day cells -->
      {#each gridCells as cell (cell.dateKey)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <rect
          x={cell.x}
          y={cell.y}
          width={cellSize}
          height={cellSize}
          rx={2}
          ry={2}
          class="day-cell level-{cell.level}"
          onmouseenter={(e) => handleCellEnter(e, cell)}
          onmousemove={handleCellMove}
          onmouseleave={handleCellLeave}
        />
      {/each}
    </svg>
  </div>

  {#if tooltipVisible}
    <div
      class="heatmap-tooltip"
      style="left: {tooltipX}px; top: {tooltipY}px;"
      role="tooltip"
    >
      {tooltipText}
    </div>
  {/if}
</div>

<style>
  .calendar-heatmap {
    position: relative;
    width: 100%;
  }

  .heatmap-scroll {
    overflow-x: auto;
    overflow-y: hidden;
    padding-bottom: var(--spacing-xs);
  }

  svg {
    display: block;
  }

  .month-label {
    fill: var(--text-tertiary);
    font-size: 10px;
    font-family: inherit;
    user-select: none;
  }

  .day-label {
    fill: var(--text-tertiary);
    font-size: 10px;
    font-family: inherit;
    text-anchor: end;
    dominant-baseline: central;
    user-select: none;
  }

  .day-cell {
    transition: opacity var(--transition-fast);
    cursor: pointer;
    stroke: var(--border-secondary);
    stroke-width: 0.5;
  }

  .day-cell:hover {
    stroke: var(--text-secondary);
    stroke-width: 1.5;
  }

  /* Intensity levels using accent-primary with varying opacity */
  .day-cell.level-0 {
    fill: var(--bg-tertiary);
  }

  .day-cell.level-1 {
    fill: var(--accent-primary);
    opacity: 0.3;
  }

  .day-cell.level-2 {
    fill: var(--accent-primary);
    opacity: 0.5;
  }

  .day-cell.level-3 {
    fill: var(--accent-primary);
    opacity: 0.75;
  }

  .day-cell.level-4 {
    fill: var(--accent-primary);
    opacity: 1;
  }

  .heatmap-tooltip {
    position: absolute;
    z-index: 150;
    background: var(--bg-inverse, #1a1614);
    color: var(--text-inverse);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium, 500);
    white-space: nowrap;
    pointer-events: none;
    transform: translateX(-50%);
    animation: tooltip-fade-in var(--transition-fast) forwards;
  }

  @keyframes tooltip-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
