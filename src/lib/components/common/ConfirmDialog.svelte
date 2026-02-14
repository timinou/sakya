<script lang="ts">
  import Modal from './Modal.svelte';

  interface Props {
    isOpen: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    destructive?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    isOpen,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    destructive = false,
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleConfirm() {
    onConfirm();
  }

  function handleCancel() {
    onCancel();
  }
</script>

<Modal {isOpen} onClose={onCancel} {title}>
  <div class="confirm-body">
    <p class="confirm-message">{message}</p>
    <div class="confirm-actions">
      <button
        class="btn btn-cancel"
        onclick={handleCancel}
        type="button"
      >
        {cancelLabel}
      </button>
      <button
        class="btn btn-confirm"
        class:btn-destructive={destructive}
        onclick={handleConfirm}
        type="button"
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</Modal>

<style>
  .confirm-body {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .confirm-message {
    font-size: var(--font-size-base);
    color: var(--text-secondary);
    line-height: var(--line-height-normal);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }

  .btn {
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .btn-cancel {
    background: var(--bg-elevated);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
  }

  .btn-cancel:hover {
    background: var(--bg-tertiary);
    border-color: var(--border-primary);
  }

  .btn-confirm {
    background: var(--accent-primary);
    border: 1px solid var(--accent-primary);
    color: var(--text-inverse);
  }

  .btn-confirm:hover {
    opacity: 0.9;
    box-shadow: var(--shadow-sm);
  }

  .btn-destructive {
    background: var(--color-error);
    border-color: var(--color-error);
  }

  .btn-destructive:hover {
    opacity: 0.9;
  }

  .btn:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }
</style>
