<script lang="ts">
  import type { EntitySchema, EntityField, SpiderAxis, FieldType } from '$lib/types/entity';
  import {
    Plus,
    ChevronUp,
    ChevronDown,
    Trash2,
    ChevronRight,
    Code,
  } from 'lucide-svelte';
  import yaml from 'yaml';

  interface Props {
    schema: EntitySchema;
    isNew?: boolean;
    onSave?: (schema: EntitySchema) => void;
    onCancel?: () => void;
  }

  let { schema, isNew = false, onSave, onCancel }: Props = $props();

  // Deep clone the input schema into local mutable state
  let localSchema = $state<EntitySchema>(structuredClone(schema));

  // Track which field/axis cards are expanded
  let expandedFields = $state<Set<number>>(new Set());
  let expandedAxes = $state<Set<number>>(new Set());
  let yamlPreviewOpen = $state(false);

  const FIELD_TYPES: { value: FieldType; label: string }[] = [
    { value: 'short_text', label: 'Short Text' },
    { value: 'long_text', label: 'Long Text' },
    { value: 'number', label: 'Number' },
    { value: 'select', label: 'Select' },
    { value: 'date', label: 'Date' },
    { value: 'boolean', label: 'Boolean' },
  ];

  // Auto-slugify the entityType from name
  function slugify(text: string): string {
    return text
      .toLowerCase()
      .trim()
      .replace(/[^\w\s-]/g, '')
      .replace(/[\s_]+/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '');
  }

  function handleNameChange(value: string) {
    localSchema.name = value;
    if (isNew) {
      localSchema.entityType = slugify(value);
    }
  }

  // YAML preview — use $state.snapshot() to avoid traversing the deep reactive proxy
  let yamlPreview = $derived(yaml.stringify($state.snapshot(localSchema)));

  // --- Field management ---
  function addField() {
    const newField: EntityField = {
      name: '',
      label: '',
      fieldType: 'short_text',
      required: false,
    };
    localSchema.fields = [...localSchema.fields, newField];
    expandedFields.add(localSchema.fields.length - 1);
  }

  function removeField(index: number) {
    localSchema.fields = localSchema.fields.filter((_, i) => i !== index);
    expandedFields.delete(index);
    // Rebuild expanded set with shifted indices
    const newExpanded = new Set<number>();
    for (const i of expandedFields) {
      if (i > index) newExpanded.add(i - 1);
      else newExpanded.add(i);
    }
    expandedFields = newExpanded;
  }

  function moveField(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= localSchema.fields.length) return;
    const fields = [...localSchema.fields];
    [fields[index], fields[target]] = [fields[target], fields[index]];
    localSchema.fields = fields;
    // Update expanded tracking
    const newExpanded = new Set<number>();
    for (const i of expandedFields) {
      if (i === index) newExpanded.add(target);
      else if (i === target) newExpanded.add(index);
      else newExpanded.add(i);
    }
    expandedFields = newExpanded;
  }

  function toggleField(index: number) {
    if (expandedFields.has(index)) {
      expandedFields.delete(index);
      expandedFields = new Set(expandedFields);
    } else {
      expandedFields.add(index);
      expandedFields = new Set(expandedFields);
    }
  }

  // --- Select options management ---
  function addOption(fieldIndex: number) {
    const field = localSchema.fields[fieldIndex];
    if (!field.options) field.options = [];
    field.options = [...field.options, ''];
  }

  function removeOption(fieldIndex: number, optionIndex: number) {
    const field = localSchema.fields[fieldIndex];
    if (field.options) {
      field.options = field.options.filter((_, i) => i !== optionIndex);
    }
  }

  function updateOption(fieldIndex: number, optionIndex: number, value: string) {
    const field = localSchema.fields[fieldIndex];
    if (field.options) {
      field.options[optionIndex] = value;
    }
  }

  // --- Axis management ---
  function addAxis() {
    const newAxis: SpiderAxis = {
      name: '',
      min: 0,
      max: 10,
      default: 5,
    };
    localSchema.spiderAxes = [...localSchema.spiderAxes, newAxis];
    expandedAxes.add(localSchema.spiderAxes.length - 1);
  }

  function removeAxis(index: number) {
    localSchema.spiderAxes = localSchema.spiderAxes.filter((_, i) => i !== index);
    expandedAxes.delete(index);
    const newExpanded = new Set<number>();
    for (const i of expandedAxes) {
      if (i > index) newExpanded.add(i - 1);
      else newExpanded.add(i);
    }
    expandedAxes = newExpanded;
  }

  function moveAxis(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= localSchema.spiderAxes.length) return;
    const axes = [...localSchema.spiderAxes];
    [axes[index], axes[target]] = [axes[target], axes[index]];
    localSchema.spiderAxes = axes;
    const newExpanded = new Set<number>();
    for (const i of expandedAxes) {
      if (i === index) newExpanded.add(target);
      else if (i === target) newExpanded.add(index);
      else newExpanded.add(i);
    }
    expandedAxes = newExpanded;
  }

  function toggleAxis(index: number) {
    if (expandedAxes.has(index)) {
      expandedAxes.delete(index);
      expandedAxes = new Set(expandedAxes);
    } else {
      expandedAxes.add(index);
      expandedAxes = new Set(expandedAxes);
    }
  }

  // --- Save / Cancel ---
  function handleSave() {
    onSave?.(structuredClone(localSchema));
  }

  function handleCancel() {
    onCancel?.();
  }
