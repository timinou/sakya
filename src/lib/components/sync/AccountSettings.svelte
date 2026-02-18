<script lang="ts">
  import { syncStore } from '$lib/stores';

  let email = $state('');
  let magicLinkSent = $state(false);
  let verificationToken = $state('');
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function sendMagicLink() {
    if (!email.trim()) return;
    loading = true;
    error = null;
    try {
      // In a full implementation, this would call a Tauri command
      // that sends a magic link email via the sync server API.
      // For now, simulate the flow.
      magicLinkSent = true;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function verifyToken() {
    if (!verificationToken.trim()) return;
    loading = true;
    error = null;
    try {
      // In a full implementation, this would verify the token
      // and retrieve account info + JWT from the server.
      syncStore.login({
        email: email,
        accountId: crypto.randomUUID(),
      });
      // Reset form
      magicLinkSent = false;
      verificationToken = '';
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function logout() {
    syncStore.logout();
    email = '';
    magicLinkSent = false;
    verificationToken = '';
    error = null;
  }
</script>

<div class="account-settings">
  {#if syncStore.isLoggedIn}
    <div class="logged-in">
      <div class="account-info">
        <span class="account-label">Signed in as</span>
        <span class="account-email">{syncStore.account?.email}</span>
      </div>
      <button class="btn btn-secondary" onclick={logout} type="button">
        Log Out
      </button>
    </div>
  {:else if magicLinkSent}
    <div class="verify-section">
      <p class="verify-message">Check your email for a login link, or enter the verification code below.</p>
      <div class="input-group">
        <input
          type="text"
          bind:value={verificationToken}
          placeholder="Verification code"
          class="input"
          disabled={loading}
        />
        <button
          class="btn btn-primary"
          onclick={verifyToken}
          disabled={loading || !verificationToken.trim()}
          type="button"
        >
          {loading ? 'Verifying...' : 'Verify'}
        </button>
      </div>
      <button class="btn-link" onclick={() => { magicLinkSent = false; }} type="button">
        Use a different email
      </button>
    </div>
  {:else}
    <div class="login-section">
      <p class="login-message">Sign in to enable sync across your devices.</p>
      <div class="input-group">
        <input
          type="email"
          bind:value={email}
          placeholder="you@example.com"
          class="input"
          disabled={loading}
        />
        <button
          class="btn btn-primary"
          onclick={sendMagicLink}
          disabled={loading || !email.trim()}
          type="button"
        >
          {loading ? 'Sending...' : 'Send Magic Link'}
        </button>
      </div>
    </div>
  {/if}

  {#if error}
    <p class="error-message">{error}</p>
  {/if}
</div>

<style>
  .account-settings {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .logged-in {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
  }

  .account-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .account-label {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .account-email {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .login-message,
  .verify-message {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0;
  }

  .input-group {
    display: flex;
    gap: var(--spacing-sm);
  }

  .input {
    flex: 1;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
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
    transition: background-color var(--transition-fast);
    white-space: nowrap;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent-primary);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-quaternary, var(--border-primary));
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--accent-primary);
    font-size: var(--font-size-xs);
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }

  .error-message {
    font-size: var(--font-size-xs);
    color: var(--color-error, #ef4444);
    margin: 0;
  }
</style>
