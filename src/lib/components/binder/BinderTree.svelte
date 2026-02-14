<script lang="ts">
  import type { ComponentType } from 'svelte';
  import {
    Users, MapPin, Package, Lightbulb, File, Plus, Pencil, Trash2, Settings,
    Star, Heart, Sword, Shield, Crown, Flame, Globe, BookOpen, Scroll,
    Gem, Skull, Wand2, Castle, Compass, Anchor, Feather, Zap, Clock,
    Eye, Key, Map, Music, Palette, Puzzle, Target, TreePine, Mountain,
  } from 'lucide-svelte';
  import { entityStore, editorState, projectState } from '$lib/stores';
  import BinderSection from './BinderSection.svelte';
  import BinderItem from './BinderItem.svelte';
  import ManuscriptSection from './ManuscriptSection.svelte';
  import NotesSection from './NotesSection.svelte';
  import ContextMenu from '$lib/components/common/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

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

  // Context menu state — entity items
  let entityContextMenu = $state<{ x: number; y: number; slug: string; title: string; schemaType: string } | null>(null);

  // Context menu state — section headers
  let sectionContextMenu = $state<{ x: number; y: number; schemaType: string; sectionTitle: string } | null>(null);

  // Delete entity state
  let deleteTarget = $state<{ slug: string; title: string; schemaType: string } | null>(null);

  // Delete entity type state
  let deleteTypeTarget = $state<{ schemaType: string; sectionTitle: string } | null>(null);

  // Rename state
  let renamingKey = $state<string | null>(null);  // "schemaType:slug" format
  let renameValue = $state('');
  let renameInputEl = $state<HTMLInputElement | null>(null);

  // Icon mapping for known entity types (hardcoded fallbacks)
  const entityIcons: Record<string, IconComponent> = {
    character: Users,
    place: MapPin,
    item: Package,
    idea: Lightbulb,
  };

  // Color mapping for known entity types (hardcoded fallbacks)
  const entityColors: Record<string, string> = {
    character: '#7c4dbd',
    place: '#2e8b57',
    item: '#c28a1e',
    idea: '#3a7bd5',
  };

  // Map schema icon strings to Lucide components
  const iconStringMap: Record<string, IconComponent> = {
    users: Users,
    'map-pin': MapPin,
    package: Package,
    lightbulb: Lightbulb,
    file: File,
    star: Star,
    heart: Heart,
    sword: Sword,
    shield: Shield,
    crown: Crown,
    flame: Flame,
    globe: Globe,
    'book-open': BookOpen,
    scroll: Scroll,
    gem: Gem,
    skull: Skull,
    wand: Wand2,
    castle: Castle,
    compass: Compass,
    anchor: Anchor,
    feather: Feather,
    zap: Zap,
    clock: Clock,
    eye: Eye,
    key: Key,
    map: Map,
    music: Music,
    palette: Palette,
    puzzle: Puzzle,
    target: Target,
    'tree-pine': TreePine,
    mountain: Mountain,
    settings: Settings,
    pencil: Pencil,
    trash: Trash2,
    plus: Plus,
  };

  function getIconForType(entityType: string): IconComponent {
    // Check schema cache for dynamic icon
    const cached = entityStore.schemaCache[entityType];
    if (cached?.icon) {
      const mapped = iconStringMap[cached.icon];
      if (mapped) return mapped;
    }
    // Fall back to hardcoded map
    return entityIcons[entityType] ?? File;
  }

  function getColorForType(entityType: string): string | undefined {
    // Check schema cache for dynamic color
    const cached = entityStore.schemaCache[entityType];
    if (cached?.color) return cached.color;
    // Fall back to hardcoded map
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

  // --- Entity item context menu ---
  function handleEntityContextMenu(e: MouseEvent, schemaType: string, slug: string, title: string): void {
    e.preventDefault();
    entityContextMenu = { x: e.clientX, y: e.clientY, slug, title, schemaType };
  }

  function closeEntityContextMenu(): void {
    entityContextMenu = null;
  }

  function getEntityContextMenuItems(slug: string, title: string, schemaType: string) {
    return [
      { label: 'Rename', icon: Pencil, onclick: () => startRename(schemaType, slug, title) },
      { label: '', separator: true },
      { label: 'Delete', icon: Trash2, onclick: () => handleDeleteRequest(schemaType, slug, title) },
    ];
  }

  // --- Section header context menu ---
  function handleSectionContextMenu(e: MouseEvent, schemaType: string, sectionTitle: string): void {
    e.preventDefault();
    sectionContextMenu = { x: e.clientX, y: e.clientY, schemaType, sectionTitle };
  }

  function closeSectionContextMenu(): void {
    sectionContextMenu = null;
  }

  function getSectionContextMenuItems(schemaType: string, sectionTitle: string) {
    return [
      { label: 'Edit Type...', icon: Settings, onclick: () => handleEditType(schemaType) },
      { label: '', separator: true },
      { label: 'New Entity Type...', icon: Plus, onclick: () => handleNewType() },
      { label: '', separator: true },
      { label: 'Delete Type', icon: Trash2, onclick: () => handleDeleteTypeRequest(schemaType, sectionTitle) },
    ];
  }

  // --- Edit Type / New Entity Type dispatchers ---
  function handleEditType(entityType: string): void {
    closeSectionContextMenu();
    window.dispatchEvent(new CustomEvent('sakya:edit-schema', { detail: { entityType } }));
  }

  function handleNewType(): void {
    closeSectionContextMenu();
    window.dispatchEvent(new CustomEvent('sakya:new-schema'));
  }

  // --- Delete entity ---
  function handleDeleteRequest(schemaType: string, slug: string, title: string): void {
    closeEntityContextMenu();
    deleteTarget = { slug, title, schemaType };
  }

  async function confirmDelete(): Promise<void> {
    if (!deleteTarget || !projectState.projectPath) return;
    const { slug, schemaType } = deleteTarget;
    try {
      // Close open tab for this entity
      const tabId = `entity:${slug}`;
      editorState.closeTab(tabId);
      await entityStore.deleteEntity(projectState.projectPath, schemaType, slug);
      await entityStore.loadEntities(projectState.projectPath, schemaType);
    } catch (e) {
      console.error('Failed to delete entity:', e);
    }
    deleteTarget = null;
  }

  function cancelDelete(): void {
    deleteTarget = null;
  }

  // --- Delete entity type ---
  function handleDeleteTypeRequest(schemaType: string, sectionTitle: string): void {
    closeSectionContextMenu();
    deleteTypeTarget = { schemaType, sectionTitle };
  }

  async function confirmDeleteType(): Promise<void> {
    if (!deleteTypeTarget || !projectState.projectPath) return;
    const { schemaType } = deleteTypeTarget;
    try {
      // Close all open tabs for entities of this type
      const entities = entityStore.entitiesByType[schemaType] ?? [];
      for (const entity of entities) {
        editorState.closeTab(`entity:${entity.slug}`);
      }
      await entityStore.deleteSchema(projectState.projectPath, schemaType);
      await entityStore.loadSchemas(projectState.projectPath);
    } catch (e) {
      console.error('Failed to delete entity type:', e);
    }
    deleteTypeTarget = null;
  }

  function cancelDeleteType(): void {
    deleteTypeTarget = null;
  }

  // --- Rename entity ---
  function startRename(schemaType: string, slug: string, title: string): void {
    closeEntityContextMenu();
    renamingKey = `${schemaType}:${slug}`;
    renameValue = title;
  }

  async function confirmRename(): Promise<void> {
    const newTitle = renameValue.trim();
    if (!newTitle || !renamingKey || !projectState.projectPath) {
      cancelRename();
      return;
    }
    const [schemaType, slug] = renamingKey.split(':');
    // Check if title actually changed
    const entities = entityStore.entitiesByType[schemaType] ?? [];
    const original = entities.find(e => e.slug === slug);
    if (!original || newTitle === original.title) {
      cancelRename();
      return;
    }
    try {
      await entityStore.renameEntity(projectState.projectPath, schemaType, slug, newTitle);
      await entityStore.loadEntities(projectState.projectPath, schemaType);
    } catch (e) {
      console.error('Failed to rename entity:', e);
    }
    cancelRename();
  }

  function cancelRename(): void {
    renamingKey = null;
    renameValue = '';
  }

  function handleRenameKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmRename();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelRename();
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

  // Auto-focus rename input
  $effect(() => {
    if (renamingKey && renameInputEl) {
      renameInputEl.focus();
      renameInputEl.select();
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
      oncontextmenu={(e) => handleSectionContextMenu(e, entityType, getSectionTitle(schema))}
    >
      {#if entities.length === 0 && !isCreatingEntity[entityType]}
        <button class="placeholder-cta" type="button" onclick={() => startCreateEntity(entityType)}>
          <Plus size={12} /> Add first {schema.name.toLowerCase()}
        </button>
      {/if}

      {#each entities as entity (entity.slug)}
        {#if renamingKey === `${entityType}:${entity.slug}`}
          <div class="inline-input-wrapper">
            <input
              bind:this={renameInputEl}
              bind:value={renameValue}
              class="inline-input rename-input"
              type="text"
              placeholder="Entity name..."
              onkeydown={handleRenameKeydown}
              onblur={confirmRename}
            />
          </div>
        {:else}
          <BinderItem
            label={entity.title}
            icon={getIconForType(entityType)}
            color={getColorForType(entityType)}
            isSelected={selectedItem?.type === entityType && selectedItem?.slug === entity.slug}
            isActive={entityStore.currentEntity?.slug === entity.slug}
            onclick={() => handleSelectEntity(entityType, entity.slug)}
            oncontextmenu={(e) => handleEntityContextMenu(e, entityType, entity.slug, entity.title)}
            indent={1}
          />
        {/if}
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

<!-- Entity item context menu -->
{#if entityContextMenu}
  <ContextMenu
    items={getEntityContextMenuItems(entityContextMenu.slug, entityContextMenu.title, entityContextMenu.schemaType)}
    x={entityContextMenu.x}
    y={entityContextMenu.y}
    onClose={closeEntityContextMenu}
  />
{/if}

<!-- Section header context menu -->
{#if sectionContextMenu}
  <ContextMenu
    items={getSectionContextMenuItems(sectionContextMenu.schemaType, sectionContextMenu.sectionTitle)}
    x={sectionContextMenu.x}
    y={sectionContextMenu.y}
    onClose={closeSectionContextMenu}
  />
{/if}

<!-- Delete entity confirmation -->
<ConfirmDialog
  isOpen={deleteTarget !== null}
  title="Delete Entity"
  message={deleteTarget ? `Are you sure you want to delete "${deleteTarget.title}"? This action cannot be undone.` : ''}
  confirmLabel="Delete"
  destructive={true}
  onConfirm={confirmDelete}
  onCancel={cancelDelete}
/>

<!-- Delete entity type confirmation -->
<ConfirmDialog
  isOpen={deleteTypeTarget !== null}
  title="Delete Entity Type"
  message={deleteTypeTarget ? `Are you sure you want to delete the "${deleteTypeTarget.sectionTitle}" type? All entities of this type will be affected. This action cannot be undone.` : ''}
  confirmLabel="Delete Type"
  destructive={true}
  onConfirm={confirmDeleteType}
  onCancel={cancelDeleteType}
/>

<style>
  .binder-tree {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .placeholder-cta {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    padding-left: calc(var(--spacing-sm) + 16px + var(--spacing-xs));
    border: none;
    background: transparent;
    font-size: var(--font-size-xs);
    font-style: italic;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background-color var(--transition-fast),
      transform var(--transition-fast);
  }

  .placeholder-cta:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    transform: translateX(2px);
  }

  .placeholder-cta :global(svg) {
    opacity: 0.6;
    transition: opacity var(--transition-fast);
  }

  .placeholder-cta:hover :global(svg) {
    opacity: 1;
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

  .rename-input {
    border-color: var(--accent-primary, #7c4dbd);
    box-shadow: 0 0 0 1px var(--accent-primary, #7c4dbd);
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
