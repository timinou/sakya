<script lang="ts">
  import Modal from '$lib/components/common/Modal.svelte';
  import { syncStore } from '$lib/stores';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    serverUrl?: string;
  }

  let { isOpen, onClose, serverUrl = '' }: Props = $props();

  let activeTab = $state<'show' | 'enter'>('show');
  let pairingCode = $state<{ qr_svg: string; pairing_string: string } | null>(null);
  let remoteCode = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let copied = $state(false);

  $effect(() => {
    if (isOpen && !pairingCode && serverUrl) {
      generateCode();
    }
  });

  async function generateCode() {
    if (!serverUrl) return;
    loading = true;
    error = null;
    try {
      pairingCode = await syncStore.generatePairingCode(serverUrl);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function copyPairingString() {
    if (!pairingCode) return;
    try {
      await navigator.clipboard.writeText(pairingCode.pairing_string);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    } catch {
      // Fallback: select text
    }
  }

  async function submitRemoteCode() {
    if (!remoteCode.trim()) return;
    loading = true;
    error = null;
    try {
      await syncStore.completePairing(remoteCode.trim());
      remoteCode = '';
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    pairingCode = null;
    remoteCode = '';
    error = null;
    activeTab = 'show';
    onClose();
  }
</script>

<Modal isOpen={isOpen} onClose={handleClose} title="Pair Device">
  <div class="pairing-dialog">
    <div class="tab-bar" role="tablist">
      <button
        class="tab"
        class:active={activeTab === 'show'}
        onclick={() => { activeTab = 'show'; }}
        role="tab"
        aria-selected={activeTab === 'show'}
        type="button"
      >
        Show Code
      </button>
      <button
        class="tab"
        class:active={activeTab === 'enter'}
        onclick={() => { activeTab = 'enter'; }}
        role="tab"
        aria-selected={activeTab === 'enter'}
        type="button"
      >
        Enter Code
      </button>
    </div>

    <div class="tab-content">
      {#if activeTab === 'show'}
        <div class="show-code" role="tabpanel">
          {#if loading}
            <p class="loading-text">Generating pairing code...</p>
          {:else if pairingCode}
            <div class="qr-container">
              {@html pairingCode.qr_svg}
            </div>
            <div class="pairing-string-group">
              <code class="pairing-string">{pairingCode.pairing_string}</code>
              <button class="btn-copy" onclick={copyPairingString} type="button">
                {copied ? 'Copied!' : 'Copy'}
              </button>
            </div>
            <p class="help-text">
              Scan the QR code or share the pairing string with your other device.
            </p>
          {:else}
            <p class="help-text">Enter a server URL to generate a pairing code.</p>
          {/if}
        </div>
      {:else}
        <div class="enter-code" role="tabpanel">
          <p class="help-text">Enter the pairing string from your other device.</p>
          <div class="input-group">
            <input
              type="text"
              bind:value={remoteCode}
              placeholder="sk-pair_v1...."
              class="input"
              disabled={loading}
            />
          </div>
          <button
            class="btn btn-primary btn-full"
            onclick={submitRemoteCode}
            disabled={loading || !remoteCode.trim()}
            type="button"
          >
            {loading ? 'Pairing...' : 'Pair Device'}
          </button>
        </div>
      {/if}

      {#if error}
        <p class="error-message">{error}</p>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .pairing-dialog {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    min-width: 320px;
  }

  .tab-bar {
    display: flex;
    border-bottom: 1px solid var(--border-secondary);
    gap: 0;
  }

  .tab {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: transparent;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color var(--transition-fast), border-color var(--transition-fast);
  }

  .tab.active {
    color: var(--accent-primary);
    border-bottom-color: var(--accent-primary);
  }

  .tab:hover:not(.active) {
    color: var(--text-primary);
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .qr-container {
    display: flex;
    justify-content: center;
    padding: var(--spacing-md);
    background: white;
    border-radius: var(--radius-md);
  }

  .qr-container :global(svg) {
    width: 200px;
    height: 200px;
  }

  .pairing-string-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .pairing-string {
    flex: 1;
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    background: var(--bg-secondary);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    word-break: break-all;
    font-family: var(--font-mono);
  }

  .btn-copy {
    border: 1px solid var(--border-primary);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
  }

  .help-text {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    margin: 0;
  }

  .loading-text {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    text-align: center;
    padding: var(--spacing-lg);
  }

  .input-group {
    display: flex;
    gap: var(--spacing-sm);
  }

  .input {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
  }

  .input:focus {
    outline: 2px solid var(--accent-primary);
    outline-offset: -1px;
  }

  .btn {
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
  }

  .btn-primary {
    background: var(--accent-primary);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-full {
    width: 100%;
  }

  .error-message {
    font-size: var(--font-size-xs);
    color: var(--color-error, #ef4444);
    margin: 0;
  }
</style>
