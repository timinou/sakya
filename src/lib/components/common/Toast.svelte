<script lang="ts" module>
  interface ToastData {
    id: number;
    message: string;
    type: 'success' | 'error' | 'info';
    duration: number;
  }

  let nextId = 0;

  class ToastManager {
    toasts = $state<ToastData[]>([]);

    show(message: string, type: 'success' | 'error' | 'info' = 'info', duration: number = 3000): void {
      const id = nextId++;
      this.toasts = [...this.toasts, { id, message, type, duration }];
    }

    dismiss(id: number): void {
      this.toasts = this.toasts.filter((t) => t.id !== id);
    }
  }

  export const toastManager = new ToastManager();
</script>

<script lang="ts">
  import { X, CheckCircle, AlertCircle, Info } from 'lucide-svelte';

  interface Props {
    message: string;
    type?: 'success' | 'error' | 'info';
    duration?: number;
    onDismiss?: () => void;
  }

  let {
    message,
    type = 'info',
    duration = 3000,
    onDismiss,
  }: Props = $props();

  let visible = $state(true);

  $effect(() => {
    if (duration <= 0) return;
    const timeout = setTimeout(() => {
      dismiss();
    }, duration);
    return () => clearTimeout(timeout);
  });

  function dismiss() {
    visible = false;
    onDismiss?.();
  }

  const iconMap = {
    success: CheckCircle,
    error: AlertCircle,
    info: Info,
  };

  let IconComponent = $derived(iconMap[type]);
</script>

{#if visible}
  <div
    class="toast toast-{type}"
    role="alert"
    aria-live="polite"
  >
    <span class="toast-icon">
      <IconComponent size={18} />
    </span>
    <span class="toast-message">{message}</span>
    <button
      class="toast-dismiss"
      onclick={dismiss}
      aria-label="Dismiss notification"
      type="button"
    >
      <X size={14} />
    </button>
  </div>
{/if}

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-primary);
    box-shadow: var(--shadow-md);
    min-width: 280px;
    max-width: 420px;
    animation: toast-slide-in var(--transition-normal) forwards;
  }

  @keyframes toast-slide-in {
    from {
      opacity: 0;
      transform: translateX(16px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .toast-success {
    border-left: 3px solid var(--color-success);
  }

  .toast-success .toast-icon {
    color: var(--color-success);
  }

  .toast-error {
    border-left: 3px solid var(--color-error);
  }

  .toast-error .toast-icon {
    color: var(--color-error);
  }

  .toast-info {
    border-left: 3px solid var(--color-info);
  }

  .toast-info .toast-icon {
    color: var(--color-info);
  }

  .toast-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    line-height: var(--line-height-tight);
  }

  .toast-dismiss {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--transition-fast), color var(--transition-fast);
  }

  .toast-dismiss:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    box-shadow: none;
    border-color: transparent;
  }

  .toast-dismiss:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }
</style>
