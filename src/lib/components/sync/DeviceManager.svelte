<script lang="ts">
  import { syncStore } from '$lib/stores';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

  interface Props {
    onAddDevice?: () => void;
  }

  let { onAddDevice }: Props = $props();

  let confirmRemoveId = $state<string | null>(null);
  let loading = $state(false);

  $effect(() => {
    // Load devices on mount
    syncStore.listDevices().catch(() => {
      // Silently fail — devices will be empty
    });
  });

  async function removeDevice(deviceId: string) {
    loading = true;
    try {
      await syncStore.removeDevice(deviceId);
    } finally {
      loading = false;
      confirmRemoveId = null;
    }
  }
</script>

<div class="device-manager">
  <div class="device-header">
    <h3 class="device-title">Devices</h3>
    <button class="btn btn-primary btn-sm" onclick={() => onAddDevice?.()} type="button">
      Add Device
    </button>
  </div>

  <ul class="device-list">
    {#each syncStore.devices as device (device.device_id)}
      <li class="device-item">
        <div class="device-info">
          <span class="device-name">
            {device.name}
            {#if device.is_current}
              <span class="badge-current">This device</span>
            {/if}
          </span>
          <span class="device-id">{device.device_id.slice(0, 8)}...</span>
        </div>
        {#if !device.is_current}
          <button
            class="btn-remove"
            onclick={() => { confirmRemoveId = device.device_id; }}
            disabled={loading}
            type="button"
            aria-label="Remove device {device.name}"
          >
            Remove
          </button>
        {/if}
      </li>
    {:else}
      <li class="device-empty">No devices paired yet.</li>
    {/each}
  </ul>
</div>

{#if confirmRemoveId}
  <ConfirmDialog
    isOpen={true}
    onConfirm={() => removeDevice(confirmRemoveId!)}
    onCancel={() => { confirmRemoveId = null; }}
    title="Remove Device"
    message="This will remove the device and rotate all encryption keys. The removed device will no longer be able to sync."
    confirmLabel="Remove"
    destructive={true}
  />
{/if}

<style>
  .device-manager {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .device-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .device-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .device-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .device-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
  }

  .device-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .device-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .badge-current {
    font-size: var(--font-size-xxs, 10px);
    font-weight: var(--font-weight-semibold);
    color: var(--accent-primary);
    background: var(--accent-subtle, rgba(59, 130, 246, 0.1));
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }

  .device-id {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .device-empty {
    font-size: var(--font-size-sm);
    color: var(--text-tertiary);
    text-align: center;
    padding: var(--spacing-lg);
  }

  .btn-remove {
    border: none;
    background: transparent;
    color: var(--color-error, #ef4444);
    font-size: var(--font-size-xs);
    cursor: pointer;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    transition: background-color var(--transition-fast);
  }

  .btn-remove:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.1);
  }

  .btn-remove:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    border-radius: var(--radius-sm);
    background: var(--accent-primary);
    color: white;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
  }

  .btn-sm {
    padding: 4px var(--spacing-sm);
    font-size: var(--font-size-xs);
  }
</style>
