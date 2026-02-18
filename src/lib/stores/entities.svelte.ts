import { invoke } from '@tauri-apps/api/core';
import type { EntitySchema, SchemaSummary, EntityInstance, EntitySummary } from '$lib/types';

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

  // --- Path-based loaded tracking (Bug 1 fix) ---
  schemasLoadedPath = $state<string | null>(null);
  schemasLoaded = $derived(this.schemasLoadedPath !== null);

  async loadSchemas(projectPath: string): Promise<void> {
    this.isLoadingSchemas = true;
    this.error = null;
    try {
      this.schemaSummaries = await invoke<SchemaSummary[]>('list_schemas', { projectPath });
      this.schemasLoadedPath = projectPath;
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoadingSchemas = false;
    }
  }

  async getSchema(projectPath: string, schemaType: string): Promise<EntitySchema> {
    // Return cached if available
    const cached = this.schemaCache[schemaType];
    if (cached) return cached;

    const schema = await invoke<EntitySchema>('get_schema', {
      projectPath,
      schemaType,
    });
    this.schemaCache[schemaType] = schema;
    return schema;
  }

  async saveSchema(projectPath: string, schema: EntitySchema): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('save_schema', { projectPath, schema });
      delete this.schemaCache[schema.entityType];
      // Reload summaries
      await this.loadSchemas(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  async deleteSchema(projectPath: string, schemaType: string): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('delete_schema', { projectPath, schemaType });
      delete this.schemaCache[schemaType];
      await this.loadSchemas(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  // --- Per-type loading guard (Bug 2 fix) ---
  async loadEntities(projectPath: string, schemaType: string): Promise<void> {
    // Skip if already loading this type
    if (this.isLoadingEntities[schemaType]) return;
    // Skip if already loaded
    if (this.entitiesByType[schemaType]) return;

    this.isLoadingEntities[schemaType] = true;
    this.error = null;
    try {
      const entities = await invoke<EntitySummary[]>('list_entities', {
        projectPath,
        schemaType,
      });
      this.entitiesByType[schemaType] = entities;
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoadingEntities[schemaType] = false;
    }
  }

  // Force-refetch entities for a type (used after create/delete/rename)
  async reloadEntities(projectPath: string, schemaType: string): Promise<void> {
    this.isLoadingEntities[schemaType] = true;
    this.error = null;
    try {
      const entities = await invoke<EntitySummary[]>('list_entities', {
        projectPath,
        schemaType,
      });
      this.entitiesByType[schemaType] = entities;
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoadingEntities[schemaType] = false;
    }
  }

  async getEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<EntityInstance> {
    this.isSaving = true;
    this.error = null;
    try {
      const entity = await invoke<EntityInstance>('get_entity', {
        projectPath,
        schemaType,
        slug,
      });
      this.currentEntity = entity;
      return entity;
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  async createEntity(
    projectPath: string,
    schemaType: string,
    title: string,
  ): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('create_entity', {
        projectPath,
        schemaType,
        title,
      });
      await this.reloadEntities(projectPath, schemaType);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  // --- In-place update for saves (Bug 3 fix) ---
  async saveEntity(projectPath: string, entity: EntityInstance): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('save_entity', {
        projectPath,
        entity,
      });
      this.currentEntity = entity;
      // Update in-place instead of invalidating the whole type
      this.updateEntityInList(entity.schemaSlug, entity);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  async renameEntity(
    projectPath: string,
    schemaType: string,
    oldSlug: string,
    newTitle: string,
  ): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      const updated = await invoke<EntityInstance>('rename_entity', {
        projectPath,
        schemaType,
        oldSlug,
        newTitle,
      });
      this.currentEntity = updated;
      await this.reloadEntities(projectPath, schemaType);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  async deleteEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<void> {
    this.isSaving = true;
    this.error = null;
    try {
      await invoke('delete_entity', {
        projectPath,
        schemaType,
        slug,
      });
      await this.reloadEntities(projectPath, schemaType);
      if (this.currentEntity?.slug === slug) {
        this.currentEntity = null;
      }
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isSaving = false;
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
