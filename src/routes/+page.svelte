<script lang="ts">
  import { projectState } from '$lib/stores';
  import { open } from '@tauri-apps/plugin-dialog';
  import { FolderOpen, Plus, FileText } from 'lucide-svelte';

  let showCreateForm = $state(false);
  let newProjectName = $state('');
  let newProjectPath = $state('');

  async function handleCreateProject() {
    if (!newProjectName.trim() || !newProjectPath.trim()) return;
    try {
      await projectState.create(newProjectName.trim(), newProjectPath);
      showCreateForm = false;
      newProjectName = '';
      newProjectPath = '';
    } catch {
      // error is captured in projectState.error
    }
  }

  async function handleChooseFolder() {
    try {
      const selected = await open({ directory: true, title: 'Choose Project Folder' });
      if (selected) {
        newProjectPath = selected as string;
      }
    } catch {
      // dialog cancelled or unavailable
    }
  }

  async function handleOpenProject() {
    try {
      const selected = await open({ directory: true, title: 'Open Project' });
      if (selected) {
        await projectState.open(selected as string);
      }
    } catch {
      // error is captured in projectState.error
    }
  }

  function handleCancelCreate() {
    showCreateForm = false;
    newProjectName = '';
    newProjectPath = '';
    projectState.error = null;
  }
</script>

{#if projectState.isOpen}
  {#await import('$lib/components/layout/AppShell.svelte') then { default: AppShell }}
    <AppShell />
  {/await}
{:else}
  <main class="launcher">
    <div class="launcher-card">
      <div class="launcher-header">
        <FileText size={40} strokeWidth={1.5} />
        <h1 class="launcher-title">Sakya</h1>
        <p class="launcher-subtitle">A writing application</p>
      </div>

      {#if projectState.error}
        <div class="error-message" role="alert">
          {projectState.error}
        </div>
      {/if}

      {#if projectState.isLoading}
        <div class="loading-indicator" aria-label="Loading">
          <span class="loading-spinner"></span>
          <span>Loading project...</span>
        </div>
      {/if}

      {#if showCreateForm}
        <form class="create-form" onsubmit={(e) => { e.preventDefault(); handleCreateProject(); }}>
          <div class="form-field">
            <label for="project-name">Project Name</label>
            <input
              id="project-name"
              type="text"
              bind:value={newProjectName}
              placeholder="My Novel"
              autocomplete="off"
            />
          </div>
          <div class="form-field">
            <label for="project-path">Location</label>
            <div class="path-input-row">
              <input
                id="project-path"
                type="text"
                value={newProjectPath}
                placeholder="Choose a folder..."
                readonly
              />
              <button type="button" class="btn-secondary" onclick={handleChooseFolder}>
                <FolderOpen size={16} />
                Choose Folder
              </button>
            </div>
          </div>
          <div class="form-actions">
            <button type="submit" class="btn-primary" disabled={!newProjectName.trim() || !newProjectPath.trim()}>
              Create
            </button>
            <button type="button" class="btn-secondary" onclick={handleCancelCreate}>
              Cancel
            </button>
          </div>
        </form>
      {:else}
        <div class="launcher-actions">
          <button class="btn-primary" onclick={() => { showCreateForm = true; }}>
            <Plus size={18} />
            Create Project
          </button>
          <button class="btn-secondary" onclick={handleOpenProject}>
            <FolderOpen size={18} />
            Open Project
          </button>
        </div>
      {/if}
    </div>
  </main>
{/if}

<style>
  .launcher {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background-color: var(--bg-primary);
    padding: var(--spacing-xl);
  }

  .launcher-card {
    background-color: var(--bg-elevated);
    border: 1px solid var(--border-secondary);
    border-radius: var(--radius-xl);
    padding: var(--spacing-2xl);
    max-width: 480px;
    width: 100%;
    box-shadow: var(--shadow-lg);
  }

  .launcher-header {
    text-align: center;
    margin-bottom: var(--spacing-xl);
    color: var(--text-primary);
  }

  .launcher-header :global(svg) {
    margin: 0 auto var(--spacing-md);
    color: var(--accent-primary);
  }

  .launcher-title {
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-bold);
    margin-bottom: var(--spacing-xs);
    color: var(--text-primary);
  }

  .launcher-subtitle {
    font-size: var(--font-size-md);
    color: var(--text-secondary);
  }

  .error-message {
    background-color: color-mix(in srgb, var(--color-error) 10%, transparent);
    border: 1px solid var(--color-error);
    border-radius: var(--radius-md);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-bottom: var(--spacing-lg);
    color: var(--color-error);
    font-size: var(--font-size-sm);
  }

  .loading-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-primary);
    border-top-color: var(--accent-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .launcher-actions {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .btn-primary,
  .btn-secondary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-lg);
    border-radius: var(--radius-md);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
    width: 100%;
  }

  .btn-primary {
    background-color: var(--accent-primary);
    color: var(--text-inverse);
    border: 1px solid var(--accent-primary);
  }

  .btn-primary:hover {
    box-shadow: var(--shadow-md);
    filter: brightness(1.1);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    filter: none;
    box-shadow: none;
  }

  .btn-secondary {
    background-color: var(--bg-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-primary);
  }

  .btn-secondary:hover {
    border-color: var(--accent-primary);
    box-shadow: var(--shadow-sm);
  }

  .create-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .form-field label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .path-input-row {
    display: flex;
    gap: var(--spacing-sm);
  }

  .path-input-row input {
    flex: 1;
    min-width: 0;
  }

  .path-input-row .btn-secondary {
    width: auto;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .form-actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .form-actions .btn-primary,
  .form-actions .btn-secondary {
    flex: 1;
  }
</style>