</script>

<div class="schema-editor">
  <!-- ===== Metadata Section ===== -->
  <section class="section">
    <h3 class="section-title">Schema Metadata</h3>

    <div class="form-grid">
      <label class="form-field">
        <span class="form-label">Name</span>
        <input
          type="text"
          value={localSchema.name}
          oninput={(e) => handleNameChange(e.currentTarget.value)}
          placeholder="e.g. Character, Location"
        />
      </label>

      <label class="form-field">
        <span class="form-label">Entity Type (slug)</span>
        <input
          type="text"
          value={localSchema.entityType}
          readonly={!isNew}
          class:readonly={!isNew}
          placeholder="auto-generated"
        />
      </label>

      <label class="form-field">
        <span class="form-label">Icon</span>
        <input
          type="text"
          bind:value={localSchema.icon}
          placeholder="e.g. user, map-pin"
        />
      </label>

      <label class="form-field">
        <span class="form-label">Color</span>
        <input
          type="text"
          bind:value={localSchema.color}
          placeholder="e.g. #7c4dbd"
        />
      </label>
    </div>

    <label class="form-field">
      <span class="form-label">Description</span>
      <textarea
        bind:value={localSchema.description}
        placeholder="Describe this entity type..."
        rows="2"
      ></textarea>
    </label>
  </section>

  <!-- ===== Fields Section ===== -->
  <section class="section">
    <div class="section-header">
      <h3 class="section-title">Fields</h3>
      <button type="button" class="btn-add" onclick={addField}>
        <Plus size={14} />
        Add Field
      </button>
    </div>

    {#if localSchema.fields.length === 0}
      <p class="empty-message">No fields configured. Add a field to get started.</p>
    {/if}

    <div class="card-list">
      {#each localSchema.fields as field, index (index)}
        <div class="card">
          <div class="card-header">
            <button
              type="button"
              class="card-toggle"
              onclick={() => toggleField(index)}
            >
              <span class="card-chevron" class:expanded={expandedFields.has(index)}>
                <ChevronRight size={14} />
              </span>
              <span class="card-title">
                {field.label || field.name || `Field ${index + 1}`}
              </span>
              <span class="card-badge">{field.fieldType}</span>
            </button>

            <div class="card-actions">
              <button
                type="button"
                class="btn-icon"
                onclick={() => moveField(index, -1)}
                disabled={index === 0}
                title="Move up"
              >
                <ChevronUp size={14} />
              </button>
              <button
                type="button"
                class="btn-icon"
                onclick={() => moveField(index, 1)}
                disabled={index === localSchema.fields.length - 1}
                title="Move down"
              >
                <ChevronDown size={14} />
              </button>
              <button
                type="button"
                class="btn-icon btn-danger"
                onclick={() => removeField(index)}
                title="Remove field"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>

          {#if expandedFields.has(index)}
            <div class="card-body">
              <div class="form-grid">
                <label class="form-field">
                  <span class="form-label">Name (key)</span>
                  <input
                    type="text"
                    bind:value={field.name}
                    placeholder="field_name"
                  />
                </label>

                <label class="form-field">
                  <span class="form-label">Label</span>
                  <input
                    type="text"
                    bind:value={field.label}
                    placeholder="Display Label"
                  />
                </label>

                <label class="form-field">
                  <span class="form-label">Field Type</span>
                  <select bind:value={field.fieldType}>
                    {#each FIELD_TYPES as ft}
                      <option value={ft.value}>{ft.label}</option>
                    {/each}
                  </select>
                </label>

                <label class="form-field">
                  <span class="form-label">Placeholder</span>
                  <input
                    type="text"
                    bind:value={field.placeholder}
                    placeholder="Placeholder text"
                  />
                </label>
              </div>

              <label class="form-field">
                <span class="form-label">Description</span>
                <input
                  type="text"
                  bind:value={field.description}
                  placeholder="Brief description of this field"
                />
              </label>

              <label class="form-checkbox">
                <input type="checkbox" bind:checked={field.required} />
                <span>Required</span>
              </label>

              <!-- Select-specific: options editor -->
              {#if field.fieldType === 'select'}
                <div class="sub-section">
                  <div class="sub-section-header">
                    <span class="form-label">Options</span>
                    <button
                      type="button"
                      class="btn-add btn-small"
                      onclick={() => addOption(index)}
                    >
                      <Plus size={12} />
                      Add Option
                    </button>
                  </div>
                  {#if field.options && field.options.length > 0}
                    <div class="options-list">
                      {#each field.options as option, optIndex}
                        <div class="option-row">
                          <input
                            type="text"
                            value={option}
                            oninput={(e) => updateOption(index, optIndex, e.currentTarget.value)}
                            placeholder={`Option ${optIndex + 1}`}
                          />
                          <button
                            type="button"
                            class="btn-icon btn-danger"
                            onclick={() => removeOption(index, optIndex)}
                            title="Remove option"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="empty-message small">No options yet.</p>
                  {/if}
                </div>
              {/if}

              <!-- Number-specific: min/max -->
              {#if field.fieldType === 'number'}
                <div class="form-grid">
                  <label class="form-field">
                    <span class="form-label">Min</span>
                    <input
                      type="number"
                      bind:value={field.min}
                      placeholder="Minimum"
                    />
                  </label>
                  <label class="form-field">
                    <span class="form-label">Max</span>
                    <input
                      type="number"
                      bind:value={field.max}
                      placeholder="Maximum"
                    />
                  </label>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  <!-- ===== Spider Axes Section ===== -->
  <section class="section">
    <div class="section-header">
      <h3 class="section-title">Spider Axes</h3>
      <button type="button" class="btn-add" onclick={addAxis}>
        <Plus size={14} />
        Add Axis
      </button>
    </div>

    {#if localSchema.spiderAxes.length === 0}
      <p class="empty-message">No spider axes configured. Add an axis for radar chart display.</p>
    {/if}

    <div class="card-list">
      {#each localSchema.spiderAxes as axis, index (index)}
        <div class="card">
          <div class="card-header">
            <button
              type="button"
              class="card-toggle"
              onclick={() => toggleAxis(index)}
            >
              <span class="card-chevron" class:expanded={expandedAxes.has(index)}>
                <ChevronRight size={14} />
              </span>
              <span class="card-title">
                {axis.name || `Axis ${index + 1}`}
              </span>
              <span class="card-badge">{axis.min}–{axis.max}</span>
            </button>

            <div class="card-actions">
              <button
                type="button"
                class="btn-icon"
                onclick={() => moveAxis(index, -1)}
                disabled={index === 0}
                title="Move up"
              >
                <ChevronUp size={14} />
              </button>
              <button
                type="button"
                class="btn-icon"
                onclick={() => moveAxis(index, 1)}
                disabled={index === localSchema.spiderAxes.length - 1}
                title="Move down"
              >
                <ChevronDown size={14} />
              </button>
              <button
                type="button"
                class="btn-icon btn-danger"
                onclick={() => removeAxis(index)}
                title="Remove axis"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>

          {#if expandedAxes.has(index)}
            <div class="card-body">
              <div class="form-grid">
                <label class="form-field">
                  <span class="form-label">Name</span>
                  <input
                    type="text"
                    bind:value={axis.name}
                    placeholder="e.g. Strength, Intelligence"
                  />
                </label>

                <label class="form-field">
                  <span class="form-label">Min</span>
                  <input
                    type="number"
                    bind:value={axis.min}
                  />
                </label>

                <label class="form-field">
                  <span class="form-label">Max</span>
                  <input
                    type="number"
                    bind:value={axis.max}
                  />
                </label>

                <label class="form-field">
                  <span class="form-label">Default</span>
                  <input
                    type="number"
                    bind:value={axis.default}
                  />
                </label>
              </div>

              <label class="form-field">
                <span class="form-label">Description</span>
                <input
                  type="text"
                  bind:value={axis.description}
                  placeholder="What does this axis measure?"
                />
              </label>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  <!-- ===== YAML Preview ===== -->
  <section class="section">
    <button
      type="button"
      class="section-toggle"
      onclick={() => (yamlPreviewOpen = !yamlPreviewOpen)}
    >
      <span class="card-chevron" class:expanded={yamlPreviewOpen}>
        <ChevronRight size={14} />
      </span>
      <Code size={14} />
      <span>YAML Preview</span>
    </button>

    {#if yamlPreviewOpen}
      <pre class="yaml-preview">{yamlPreview}</pre>
    {/if}
  </section>

  <!-- ===== Action Buttons ===== -->
  <div class="actions">
    <button type="button" class="btn-secondary" onclick={handleCancel}>
      Cancel
    </button>
    <button type="button" class="btn-primary" onclick={handleSave}>
      Save Schema
    </button>
  </div>
</div>

<style>
  .schema-editor {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg);
    overflow-y: auto;
    max-height: 100%;
  }

  /* --- Sections --- */
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .section-title {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    background: none;
    border: none;
    padding: var(--spacing-xs) 0;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: color var(--transition-fast);
  }

  .section-toggle:hover {
    color: var(--text-primary);
    border-color: transparent;
    box-shadow: none;
  }

  /* --- Form Fields --- */
  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-sm);
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .form-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .form-field input,
  .form-field textarea,
  .form-field select {
    width: 100%;
  }

  .form-field select {
    border: 1px solid var(--border-primary);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-base);
    color: var(--text-primary);
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .form-field select:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px rgba(59, 111, 212, 0.15);
  }

  .form-checkbox {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    cursor: pointer;
  }

  .form-checkbox input[type='checkbox'] {
    width: auto;
    accent-color: var(--accent-primary);
  }

  .readonly {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* --- Cards (Fields & Axes) --- */
  .card-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .card {
    border: 1px solid var(--border-primary);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-sm);
    min-height: 36px;
  }

  .card-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    padding: 0;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    text-align: left;
  }

  .card-toggle:hover {
    border-color: transparent;
    box-shadow: none;
  }

  .card-chevron {
    display: flex;
    align-items: center;
    transition: transform var(--transition-fast);
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .card-chevron.expanded {
    transform: rotate(90deg);
  }

  .card-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-badge {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    background: var(--bg-secondary);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    padding-top: 0;
    border-top: 1px solid var(--border-secondary);
  }

  /* --- Buttons --- */
  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background-color var(--transition-fast);
  }

  .btn-icon:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border-color: transparent;
  }

  .btn-icon:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-icon.btn-danger:hover:not(:disabled) {
    color: var(--color-error);
    background: rgba(196, 57, 43, 0.08);
  }

  .btn-add {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px dashed var(--border-primary);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      border-color var(--transition-fast),
      background-color var(--transition-fast);
  }

  .btn-add:hover {
    color: var(--accent-primary);
    border-color: var(--accent-primary);
    background: rgba(59, 111, 212, 0.04);
    box-shadow: none;
  }

  .btn-add.btn-small {
    padding: 2px var(--spacing-xs);
    font-size: var(--font-size-xs);
  }

  .btn-primary {
    background: var(--accent-primary);
    color: var(--text-inverse);
    border-color: var(--accent-primary);
    font-weight: var(--font-weight-semibold);
  }

  .btn-primary:hover {
    opacity: 0.9;
    border-color: var(--accent-primary);
  }

  .btn-secondary {
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  /* --- Sub-sections (e.g. select options) --- */
  .sub-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
  }

  .sub-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .option-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .option-row input {
    flex: 1;
  }

  /* --- YAML Preview --- */
  .yaml-preview {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-secondary);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 300px;
    overflow-y: auto;
  }

  /* --- Actions Bar --- */
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--border-secondary);
  }

  /* --- Empty state --- */
  .empty-message {
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
    font-style: italic;
    padding: var(--spacing-sm) 0;
  }

  .empty-message.small {
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) 0;
  }
</style>
