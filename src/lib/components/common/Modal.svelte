<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    title?: string;
    closeOnBackdrop?: boolean;
    children: Snippet;
  }

  let {
    isOpen,
    onClose,
    title,
    closeOnBackdrop = true,
    children,
  }: Props = $props();

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogEl) return;
    if (isOpen && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!isOpen && dialogEl.open) {
      dialogEl.close();
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (!closeOnBackdrop) return;
    if (e.target === dialogEl) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <dialog
    bind:this={dialogEl}
    class="modal"
    onkeydown={handleKeydown}
    onclick={handleBackdropClick}
    aria-label={title ?? 'Dialog'}
  >
    <div class="modal-content" role="document">
      {#if title}
        <header class="modal-header">
          <h2 class="modal-title">{title}</h2>
          <button
            class="modal-close"
            onclick={onClose}
            aria-label="Close dialog"
            type="button"
          >
            &times;
          </button>
        </header>
      {/if}
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </dialog>
{/if}

<style>
  .modal {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    padding: 0;
    margin: 0;
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    background: transparent;
    animation: modal-fade-in var(--transition-normal) forwards;
  }

  .modal::backdrop {
    background: rgba(0, 0, 0, 0.5);
    animation: backdrop-fade-in var(--transition-normal) forwards;
  }

  @keyframes modal-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes backdrop-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-content {
    background: var(--bg-elevated);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    min-width: 320px;
    max-width: 560px;
    max-height: 80vh;
    overflow-y: auto;
    animation: content-slide-in var(--transition-normal) forwards;
  }

  @keyframes content-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--border-secondary);
  }

  .modal-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xl);
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--transition-fast), color var(--transition-fast);
  }

  .modal-close:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    box-shadow: none;
  }

  .modal-close:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }

  .modal-body {
    padding: var(--spacing-lg);
  }
</style>
