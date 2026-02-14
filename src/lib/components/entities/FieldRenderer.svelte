<script lang="ts">
  import type { EntityField } from '$lib/types';

  interface Props {
    field: EntityField;
    value: unknown;
    onchange: (value: unknown) => void;
    readonly?: boolean;
  }

  let { field, value, onchange, readonly = false }: Props = $props();

  function handleTextInput(e: Event) {
    onchange((e.currentTarget as HTMLInputElement).value);
  }

  function handleTextareaInput(e: Event) {
    onchange((e.currentTarget as HTMLTextAreaElement).value);
  }

  function handleNumberInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    if (raw === '') {
      onchange(undefined);
      return;
    }
    const parsed = parseFloat(raw);
    if (!isNaN(parsed)) {
      onchange(parsed);
    }
  }

  function handleSelectChange(e: Event) {
    onchange((e.currentTarget as HTMLSelectElement).value);
  }

  function handleDateInput(e: Event) {
    onchange((e.currentTarget as HTMLInputElement).value);
  }

  function handleBooleanChange(e: Event) {
    onchange((e.currentTarget as HTMLInputElement).checked);
  }
</script>

<div class="field-renderer">
  <label class="field-label" for="field-{field.name}">
    <span class="label-text">{field.label}</span>
    {#if field.required}
      <span class="required-indicator" aria-label="required">*</span>
    {/if}
  </label>

  {#if field.description}
    <p class="field-description">{field.description}</p>
  {/if}

  {#if field.fieldType === 'short_text'}
    <input
      id="field-{field.name}"
      type="text"
      class="field-input"
      value={typeof value === 'string' ? value : ''}
      placeholder={field.placeholder ?? ''}
      required={field.required}
      disabled={readonly}
      oninput={handleTextInput}
    />
  {:else if field.fieldType === 'long_text'}
    <textarea
      id="field-{field.name}"
      class="field-input field-textarea"
      placeholder={field.placeholder ?? ''}
      required={field.required}
      disabled={readonly}
      oninput={handleTextareaInput}
    >{typeof value === 'string' ? value : ''}</textarea>
  {:else if field.fieldType === 'number'}
    <input
      id="field-{field.name}"
      type="number"
      class="field-input"
      value={typeof value === 'number' ? value : ''}
      placeholder={field.placeholder ?? ''}
      min={field.min}
      max={field.max}
      required={field.required}
      disabled={readonly}
      oninput={handleNumberInput}
    />
  {:else if field.fieldType === 'select'}
    <select
      id="field-{field.name}"
      class="field-input field-select"
      value={typeof value === 'string' ? value : ''}
      required={field.required}
      disabled={readonly}
      onchange={handleSelectChange}
    >
      <option value="">{field.placeholder ?? 'Select...'}</option>
      {#each field.options ?? [] as option}
        <option value={option} selected={value === option}>{option}</option>
      {/each}
    </select>
  {:else if field.fieldType === 'date'}
    <input
      id="field-{field.name}"
      type="date"
      class="field-input"
      value={typeof value === 'string' ? value : ''}
      required={field.required}
      disabled={readonly}
      oninput={handleDateInput}
    />
  {:else if field.fieldType === 'boolean'}
    <label class="checkbox-wrapper" for="field-{field.name}">
      <input
        id="field-{field.name}"
        type="checkbox"
        class="field-checkbox"
        checked={!!value}
        disabled={readonly}
        onchange={handleBooleanChange}
      />
      <span class="checkbox-label">{field.label}</span>
    </label>
  {/if}
</div>

<style>
  .field-renderer {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .field-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .required-indicator {
    color: var(--color-error);
    font-weight: var(--font-weight-bold);
    line-height: 1;
  }

  .field-description {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    line-height: var(--line-height-normal);
    margin-bottom: var(--spacing-xs);
  }

  .field-input {
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

  .field-input:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px rgba(59, 111, 212, 0.15);
  }

  .field-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    background: var(--bg-secondary);
  }

  .field-input::placeholder {
    color: var(--text-tertiary);
  }

  .field-textarea {
    min-height: 5rem;
    resize: vertical;
    line-height: var(--line-height-normal);
  }

  .field-select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%235c554e' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--spacing-sm) center;
    padding-right: calc(var(--spacing-sm) + 1.25rem);
    cursor: pointer;
  }

  .field-select:disabled {
    cursor: not-allowed;
  }

  .checkbox-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
  }

  .checkbox-wrapper:has(.field-checkbox:disabled) {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .field-checkbox {
    width: 1rem;
    height: 1rem;
    accent-color: var(--accent-primary);
    cursor: pointer;
    flex-shrink: 0;
  }

  .field-checkbox:disabled {
    cursor: not-allowed;
  }

  .checkbox-label {
    font-size: var(--font-size-sm);
  }
</style>
