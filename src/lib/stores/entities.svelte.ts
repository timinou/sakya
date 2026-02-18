import { invoke } from '@tauri-apps/api/core';
import type { EntitySchema, SchemaSummary, EntityInstance, EntitySummary } from '$lib/types';
import { StaleGuard } from './stale-guard';

class EntityStore {
  schemaSummaries = $state<SchemaSummary[]>([]);
  schemaCache = $state<Record<string, EntitySchema>>({});
  entitiesByType = $state<Record<string, EntitySummary[]>>({});
  currentEntity = $state<EntityInstance | null>(null);
  error = $state<string | null>(null);

  // --- Granular loading flags (Bug 4 fix) ---
  isLoadingSchemas = $state(false);
  isLoadingEntities = $state<Record<string, boolean>>({});
  isSaving = $state(false);

  // Convenience derived that ORs all sub-flags
  isLoading = $derived(
    this.isLoadingSchemas ||
    this.isSaving ||
    Object.values(this.isLoadingEntities).some(Boolean),
  );

  // schemasLoadedPath is a load-deduplication tracker (prevents re-fetching on re-mount), NOT a stale guard
  schemasLoadedPath = $state<string | null>(null);
  schemasLoaded = $derived(this.schemasLoadedPath !== null);

  private guard = new StaleGuard();

  async loadSchemas(projectPath: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoadingSchemas = true;
    this.error = null;
    try {
      const summaries = await invoke<SchemaSummary[]>('list_schemas', { projectPath });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.schemaSummaries = summaries;
      this.schemasLoadedPath = projectPath;
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoadingSchemas = false;
      }
    }
  }

  async getSchema(projectPath: string, schemaType: string): Promise<EntitySchema> {
    // Return cached if available
    const cached = this.schemaCache[schemaType];
    if (cached) return cached;

    const token = this.guard.snapshot(); // STALE GUARD
    const schema = await invoke<EntitySchema>('get_schema', {
      projectPath,
      schemaType,
    });
    if (this.guard.isStale(token)) return schema; // STALE GUARD — return result but skip cache write
    this.schemaCache[schemaType] = schema;
    return schema;
  }

  async saveSchema(projectPath: string, schema: EntitySchema): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('save_schema', { projectPath, schema });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.schemaCache[schema.entityType];
      // Reload summaries (inner call creates its own guard token)
      await this.loadSchemas(projectPath);
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  async deleteSchema(projectPath: string, schemaType: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('delete_schema', { projectPath, schemaType });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.schemaCache[schemaType];
      // Reload summaries (inner call creates its own guard token)
      await this.loadSchemas(projectPath);
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  // --- Per-type loading guard (Bug 2 fix) ---
  async loadEntities(projectPath: string, schemaType: string): Promise<void> {
    // Skip if already loading this type
    if (this.isLoadingEntities[schemaType]) return;
    // Skip if already loaded
    if (this.entitiesByType[schemaType]) return;

    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoadingEntities[schemaType] = true;
    this.error = null;
    try {
      const entities = await invoke<EntitySummary[]>('list_entities', {
        projectPath,
        schemaType,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.entitiesByType[schemaType] = entities;
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoadingEntities[schemaType] = false;
      }
    }
  }

  // Force-refetch entities for a type (used after create/delete/rename)
  async reloadEntities(projectPath: string, schemaType: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoadingEntities[schemaType] = true;
    this.error = null;
    try {
      const entities = await invoke<EntitySummary[]>('list_entities', {
        projectPath,
        schemaType,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.entitiesByType[schemaType] = entities;
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoadingEntities[schemaType] = false;
      }
    }
  }

  async getEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<EntityInstance> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      const entity = await invoke<EntityInstance>('get_entity', {
        projectPath,
        schemaType,
        slug,
      });
      if (this.guard.isStale(token)) return entity; // STALE GUARD — return result but skip state write
      this.currentEntity = entity;
      return entity;
    } catch (e) {
      if (this.guard.isStale(token)) throw e; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  async createEntity(
    projectPath: string,
    schemaType: string,
    title: string,
  ): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('create_entity', {
        projectPath,
        schemaType,
        title,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      // Inner call creates its own guard token
      await this.reloadEntities(projectPath, schemaType);
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  // --- In-place update for saves (Bug 3 fix) ---
  async saveEntity(projectPath: string, entity: EntityInstance): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('save_entity', {
        projectPath,
        entity,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.currentEntity = entity;
      // Update in-place instead of invalidating the whole type
      this.updateEntityInList(entity.schemaSlug, entity);
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  async renameEntity(
    projectPath: string,
    schemaType: string,
    oldSlug: string,
    newTitle: string,
  ): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      const updated = await invoke<EntityInstance>('rename_entity', {
        projectPath,
        schemaType,
        oldSlug,
        newTitle,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.currentEntity = updated;
      // Inner call creates its own guard token
      await this.reloadEntities(projectPath, schemaType);
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  async deleteEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('delete_entity', {
        projectPath,
        schemaType,
        slug,
      });
      if (this.guard.isStale(token)) return; // STALE GUARD
      // Inner call creates its own guard token
      await this.reloadEntities(projectPath, schemaType);
      if (this.currentEntity?.slug === slug) {
        this.currentEntity = null;
      }
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isSaving = false;
      }
    }
  }

  /** Update title/tags in the existing entitiesByType array without full reload. */
  private updateEntityInList(schemaType: string, entity: EntityInstance): void {
    const list = this.entitiesByType[schemaType];
    if (!list) return;
    const idx = list.findIndex((e) => e.slug === entity.slug);
    if (idx !== -1) {
      list[idx] = {
        title: entity.title,
        slug: entity.slug,
        schemaType: entity.schemaSlug,
        tags: entity.tags,
      };
    }
  }

  reset(): void {
    this.guard.reset();
    this.schemaSummaries = [];
    this.schemaCache = {};
    this.entitiesByType = {};
    this.currentEntity = null;
    this.isLoadingSchemas = false;
    this.isLoadingEntities = {};
    this.isSaving = false;
    this.schemasLoadedPath = null;
    this.error = null;
  }
}

export const entityStore = new EntityStore();
