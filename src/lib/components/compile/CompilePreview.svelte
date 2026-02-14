<script lang="ts">
  import type { CompileOutput } from '$lib/types';

  interface Props {
    output: CompileOutput | null;
    loading: boolean;
    error: string | null;
  }

  let { output, loading, error }: Props = $props();

  const MAX_PREVIEW_CHARS = 500;

  let truncatedContent = $derived(() => {
    if (!output?.content) return '';
    if (output.content.length <= MAX_PREVIEW_CHARS) return output.content;
    return output.content.slice(0, MAX_PREVIEW_CHARS) + '\u2026';
  });
</script>

<div class="compile-preview">
  <div class="preview-header">
    <h3 class="preview-title">Preview</h3>
    {#if output}
      <div class="preview-badges">
        <span class="badge">{output.chapterCount} {output.chapterCount === 1 ? 'chapter' : 'chapters'}</span>
        <span class="badge">{output.wordCount.toLocaleString()} words</span>
      </div>
    {/if}
  </div>

  <div class="preview-content">
    {#if loading}
      <div class="preview-loading">
        <span class="loading-dots">Generating preview</span>
      </div>
    {:else if error}
      <div class="preview-error">
        <p>{error}</p>
      </div>
    {:else if output?.content}
      <pre class="preview-text">{truncatedContent()}</pre>
    {:else}
      <div class="preview-empty">
        <p>No content to preview. Make sure your manuscript has chapters.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .compile-preview {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--border-secondary);
    margin-bottom: var(--spacing-sm);
    flex-shrink: 0;
  }

  .preview-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .preview-badges {
    display: flex;
    gap: var(--spacing-xs);
  }

  .badge {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    padding: 2px var(--spacing-xs);
    border-radius: var(--radius-sm);
  }

  .preview-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .preview-text {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
    margin: 0;
    padding: var(--spacing-sm);
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-secondary);
  }

  .preview-loading,
  .preview-error,
  .preview-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 120px;
    padding: var(--spacing-md);
  }

  .preview-loading {
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
  }

  .loading-dots::after {
    content: '';
    animation: dots 1.5s steps(4, end) infinite;
  }

  @keyframes dots {
    0%, 20% { content: ''; }
    40% { content: '.'; }
    60% { content: '..'; }
    80%, 100% { content: '...'; }
  }

  .preview-error p {
    color: var(--color-error);
    font-size: var(--font-size-sm);
    margin: 0;
  }

  .preview-empty p {
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
    font-style: italic;
    margin: 0;
  }
</style>
