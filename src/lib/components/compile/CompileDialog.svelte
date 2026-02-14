<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import { projectState } from '$lib/stores';
  import type {
    CompileConfig,
    CompileOutput,
    OutputFormat,
    ChapterHeaderStyle,
    ChapterSeparator,
  } from '$lib/types';
  import { defaultCompileConfig } from '$lib/types';
  import CompilePreview from './CompilePreview.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onCompile: (config: CompileConfig, output: CompileOutput) => void;
  }

  let { isOpen, onClose, onCompile }: Props = $props();

  // --- Config state ---
  let config: CompileConfig = $state(defaultCompileConfig());

  // --- Preview state ---
  let previewOutput: CompileOutput | null = $state(null);
  let previewLoading = $state(false);
  let previewError: string | null = $state(null);

  // --- UI state ---
  let frontMatterExpanded = $state(false);
  let compiling = $state(false);

  // --- Debounced preview ---
  let previewTimer: ReturnType<typeof setTimeout> | null = null;

  // Initialize config from project manifest when dialog opens
  $effect(() => {
    if (isOpen) {
      const manifest = projectState.manifest;
      config = {
        ...defaultCompileConfig(),
        title: manifest?.name ?? '',
        author: manifest?.author ?? '',
      };
      frontMatterExpanded = false;
      compiling = false;
      previewOutput = null;
      previewError = null;
      // Trigger initial preview
      schedulePreview();
    } else {
      // Clean up timer when dialog closes
      if (previewTimer) {
        clearTimeout(previewTimer);
        previewTimer = null;
      }
    }
  });

  // Watch config changes for preview updates
  $effect(() => {
    if (!isOpen) return;
    // Read all config fields to track reactively
    const _fmt = config.outputFormat;
    const _tp = config.includeTitlePage;
    const _title = config.title;
    const _author = config.author;
    const _header = config.chapterHeaderStyle;
    const _sep = config.chapterSeparator;
    const _syn = config.includeSynopsis;
    const _fm = config.frontMatter;

    schedulePreview();
  });

  function schedulePreview() {
    if (previewTimer) clearTimeout(previewTimer);
    previewTimer = setTimeout(() => {
      fetchPreview();
    }, 500);
  }

  async function fetchPreview() {
    const path = projectState.projectPath;
    if (!path) {
      previewError = 'No project open';
      return;
    }

    previewLoading = true;
    previewError = null;

    try {
      const result = await invoke<CompileOutput>('compile_manuscript', {
        projectPath: path,
        config: { ...config },
      });
      previewOutput = result;
    } catch (err) {
      previewError = err instanceof Error ? err.message : String(err);
      previewOutput = null;
    } finally {
      previewLoading = false;
    }
  }

  async function handleCompile() {
    const path = projectState.projectPath;
    if (!path) return;

    compiling = true;
    try {
      const result = await invoke<CompileOutput>('compile_manuscript', {
        projectPath: path,
        config: { ...config },
      });
      onCompile(config, result);
    } catch (err) {
      previewError = err instanceof Error ? err.message : String(err);
    } finally {
      compiling = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  // --- Label maps ---
  const formatLabels: Record<OutputFormat, string> = {
    markdown: 'Markdown',
    html: 'HTML',
    plain_text: 'Plain Text',
  };

  const headerLabels: Record<ChapterHeaderStyle, string> = {
    numbered: 'Numbered',
    titled: 'Titled',
    numbered_and_titled: 'Numbered & Titled',
    none: 'None',
  };

  const separatorLabels: Record<ChapterSeparator, string> = {
    page_break: 'Page Break',
    three_stars: 'Three Stars (* * *)',
    horizontal_rule: 'Horizontal Rule',
    blank_lines: 'Blank Lines',
  };

  const formats: OutputFormat[] = ['markdown', 'html', 'plain_text'];
  const headerStyles: ChapterHeaderStyle[] = ['numbered', 'titled', 'numbered_and_titled', 'none'];
  const separators: ChapterSeparator[] = ['page_break', 'three_stars', 'horizontal_rule', 'blank_lines'];

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogEl) return;
    if (isOpen && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!isOpen && dialogEl.open) {
      dialogEl.close();
    }
  });
