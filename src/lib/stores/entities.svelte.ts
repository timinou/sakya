import { invoke } from '@tauri-apps/api/core';
import type { EntitySchema, SchemaSummary, EntityInstance, EntitySummary } from '$lib/types';

class EntityStore {
  schemaSummaries = $state<SchemaSummary[]>([]);
  schemaCache = $state<Record<string, EntitySchema>>({});
  entitiesByType = $state<Record<string, EntitySummary[]>>({});
  currentEntity = $state<EntityInstance | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);

  schemasLoaded = $derived(this.schemaSummaries.length > 0);

  async loadSchemas(projectPath: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      this.schemaSummaries = await invoke<SchemaSummary[]>('list_schemas', { projectPath });
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
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
    this.isLoading = true;
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
      this.isLoading = false;
    }
  }

  async deleteSchema(projectPath: string, schemaType: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_schema', { projectPath, schemaType });
      delete this.schemaCache[schemaType];
      await this.loadSchemas(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  // Entity instance methods (ITEM-013 - commands not yet implemented)
  async loadEntities(projectPath: string, schemaType: string): Promise<void> {
    if (this.entitiesByType[schemaType]) return;

    this.isLoading = true;
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
      this.isLoading = false;
    }
  }

  async getEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<EntityInstance> {
    this.isLoading = true;
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
      this.isLoading = false;
    }
  }

  async createEntity(
    projectPath: string,
    schemaType: string,
    title: string,
  ): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_entity', {
        projectPath,
        schemaType,
        title,
      });
      this.invalidateType(schemaType);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async saveEntity(projectPath: string, entity: EntityInstance): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_entity', {
        projectPath,
        entity,
      });
      this.currentEntity = entity;
      this.invalidateType(entity.schemaSlug);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async renameEntity(
    projectPath: string,
    schemaType: string,
    oldSlug: string,
    newTitle: string,
  ): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      const updated = await invoke<EntityInstance>('rename_entity', {
        projectPath,
        schemaType,
        oldSlug,
        newTitle,
      });
      this.currentEntity = updated;
      this.invalidateType(schemaType);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async deleteEntity(
    projectPath: string,
    schemaType: string,
    slug: string,
  ): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_entity', {
        projectPath,
        schemaType,
        slug,
      });
      this.invalidateType(schemaType);
      if (this.currentEntity?.slug === slug) {
        this.currentEntity = null;
      }
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  invalidateType(schemaType: string): void {
    delete this.entitiesByType[schemaType];
  }

  reset(): void {
    this.schemaSummaries = [];
    this.schemaCache = {};
    this.entitiesByType = {};
    this.currentEntity = null;
    this.isLoading = false;
    this.error = null;
  }
}

export const entityStore = new EntityStore();
