<script lang="ts">
  import Toast, { toastManager } from './Toast.svelte';

  let toasts = $derived(toastManager.toasts);
</script>

<div class="toast-container" aria-label="Notifications" role="status">
  {#each toasts as toast (toast.id)}
    <Toast
      message={toast.message}
      type={toast.type}
      duration={toast.duration}
      onDismiss={() => toastManager.dismiss(toast.id)}
    />
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: var(--spacing-md);
    right: var(--spacing-md);
    z-index: 300;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    pointer-events: none;
  }

  .toast-container :global(.toast) {
    pointer-events: auto;
  }
</style>
