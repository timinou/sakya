<script lang="ts">
  import { syncStore } from '$lib/stores';

  let popoverOpen = $state(false);

  const statusConfig = $derived.by(() => {
    switch (syncStore.connectionStatus) {
      case 'connected':
        return { label: 'Synced', cssClass: 'status-synced' };
      case 'connecting':
        return { label: 'Connecting...', cssClass: 'status-syncing' };
      case 'reconnecting':
        return { label: 'Reconnecting...', cssClass: 'status-syncing' };
      case 'disconnected':
        return { label: 'Offline', cssClass: 'status-offline' };
      case 'error':
        return { label: 'Sync Error', cssClass: 'status-error' };
      default:
        return { label: 'Offline', cssClass: 'status-offline' };
    }
  });

  function togglePopover() {
    popoverOpen = !popoverOpen;
  }
</script>

<div class="sync-indicator-wrapper">
  <button
    class="sync-indicator {statusConfig.cssClass}"
    onclick={togglePopover}
    aria-label="Sync status: {statusConfig.label}"
    type="button"
  >
    <span class="status-dot"></span>
    <span class="status-label">{statusConfig.label}</span>
  </button>

  {#if popoverOpen}
    <div class="sync-popover" role="dialog" aria-label="Sync details">
      <div class="popover-header">
        <span class="popover-title">Sync Status</span>
        <button class="popover-close" onclick={() => popoverOpen = false} type="button" aria-label="Close">&times;</button>
      </div>
      <div class="popover-body">
        <div class="popover-row">
          <span class="popover-label">Status</span>
          <span class="popover-value {statusConfig.cssClass}">{statusConfig.label}</span>
        </div>
        {#if syncStore.lastError}
          <div class="popover-row">
            <span class="popover-label">Error</span>
            <span class="popover-value popover-error">{syncStore.lastError}</span>
          </div>
        {/if}
        <div class="popover-row">
          <span class="popover-label">Pending</span>
          <span class="popover-value">{syncStore.pendingUpdates} updates</span>
        </div>
        {#if syncStore.isLoggedIn}
          <div class="popover-row">
            <span class="popover-label">Account</span>
            <span class="popover-value">{syncStore.account?.email}</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .sync-indicator-wrapper {
    position: relative;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }

  .sync-indicator:hover {
    background: var(--bg-tertiary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-synced .status-dot {
    background: var(--color-success, #22c55e);
  }

  .status-syncing .status-dot {
    background: var(--color-warning, #f59e0b);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status-offline .status-dot {
    background: var(--text-tertiary, #9ca3af);
  }

  .status-error .status-dot {
    background: var(--color-error, #ef4444);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .sync-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    min-width: 240px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 50;
  }

  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid var(--border-secondary);
  }

  .popover-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .popover-close {
    border: none;
    background: transparent;
    font-size: var(--font-size-lg);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0 4px;
  }

  .popover-body {
    padding: var(--spacing-sm) var(--spacing-md);
  }

  .popover-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 0;
  }

  .popover-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .popover-value {
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }

  .popover-error {
    color: var(--color-error, #ef4444);
  }
</style>
