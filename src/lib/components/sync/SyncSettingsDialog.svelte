<script lang="ts">
  import Modal from '$lib/components/common/Modal.svelte';
  import { syncStore } from '$lib/stores';
  import AccountSettings from './AccountSettings.svelte';
  import DeviceManager from './DeviceManager.svelte';
  import PairingDialog from './PairingDialog.svelte';
  import ProjectSyncSettings from './ProjectSyncSettings.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    projectId?: string;
    projectName?: string;
  }

  let { isOpen, onClose, projectId, projectName }: Props = $props();

  let pairingOpen = $state(false);
</script>

<Modal {isOpen} {onClose} title="Sync Settings">
  <div class="sync-settings-dialog">
    <section class="settings-section">
      <h3 class="section-title">Account</h3>
      <AccountSettings />
    </section>

    <section class="settings-section">
      <h3 class="section-title">Devices</h3>
      <DeviceManager onAddDevice={() => { pairingOpen = true; }} />
    </section>

    {#if projectId}
      <section class="settings-section">
        <h3 class="section-title">Project Sync</h3>
        <ProjectSyncSettings {projectId} {projectName} />
      </section>
    {/if}
  </div>
</Modal>

<PairingDialog isOpen={pairingOpen} onClose={() => { pairingOpen = false; }} serverUrl={syncStore.serverUrl} />

<style>
  .sync-settings-dialog {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .settings-section + .settings-section {
    padding-top: var(--spacing-lg);
    border-top: 1px solid var(--border-secondary);
  }

  .section-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }
</style>
