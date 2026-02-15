import { invoke } from '@tauri-apps/api/core';
import type { ProjectManifest, RecentProject } from '$lib/types';

class ProjectState {
  manifest = $state<ProjectManifest | null>(null);
  projectPath = $state<string | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);
  recentProjects = $state<RecentProject[]>([]);

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
    this.isLoading = true;
    this.error = null;
    try {
      const manifest = await invoke<ProjectManifest>('open_project', { path });
      this.manifest = manifest;
      this.projectPath = path;
      // Add to recent projects (fire and forget)
      invoke<RecentProject[]>('add_recent_project', { name: manifest.name, path })
        .then((list) => { this.recentProjects = list; })
        .catch(() => {});
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async create(name: string, path: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      const manifest = await invoke<ProjectManifest>('create_project', { name, path });
      this.manifest = manifest;
      this.projectPath = path;
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
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
    this.manifest = null;
    this.projectPath = null;
    this.error = null;
  }
}

export const projectState = new ProjectState();
