<script lang="ts">
  interface Props {
    onResize: (delta: number) => void;
  }

  let { onResize }: Props = $props();
  let isDragging = $state(false);
  let startX = $state(0);

  function handlePointerDown(e: PointerEvent) {
    isDragging = true;
    startX = e.clientX;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    const delta = e.clientX - startX;
    startX = e.clientX;
    onResize(delta);
  }

  function handlePointerUp(e: PointerEvent) {
    isDragging = false;
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
  }
</script>

<div
  class="pane-resizer"
  class:dragging={isDragging}
  role="separator"
  aria-orientation="vertical"
  tabindex="-1"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
>
  <div class="resizer-line"></div>
</div>

<style>
  .pane-resizer {
    width: 4px;
    cursor: col-resize;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    transition: background-color var(--transition-fast);
    user-select: none;
    touch-action: none;
    z-index: 1;
  }

  .pane-resizer:hover,
  .pane-resizer.dragging {
    background: var(--accent-primary);
    opacity: 0.4;
  }

  .pane-resizer.dragging {
    opacity: 0.7;
  }

  .resizer-line {
    width: 1px;
    height: 100%;
    background: var(--border-secondary);
  }

  .pane-resizer:hover .resizer-line,
  .pane-resizer.dragging .resizer-line {
    background: transparent;
  }
</style>
