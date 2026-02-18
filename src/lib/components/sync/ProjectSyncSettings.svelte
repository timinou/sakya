<script lang="ts">
  import { syncStore } from '$lib/stores';

  interface Props {
    projectId: string;
    projectName?: string;
  }

  let { projectId, projectName = 'Project' }: Props = $props();

  let loading = $state(false);
  let error = $state<string | null>(null);

  const syncState = $derived(syncStore.syncedProjects.get(projectId));
  const isSyncEnabled = $derived(syncState?.enabled ?? false);

  async function toggleSync() {
    loading = true;
    error = null;
    try {
      if (isSyncEnabled) {
        await syncStore.disableProjectSync(projectId);
      } else {
        // In a full implementation, the doc key would come from the keystore.
        // For now, we use a placeholder that will be replaced when
        // the full sync flow is integrated.
        const placeholderKey = Array.from({ length: 32 }, () => 0);
        await syncStore.enableProjectSync(projectId, placeholderKey);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="project-sync">
  <div class="sync-toggle-row">
    <div class="toggle-info">
      <span class="toggle-label">Sync "{projectName}"</span>
      <span class="toggle-description">
        {#if isSyncEnabled}
          Sync is enabled for this project.
        {:else}
          Enable sync to collaborate across devices.
        {/if}
      </span>
    </div>
    <button
      class="toggle-switch"
      class:active={isSyncEnabled}
      onclick={toggleSync}
      disabled={loading || !syncStore.isConnected}
      role="switch"
      aria-checked={isSyncEnabled}
      aria-label="Toggle sync for {projectName}"
      type="button"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>

  {#if isSyncEnabled && syncState}
    <div class="sync-details">
      <div class="detail-row">
        <span class="detail-label">Last sync</span>
        <span class="detail-value">
          {syncState.lastSyncTime
            ? new Date(syncState.lastSyncTime).toLocaleTimeString()
            : 'Not yet synced'}
        </span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Pending updates</span>
        <span class="detail-value">{syncState.pendingUpdates}</span>
      </div>
    </div>
  {/if}

  {#if !syncStore.isConnected}
    <p class="warning-message">Connect to the sync server to enable project sync.</p>
  {/if}

  {#if error}
    <p class="error-message">{error}</p>
  {/if}
</div>

<style>
  .project-sync {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .sync-toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .toggle-description {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    border: none;
    border-radius: 12px;
    background: var(--bg-tertiary);
    cursor: pointer;
    transition: background-color var(--transition-fast);
    flex-shrink: 0;
  }

  .toggle-switch.active {
    background: var(--accent-primary);
  }

  .toggle-switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: white;
    transition: transform var(--transition-fast);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(20px);
  }

  .sync-details {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .detail-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .detail-value {
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }

  .warning-message {
    font-size: var(--font-size-xs);
    color: var(--color-warning, #f59e0b);
    margin: 0;
  }

  .error-message {
    font-size: var(--font-size-xs);
    color: var(--color-error, #ef4444);
    margin: 0;
  }
</style>
