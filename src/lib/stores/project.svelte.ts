import { invoke } from '@tauri-apps/api/core';
import type { ProjectManifest, RecentProject } from '$lib/types';
import { StaleGuard } from './stale-guard';
import { manuscriptStore } from './manuscript.svelte';
import { notesStore } from './notes.svelte';
import { entityStore } from './entities.svelte';
import { sessionsStore } from './sessions.svelte';
import { uiState } from './ui.svelte';
import { editorState } from './editor.svelte';

class ProjectState {
  manifest = $state<ProjectManifest | null>(null);
  projectPath = $state<string | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);
  recentProjects = $state<RecentProject[]>([]);
  private guard = new StaleGuard();

  isOpen = $derived(this.manifest !== null);

  async loadRecent(): Promise<void> {
    try {
      this.recentProjects = await invoke<RecentProject[]>('list_recent_projects');
    } catch {
      // Silently fail - recent projects is non-critical
      this.recentProjects = [];
    }
  }

  async removeRecent(path: string): Promise<void> {
    try {
      this.recentProjects = await invoke<RecentProject[]>('remove_recent_project', { path });
    } catch {
      // Silently fail
    }
  }

  async open(path: string): Promise<void> {
    const token = this.guard.begin(); // STALE GUARD
    // Clear projectPath FIRST so effects don't fire for the old project during await.
    // Without this, effects see the old path + reset stores and re-load stale data.
    this.projectPath = null;
    // Reset all subordinate stores to prevent stale data from previous project
    this.resetSubordinateStores();
    this.isLoading = true;
    this.error = null;
    try {
      const manifest = await invoke<ProjectManifest>('open_project', { path });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.manifest = manifest;
      this.projectPath = path;
      // Add to recent projects (fire and forget)
      invoke<RecentProject[]>('add_recent_project', { name: manifest.name, path })
        .then((list) => {
          if (!this.guard.isStale(token)) { // STALE GUARD
            this.recentProjects = list;
          }
        })
        .catch(() => {});
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoading = false;
      }
    }
  }

  async create(name: string, path: string): Promise<void> {
    const token = this.guard.begin(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      const manifest = await invoke<ProjectManifest>('create_project', { name, path });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.manifest = manifest;
      this.projectPath = path;
    } catch (e) {
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) { // STALE GUARD
        this.isLoading = false;
      }
    }
  }

  async save(): Promise<void> {
    if (!this.manifest || !this.projectPath) return;
    await invoke('save_project_manifest', {
      path: this.projectPath,
      manifest: this.manifest,
    });
  }

  close(): void {
    this.guard.reset();
    // Reset all subordinate stores to prevent stale data from previous project
    this.resetSubordinateStores();
    this.manifest = null;
    this.projectPath = null;
    this.error = null;
  }

  private resetSubordinateStores(): void {
    manuscriptStore.reset();
    notesStore.reset();
    entityStore.reset();
    sessionsStore.reset();
    uiState.reset();
    editorState.reset();
  }
}

export const projectState = new ProjectState();
