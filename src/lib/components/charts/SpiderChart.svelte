<script lang="ts">
  import type { SpiderAxis } from '$lib/types';

  interface Props {
    axes: SpiderAxis[];
    values: Record<string, number>;
    onChange?: (values: Record<string, number>) => void;
    color?: string;
    size?: number;
    readonly?: boolean;
  }

  let { axes, values, onChange, color = '#7c4dbd', size = 300, readonly = false }: Props = $props();

  const padding = 40;
  const labelOffset = 18;
  const gridLevels = 4;
  const pointRadius = 6;
  const pointHitRadius = 12;

  let radius = $derived((size - padding * 2) / 2);
  let cx = $derived(size / 2);
  let cy = $derived(size / 2);
  let angleStep = $derived(axes.length > 0 ? (2 * Math.PI) / axes.length : 0);

  let draggingIndex = $state<number | null>(null);
  let hoveredIndex = $state<number | null>(null);
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  function angleFor(i: number): number {
    return i * angleStep - Math.PI / 2;
  }

  function axisEndpoint(i: number): { x: number; y: number } {
    const angle = angleFor(i);
    return {
      x: cx + radius * Math.cos(angle),
      y: cy + radius * Math.sin(angle),
    };
  }

  function valuePoint(i: number): { x: number; y: number } {
    const axis = axes[i];
    const val = values[axis.name] ?? axis.default;
    const ratio = (val - axis.min) / (axis.max - axis.min);
    const clampedRatio = Math.max(0, Math.min(1, ratio));
    const angle = angleFor(i);
    return {
      x: cx + clampedRatio * radius * Math.cos(angle),
      y: cy + clampedRatio * radius * Math.sin(angle),
    };
  }

  function labelPosition(i: number): { x: number; y: number; anchor: string } {
    const angle = angleFor(i);
    const dist = radius + labelOffset;
    const x = cx + dist * Math.cos(angle);
    const y = cy + dist * Math.sin(angle);
    const cos = Math.cos(angle);
    let anchor = 'middle';
    if (cos > 0.1) anchor = 'start';
    else if (cos < -0.1) anchor = 'end';
    return { x, y, anchor };
  }

  let gridPolygons = $derived(
    Array.from({ length: gridLevels }, (_, level) => {
      const ratio = (level + 1) / gridLevels;
      return axes
        .map((_, i) => {
          const angle = angleFor(i);
          const x = cx + ratio * radius * Math.cos(angle);
          const y = cy + ratio * radius * Math.sin(angle);
          return `${x},${y}`;
        })
        .join(' ');
    }),
  );

  let valuePolygonPoints = $derived(axes.map((_, i) => valuePoint(i)));

  let valuePolygon = $derived(valuePolygonPoints.map((p) => `${p.x},${p.y}`).join(' '));

  function clampValue(axis: SpiderAxis, val: number): number {
    return Math.round(Math.max(axis.min, Math.min(axis.max, val)));
  }

  function handlePointerDown(e: PointerEvent, index: number) {
    if (readonly) return;
    draggingIndex = index;
    (e.currentTarget as SVGElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function handlePointerMove(e: PointerEvent) {
    if (draggingIndex === null) return;

    const svg = (e.currentTarget as SVGElement).closest('svg');
    if (!svg) return;

    const pt = svg.createSVGPoint();
    pt.x = e.clientX;
    pt.y = e.clientY;
    const svgPt = pt.matrixTransform(svg.getScreenCTM()!.inverse());

    const dx = svgPt.x - cx;
    const dy = svgPt.y - cy;

    const angle = angleFor(draggingIndex);
    const axisX = Math.cos(angle);
    const axisY = Math.sin(angle);

    // Project pointer position onto axis direction
    const projection = dx * axisX + dy * axisY;
    const ratio = Math.max(0, Math.min(1, projection / radius));

    const axis = axes[draggingIndex];
    const newVal = clampValue(axis, axis.min + ratio * (axis.max - axis.min));

    if (values[axis.name] !== newVal) {
      const updated = { ...values, [axis.name]: newVal };
      onChange?.(updated);
    }
  }

  function handlePointerUp() {
    draggingIndex = null;
  }

  function handlePointHover(e: PointerEvent, index: number) {
    hoveredIndex = index;
    const svg = (e.currentTarget as SVGElement).closest('svg');
    if (!svg) return;
    const pt = svg.createSVGPoint();
    pt.x = e.clientX;
    pt.y = e.clientY;
    const svgPt = pt.matrixTransform(svg.getScreenCTM()!.inverse());
    tooltipX = svgPt.x;
    tooltipY = svgPt.y - 14;
  }

  function handlePointLeave() {
    if (draggingIndex === null) {
      hoveredIndex = null;
    }
  }

  function handleInputChange(axisName: string, raw: string) {
    const axis = axes.find((a) => a.name === axisName);
    if (!axis) return;
    const parsed = parseInt(raw, 10);
    if (isNaN(parsed)) return;
    const clamped = clampValue(axis, parsed);
    const updated = { ...values, [axisName]: clamped };
    onChange?.(updated);
  }
</script>

<div class="spider-chart" style:--chart-color={color}>
  <svg
    viewBox="0 0 {size} {size}"
    width="100%"
    height="100%"
    role="img"
    aria-label="Spider chart showing {axes.length} axes"
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
  >
    <!-- Grid polygons -->
    {#each gridPolygons as points, level}
      <polygon
        {points}
        class="grid-polygon"
        opacity={0.3 + (level * 0.2)}
      />
    {/each}

    <!-- Axis lines -->
    {#each axes as _, i}
      {@const end = axisEndpoint(i)}
      <line
        x1={cx}
        y1={cy}
        x2={end.x}
        y2={end.y}
        class="axis-line"
      />
    {/each}

    <!-- Value polygon -->
    {#if axes.length > 0}
      <polygon
        points={valuePolygon}
        class="value-polygon"
        fill={color}
        stroke={color}
      />
    {/if}

    <!-- Value points (interactive) -->
    {#each axes as axis, i}
      {@const pt = valuePoint(i)}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- Invisible larger hit area -->
      <circle
        cx={pt.x}
        cy={pt.y}
        r={pointHitRadius}
        fill="transparent"
        style="cursor: {readonly ? 'default' : 'grab'};"
        onpointerdown={(e) => handlePointerDown(e, i)}
        onpointerenter={(e) => handlePointHover(e, i)}
        onpointerleave={handlePointLeave}
        role={readonly ? 'presentation' : 'slider'}
        aria-label="{axis.name}: {values[axis.name] ?? axis.default}"
        aria-valuemin={axis.min}
        aria-valuemax={axis.max}
        aria-valuenow={values[axis.name] ?? axis.default}
        tabindex={readonly ? undefined : 0}
      />
      <!-- Visible point -->
      <circle
        cx={pt.x}
        cy={pt.y}
        r={pointRadius}
        class="value-point"
        class:dragging={draggingIndex === i}
        fill={color}
        style="pointer-events: none;"
      />
    {/each}

    <!-- Axis labels -->
    {#each axes as axis, i}
      {@const pos = labelPosition(i)}
      <text
        x={pos.x}
        y={pos.y}
        text-anchor={pos.anchor}
        dominant-baseline="central"
        class="axis-label"
      >{axis.name}</text>
    {/each}

    <!-- Tooltip -->
    {#if hoveredIndex !== null}
      {@const axis = axes[hoveredIndex]}
      {@const val = values[axis.name] ?? axis.default}
      <g class="tooltip-group" transform="translate({tooltipX}, {tooltipY})">
        <rect
          x={-60}
          y={-28}
          width={120}
          height={axis.description ? 40 : 26}
          rx={4}
          class="tooltip-bg"
        />
        <text x={0} y={-14} text-anchor="middle" class="tooltip-title">
          {axis.name}: {val}
        </text>
        {#if axis.description}
          <text x={0} y={2} text-anchor="middle" class="tooltip-desc">
            {axis.description.length > 20 ? axis.description.slice(0, 20) + '...' : axis.description}
          </text>
        {/if}
      </g>
    {/if}
  </svg>

  <!-- Number input fallbacks for accessibility -->
  {#if !readonly}
    <div class="input-fallbacks">
      {#each axes as axis}
        <label class="axis-input">
          <span class="axis-input-label">{axis.name}</span>
          <input
            type="number"
            min={axis.min}
            max={axis.max}
            value={values[axis.name] ?? axis.default}
            oninput={(e) => handleInputChange(axis.name, (e.currentTarget as HTMLInputElement).value)}
          />
        </label>
      {/each}
    </div>
  {/if}
</div>

<style>
  .spider-chart {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
  }

  svg {
    max-width: 100%;
    aspect-ratio: 1;
  }

  .grid-polygon {
    fill: none;
    stroke: var(--border-secondary);
    stroke-width: 0.5;
  }

  .axis-line {
    stroke: var(--border-primary);
    stroke-width: 0.5;
  }

  .value-polygon {
    fill-opacity: 0.15;
    stroke-width: 2;
    stroke-opacity: 0.8;
    transition: d 0.15s ease;
  }

  .value-point {
    stroke: var(--bg-primary);
    stroke-width: 2;
    transition:
      cx 0.15s ease,
      cy 0.15s ease;
  }

  .value-point.dragging {
    r: 8;
    stroke-width: 3;
  }

  .axis-label {
    fill: var(--text-secondary);
    font-size: 11px;
    font-family: inherit;
    user-select: none;
  }

  .tooltip-bg {
    fill: var(--bg-elevated);
    stroke: var(--border-primary);
    stroke-width: 0.5;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.15));
  }

  .tooltip-title {
    fill: var(--text-primary);
    font-size: 11px;
    font-weight: 600;
    font-family: inherit;
  }

  .tooltip-desc {
    fill: var(--text-tertiary);
    font-size: 10px;
    font-family: inherit;
  }

  .input-fallbacks {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: center;
    width: 100%;
  }

  .axis-input {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.2rem;
    font-size: 0.75rem;
  }

  .axis-input-label {
    color: var(--text-secondary);
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .axis-input input {
    width: 3.5rem;
    padding: 0.2rem 0.3rem;
    text-align: center;
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-size: 0.75rem;
    font-family: inherit;
  }

  .axis-input input:focus {
    outline: none;
    border-color: var(--chart-color, var(--border-accent));
  }
</style>
