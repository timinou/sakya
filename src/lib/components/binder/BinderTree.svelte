<script lang="ts">
  import type { ComponentType } from 'svelte';
  import { BookOpen, Users, MapPin, Package, Lightbulb, File } from 'lucide-svelte';
  import { entityStore, projectState } from '$lib/stores';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = ComponentType<any>;

  interface Props {
    onSelectEntity?: (schemaType: string, slug: string) => void;
    onCreateEntity?: (schemaType: string) => void;
    onSelectChapter?: (slug: string) => void;
  }

  let {
    onSelectEntity,
    onCreateEntity,
    onSelectChapter,
  }: Props = $props();

  // Section open states
  let manuscriptOpen = $state(true);
  let entitySectionsOpen = $state<Record<string, boolean>>({});

  // Selected item tracking
  let selectedItem = $state<{ type: string; slug: string } | null>(null);

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
    // Pluralize the schema name for the section header
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

  function handleCreateEntity(schemaType: string): void {
    onCreateEntity?.(schemaType);
  }

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
      // Only load if not already cached
      if (!entityStore.entitiesByType[schema.entityType]) {
        entityStore.loadEntities(path, schema.entityType);
      }
    }
  });
</script>

<nav class="binder-tree" aria-label="Project binder">
  <!-- Manuscript section (placeholder for ITEM-031) -->
  <BinderSection
    title="Manuscript"
    icon={BookOpen}
    bind:isOpen={manuscriptOpen}
  >
    <div class="placeholder-text">
      No chapters yet
    </div>
  </BinderSection>

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
      onAdd={() => handleCreateEntity(entityType)}
      ontoggle={() => toggleSection(entityType)}
    >
      {#if entities.length === 0}
        <div class="placeholder-text">
          No {schema.name.toLowerCase()}s yet
        </div>
      {:else}
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
      {/if}
    </BinderSection>
  {/each}

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