</script>

{#if isOpen}
  <dialog
    bind:this={dialogEl}
    class="compile-modal"
    onkeydown={handleKeydown}
    onclick={handleBackdropClick}
    aria-label="Compile Manuscript"
  >
    <div class="compile-dialog" role="document">
      <header class="dialog-header">
        <h2 class="dialog-title">Compile Manuscript</h2>
        <button
          class="dialog-close"
          onclick={onClose}
          aria-label="Close dialog"
          type="button"
        >
          &times;
        </button>
      </header>

      <div class="dialog-body">
        <!-- Left panel: configuration -->
        <div class="config-panel">
          <!-- Output Format -->
          <fieldset class="config-section">
            <legend class="section-label">Output Format</legend>
            <div class="format-group" role="radiogroup" aria-label="Output format">
              {#each formats as fmt}
                <label class="format-option" class:active={config.outputFormat === fmt}>
                  <input
                    type="radio"
                    name="outputFormat"
                    value={fmt}
                    checked={config.outputFormat === fmt}
                    onchange={() => { config.outputFormat = fmt; }}
                  />
                  <span class="format-label">{formatLabels[fmt]}</span>
                </label>
              {/each}
            </div>
          </fieldset>

          <!-- Title Page -->
          <div class="config-section">
            <label class="toggle-row">
              <input
                type="checkbox"
                checked={config.includeTitlePage}
                onchange={(e) => { config.includeTitlePage = (e.target as HTMLInputElement).checked; }}
              />
              <span class="toggle-label">Include Title Page</span>
            </label>

            {#if config.includeTitlePage}
              <div class="title-fields">
                <label class="field-group">
                  <span class="field-label">Title</span>
                  <input
                    type="text"
                    class="text-input"
                    value={config.title}
                    oninput={(e) => { config.title = (e.target as HTMLInputElement).value; }}
                    placeholder="Manuscript title"
                  />
                </label>
                <label class="field-group">
                  <span class="field-label">Author</span>
                  <input
                    type="text"
                    class="text-input"
                    value={config.author}
                    oninput={(e) => { config.author = (e.target as HTMLInputElement).value; }}
                    placeholder="Author name"
                  />
                </label>
              </div>
            {/if}
          </div>

          <!-- Chapter Header Style -->
          <label class="config-section field-group">
            <span class="field-label">Chapter Header Style</span>
            <select
              class="select-input"
              value={config.chapterHeaderStyle}
              onchange={(e) => { config.chapterHeaderStyle = (e.target as HTMLSelectElement).value as ChapterHeaderStyle; }}
            >
              {#each headerStyles as style}
                <option value={style}>{headerLabels[style]}</option>
              {/each}
            </select>
          </label>

          <!-- Chapter Separator -->
          <label class="config-section field-group">
            <span class="field-label">Chapter Separator</span>
            <select
              class="select-input"
              value={config.chapterSeparator}
              onchange={(e) => { config.chapterSeparator = (e.target as HTMLSelectElement).value as ChapterSeparator; }}
            >
              {#each separators as sep}
                <option value={sep}>{separatorLabels[sep]}</option>
              {/each}
            </select>
          </label>

          <!-- Include Synopsis -->
          <div class="config-section">
            <label class="toggle-row">
              <input
                type="checkbox"
                checked={config.includeSynopsis}
                onchange={(e) => { config.includeSynopsis = (e.target as HTMLInputElement).checked; }}
              />
              <span class="toggle-label">Include Synopsis</span>
            </label>
          </div>

          <!-- Front Matter (collapsible) -->
          <div class="config-section">
            <button
              class="collapsible-header"
              type="button"
              onclick={() => { frontMatterExpanded = !frontMatterExpanded; }}
              aria-expanded={frontMatterExpanded}
            >
              {#if frontMatterExpanded}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
              <span class="toggle-label">Custom Front Matter</span>
            </button>

            {#if frontMatterExpanded}
              <textarea
                class="textarea-input"
                value={config.frontMatter}
                oninput={(e) => { config.frontMatter = (e.target as HTMLTextAreaElement).value; }}
                placeholder="Dedication, acknowledgements, epigraph..."
                rows={4}
              ></textarea>
            {/if}
          </div>
        </div>

        <!-- Right panel: preview -->
        <div class="preview-panel">
          <CompilePreview
            output={previewOutput}
            loading={previewLoading}
            error={previewError}
          />
        </div>
      </div>

      <footer class="dialog-footer">
        <button
          class="btn btn-cancel"
          onclick={onClose}
          type="button"
        >
          Cancel
        </button>
        <button
          class="btn btn-primary"
          onclick={handleCompile}
          disabled={compiling}
          type="button"
        >
          {compiling ? 'Compiling\u2026' : 'Compile & Save'}
        </button>
      </footer>
    </div>
  </dialog>
{/if}

<style>
  /* --- Dialog chrome --- */
  .compile-modal {
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

  .compile-modal::backdrop {
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

  .compile-dialog {
    background: var(--bg-elevated);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    width: 720px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
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

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--border-secondary);
    flex-shrink: 0;
  }

  .dialog-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .dialog-close {
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

  .dialog-close:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    box-shadow: none;
  }

  .dialog-close:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }

  /* --- Body: two-panel layout --- */
  .dialog-body {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg);
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .config-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    min-width: 0;
  }

  .preview-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  /* --- Config sections --- */
  .config-section {
    border: none;
    margin: 0;
    padding: 0;
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
  }

  /* --- Format segmented control --- */
  .format-group {
    display: flex;
    gap: 1px;
    background: var(--border-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .format-option {
    flex: 1;
    cursor: pointer;
  }

  .format-option input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
    pointer-events: none;
  }

  .format-label {
    display: block;
    padding: var(--spacing-xs) var(--spacing-sm);
    text-align: center;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    transition: background-color var(--transition-fast), color var(--transition-fast);
  }

  .format-label:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .format-option.active .format-label {
    background: var(--accent-primary);
    color: var(--text-inverse);
  }

  .format-option input:focus-visible + .format-label {
    outline: 2px solid var(--accent-primary);
    outline-offset: -2px;
  }

  /* --- Toggle rows (checkboxes) --- */
  .toggle-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
  }

  .toggle-row input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-primary);
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  /* --- Text inputs --- */
  .title-fields {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
    padding-left: calc(16px + var(--spacing-sm));
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .text-input,
  .select-input,
  .textarea-input {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
    box-sizing: border-box;
  }

  .text-input:focus,
  .select-input:focus,
  .textarea-input:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 25%, transparent);
  }

  .textarea-input {
    resize: vertical;
    min-height: 60px;
    margin-top: var(--spacing-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    line-height: 1.5;
  }

  /* --- Collapsible header --- */
  .collapsible-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    transition: color var(--transition-fast);
    box-shadow: none;
  }

  .collapsible-header:hover {
    color: var(--accent-primary);
    border-color: transparent;
    box-shadow: none;
  }

  .collapsible-header:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }

  /* --- Footer --- */
  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--border-secondary);
    flex-shrink: 0;
  }

  .btn {
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    font-size: var(--font-size-base, var(--font-size-sm));
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast),
      box-shadow var(--transition-fast),
      opacity var(--transition-fast);
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

  .btn-primary {
    background: var(--accent-primary);
    border: 1px solid var(--accent-primary);
    color: var(--text-inverse);
  }

  .btn-primary:hover {
    opacity: 0.9;
    box-shadow: var(--shadow-sm);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }

  /* --- Responsive --- */
  @media (max-width: 640px) {
    .dialog-body {
      flex-direction: column;
    }

    .compile-dialog {
      width: 95vw;
      max-height: 90vh;
    }
  }
</style>
