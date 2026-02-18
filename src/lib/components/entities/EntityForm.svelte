<script lang="ts">
  import { untrack } from 'svelte';
  import type { EntitySchema, EntityInstance } from '$lib/types';
  import FieldRenderer from './FieldRenderer.svelte';
  import SpiderChart from '$lib/components/charts/SpiderChart.svelte';

  interface Props {
    schema: EntitySchema;
    entity: EntityInstance;
    onSave?: (entity: EntityInstance) => void;
    readonly?: boolean;
  }

  let { schema, entity, onSave, readonly = false }: Props = $props();

  let entitySnapshot = $derived(JSON.stringify(entity));

  let localEntity = $state<EntityInstance>(undefined as unknown as EntityInstance);
  let isDirty = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSyncedSnapshot = $state('');

  let tagsInput = $derived(localEntity?.tags?.join(', ') ?? '');

  let hasSpiderAxes = $derived(schema.spiderAxes.length > 0);

  // Sync external entity changes into local state (including initial mount).
  // Reads only entitySnapshot as a reactive dependency; all writes are wrapped
  // in untrack() so this effect never re-runs because of its own writes.
  $effect(() => {
    const snap = entitySnapshot;
    untrack(() => {
      if (snap !== lastSyncedSnapshot) {
        localEntity = JSON.parse(snap);
        lastSyncedSnapshot = snap;
        isDirty = false;
      }
    });
  });

  // Debounced auto-save.
  // Reactive dependencies: isDirty, readonly, onSave, localEntity (via JSON.stringify).
  // All side-effecting writes are wrapped in untrack() so scheduling the timer
  // does not re-trigger this effect.
  $effect(() => {
    if (!isDirty || readonly || !onSave || !localEntity) return;

    // Read localEntity here (outside untrack) so the effect tracks it.
    const snapshot = JSON.stringify(localEntity);

    untrack(() => {
      if (snapshot === lastSyncedSnapshot) {
        isDirty = false;
        return;
      }

      if (saveTimer) clearTimeout(saveTimer);

      const entityToSave = JSON.parse(snapshot) as EntityInstance;
      saveTimer = setTimeout(() => {
        onSave(entityToSave);
        lastSyncedSnapshot = snapshot;
        isDirty = false;
        saveTimer = null;
      }, 1500);
    });

    return () => {
      if (saveTimer) clearTimeout(saveTimer);
    };
  });

  function markDirty() {
    isDirty = true;
  }

  function handleTitleInput(e: Event) {
    localEntity.title = (e.currentTarget as HTMLInputElement).value;
    markDirty();
  }

  function handleTagsInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    localEntity.tags = raw
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t.length > 0);
    markDirty();
  }

  function handleFieldChange(fieldName: string, value: unknown) {
    localEntity.fields = { ...localEntity.fields, [fieldName]: value };
    markDirty();
  }

  function handleSpiderChange(values: Record<string, number>) {
    localEntity.spiderValues = values;
    markDirty();
  }
</script>

{#if localEntity}
<div class="entity-form">
  <!-- Title -->
  <div class="form-section title-section">
    <input
      class="title-input"
      type="text"
      value={localEntity.title}
      placeholder="Entity title..."
      disabled={readonly}
      oninput={handleTitleInput}
      aria-label="Entity title"
    />
    {#if isDirty}
      <span class="unsaved-indicator" aria-label="Unsaved changes">Unsaved</span>
    {/if}
  </div>

  <!-- Tags -->
  <div class="form-section tags-section">
    <label class="section-label" for="entity-tags">Tags</label>
    <input
      id="entity-tags"
      class="tags-input"
      type="text"
      value={tagsInput}
      placeholder="tag1, tag2, tag3..."
      disabled={readonly}
      oninput={handleTagsInput}
    />
    {#if localEntity.tags.length > 0}
      <div class="tags-preview">
        {#each localEntity.tags as tag}
          <span class="tag-chip">{tag}</span>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Divider -->
  <hr class="form-divider" />

  <!-- Fields -->
  {#if schema.fields.length > 0}
    <div class="form-section fields-section">
      <h3 class="section-heading">Fields</h3>
      <div class="fields-grid">
        {#each schema.fields as field (field.name)}
          <FieldRenderer
            {field}
            value={localEntity.fields[field.name] ?? field.defaultValue}
            onchange={(val) => handleFieldChange(field.name, val)}
            {readonly}
          />
        {/each}
      </div>
    </div>
  {/if}

  <!-- Spider Chart -->
  {#if hasSpiderAxes}
    <hr class="form-divider" />

    <div class="form-section spider-section">
      <h3 class="section-heading">Characteristics</h3>
      <div class="spider-wrapper">
        <SpiderChart
          axes={schema.spiderAxes}
          values={localEntity.spiderValues}
          onChange={handleSpiderChange}
          color={schema.color}
          {readonly}
        />
      </div>
    </div>
  {/if}
</div>
{/if}

<style>
  .entity-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    max-width: 640px;
    width: 100%;
  }

  /* -- Title Section ------------------------------------------------------- */

  .title-section {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .title-input {
    flex: 1;
    padding: var(--spacing-sm) 0;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-tight);
    transition: border-color var(--transition-fast);
  }

  .title-input:focus {
    outline: none;
    border-bottom-color: var(--accent-primary);
    box-shadow: none;
  }

  .title-input:disabled {
    opacity: 0.8;
    cursor: not-allowed;
  }

  .title-input::placeholder {
    color: var(--text-tertiary);
    font-weight: var(--font-weight-normal);
  }

  .unsaved-indicator {
    flex-shrink: 0;
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-full);
    background: var(--color-warning);
    color: var(--text-inverse);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* -- Tags Section -------------------------------------------------------- */

  .tags-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .tags-input {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-size: var(--font-size-base);
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .tags-input:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px rgba(59, 111, 212, 0.15);
  }

  .tags-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    background: var(--bg-secondary);
  }

  .tags-input::placeholder {
    color: var(--text-tertiary);
  }

  .tags-preview {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
  }

  /* -- Divider ------------------------------------------------------------- */

  .form-divider {
    border: none;
    border-top: 1px solid var(--border-secondary);
    margin: var(--spacing-xs) 0;
  }

  /* -- Section Headings ---------------------------------------------------- */

  .section-heading {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--spacing-sm);
  }

  /* -- Fields Grid --------------------------------------------------------- */

  .fields-section {
    display: flex;
    flex-direction: column;
  }

  .fields-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  /* -- Spider Section ------------------------------------------------------ */

  .spider-section {
    display: flex;
    flex-direction: column;
  }

  .spider-wrapper {
    max-width: 360px;
    width: 100%;
    align-self: center;
  }
</style>
