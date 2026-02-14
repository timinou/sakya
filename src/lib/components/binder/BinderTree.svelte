<script lang="ts">
  import type { ComponentType } from 'svelte';
  import { Users, MapPin, Package, Lightbulb, File } from 'lucide-svelte';
  import { entityStore, projectState } from '$lib/stores';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';
  import ManuscriptSection from './ManuscriptSection.svelte';
  import NotesSection from './NotesSection.svelte';

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = ComponentType<any>;

  interface Props {
    onSelectEntity?: (schemaType: string, slug: string) => void;
    onSelectChapter?: (slug: string) => void;
    onSelectNote?: (slug: string) => void;
  }

  let {
    onSelectEntity,
    onSelectChapter,
    onSelectNote,
  }: Props = $props();

  // Section open states
  let entitySectionsOpen = $state<Record<string, boolean>>({});

  // Selected item tracking
  let selectedItem = $state<{ type: string; slug: string } | null>(null);

  // Inline entity creation state
  let isCreatingEntity = $state<Record<string, boolean>>({});
  let newEntityTitle = $state<Record<string, string>>({});
  let entityInputRefs = $state<Record<string, HTMLInputElement | null>>({});

  // Icon mapping for known entity types
  const entityIcons: Record<string, IconComponent> = {
    character: Users,
    place: MapPin,
    item: Package,
    idea: Lightbulb,
  };

  // Color mapping for known entity types
  const entityColors: Record<string, string> = {
    character: '#7c4dbd',
    place: '#2e8b57',
    item: '#c28a1e',
    idea: '#3a7bd5',
  };

  function getIconForType(entityType: string): IconComponent {
    return entityIcons[entityType] ?? File;
  }

  function getColorForType(entityType: string): string | undefined {
    return entityColors[entityType];
  }

  function getSectionTitle(schema: { name: string; entityType: string }): string {
    const name = schema.name;
    if (name.endsWith('s') || name.endsWith('x') || name.endsWith('z')) {
      return name + 'es';
    }
    if (name.endsWith('y') && !['a', 'e', 'i', 'o', 'u'].includes(name[name.length - 2])) {
      return name.slice(0, -1) + 'ies';
    }
    return name + 's';
  }

  function toggleSection(entityType: string): void {
    entitySectionsOpen[entityType] = !(entitySectionsOpen[entityType] ?? true);
  }

  function isSectionOpen(entityType: string): boolean {
    return entitySectionsOpen[entityType] ?? true;
  }

  function handleSelectEntity(schemaType: string, slug: string): void {
    selectedItem = { type: schemaType, slug };
    onSelectEntity?.(schemaType, slug);
  }

  function startCreateEntity(schemaType: string): void {
    isCreatingEntity[schemaType] = true;
    newEntityTitle[schemaType] = '';
  }

  async function confirmCreateEntity(schemaType: string): Promise<void> {
    const title = (newEntityTitle[schemaType] ?? '').trim();
    if (!title || !projectState.projectPath) {
      cancelCreateEntity(schemaType);
      return;
    }
    try {
      await entityStore.createEntity(projectState.projectPath, schemaType, title);
      await entityStore.loadEntities(projectState.projectPath, schemaType);
    } catch (e) {
      console.error('Failed to create entity:', e);
    }
    cancelCreateEntity(schemaType);
  }

  function cancelCreateEntity(schemaType: string): void {
    isCreatingEntity[schemaType] = false;
    newEntityTitle[schemaType] = '';
  }

  function handleEntityInputKeydown(e: KeyboardEvent, schemaType: string): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmCreateEntity(schemaType);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelCreateEntity(schemaType);
    }
  }

  // Auto-focus input when creating entity
  $effect(() => {
    for (const [type, creating] of Object.entries(isCreatingEntity)) {
      if (creating && entityInputRefs[type]) {
        entityInputRefs[type]!.focus();
      }
    }
  });

  // Listen for sakya:create-entity custom events from WelcomeCard
  $effect(() => {
    function handleCreateEntityEvent(e: Event) {
      const detail = (e as CustomEvent).detail;
      if (detail?.entityType) {
        startCreateEntity(detail.entityType);
      }
    }
    window.addEventListener('sakya:create-entity', handleCreateEntityEvent);
    return () => {
      window.removeEventListener('sakya:create-entity', handleCreateEntityEvent);
    };
  });

  // Auto-load schemas on mount if not loaded
  $effect(() => {
    const path = projectState.projectPath;
    if (path && !entityStore.schemasLoaded && !entityStore.isLoading) {
      entityStore.loadSchemas(path).catch((e) => {
        console.error('Failed to load entity schemas:', e);
      });
    }
  });

  // Initialize section open states for new schemas
  $effect(() => {
    for (const schema of entityStore.schemaSummaries) {
      if (entitySectionsOpen[schema.entityType] === undefined) {
        entitySectionsOpen[schema.entityType] = true;
      }
    }
  });

  // Load entities when schemas are available
  $effect(() => {
    const schemas = entityStore.schemaSummaries;
    const path = projectState.projectPath;
    if (!path || schemas.length === 0) return;

    for (const schema of schemas) {
      if (!entityStore.entitiesByType[schema.entityType]) {
        entityStore.loadEntities(path, schema.entityType);
      }
    }
  });
</script>

<nav class="binder-tree" aria-label="Project binder">
  <!-- Manuscript section -->
  <ManuscriptSection {onSelectChapter} />

  <!-- Entity sections from loaded schemas -->
  {#each entityStore.schemaSummaries as schema (schema.entityType)}
    {@const entityType = schema.entityType}
    {@const entities = entityStore.entitiesByType[entityType] ?? []}
    <BinderSection
      title={getSectionTitle(schema)}
      icon={getIconForType(entityType)}
      color={getColorForType(entityType)}
      count={entities.length}
      isOpen={isSectionOpen(entityType)}
      onAdd={() => startCreateEntity(entityType)}
      ontoggle={() => toggleSection(entityType)}
    >
      {#if entities.length === 0 && !isCreatingEntity[entityType]}
        <div class="placeholder-text">
          No {schema.name.toLowerCase()}s yet
        </div>
      {/if}

      {#each entities as entity (entity.slug)}
        <BinderItem
          label={entity.title}
          icon={getIconForType(entityType)}
          color={getColorForType(entityType)}
          isSelected={selectedItem?.type === entityType && selectedItem?.slug === entity.slug}
          isActive={entityStore.currentEntity?.slug === entity.slug}
          onclick={() => handleSelectEntity(entityType, entity.slug)}
          indent={1}
        />
      {/each}

      {#if isCreatingEntity[entityType]}
        <div class="inline-input-wrapper">
          <input
            bind:this={entityInputRefs[entityType]}
            bind:value={newEntityTitle[entityType]}
            class="inline-input"
            type="text"
            placeholder="{schema.name} name..."
            onkeydown={(e) => handleEntityInputKeydown(e, entityType)}
            onblur={() => confirmCreateEntity(entityType)}
          />
        </div>
      {/if}
    </BinderSection>
  {/each}

  <!-- Notes section -->
  <NotesSection {onSelectNote} />

  {#if entityStore.isLoading}
    <div class="loading-indicator">
      <span class="loading-dot"></span>
      Loading...
    </div>
  {/if}
</nav>

<style>
  .binder-tree {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .placeholder-text {
    padding: var(--spacing-xs) var(--spacing-sm);
    padding-left: calc(var(--spacing-sm) + 16px + var(--spacing-xs));
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    font-style: italic;
  }

  .inline-input-wrapper {
    padding: 2px var(--spacing-xs);
    padding-left: calc(8px + 1 * 16px);
  }

  .inline-input {
    width: 100%;
    padding: 3px var(--spacing-xs);
    border: 1px solid var(--border-primary, #555);
    border-radius: var(--radius-sm);
    background: var(--bg-primary, #1e1e1e);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: inherit;
    outline: none;
  }

  .inline-input:focus {
    border-color: var(--accent-primary, #7c4dbd);
  }

  .inline-input::placeholder {
    color: var(--text-tertiary);
  }

  .loading-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }

  .loading-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background-color: var(--accent-primary);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }
</style>
