import { invoke } from '@tauri-apps/api/core';
import type { NotesConfig, NoteContent, CorkboardPosition, CorkboardSize } from '$lib/types';
import { StaleGuard } from './stale-guard';

/**
 * NotebookStore — mirrors NotesStore but operates on the app-level notebook
 * stored in app_data_dir()/notebook/. NOT a subordinate store — it survives
 * project switches.
 */
class NotebookStore {
  config = $state<NotesConfig>({ notes: [] });
  noteContent = $state<Record<string, NoteContent>>({});
  activeNoteSlug = $state<string | null>(null);
  isLoading = $state(false);
  isLoaded = $state(false);
  error = $state<string | null>(null);

  private guard = new StaleGuard();

  notes = $derived(this.config.notes);
  activeNote = $derived(this.config.notes.find((n) => n.slug === this.activeNoteSlug) ?? null);
  noteCount = $derived(this.config.notes.length);
  hasNotes = $derived(this.config.notes.length > 0);

  async loadConfig(): Promise<void> {
    if (this.isLoaded) return;
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      const config = await invoke<NotesConfig>('get_notebook_config');
      if (this.guard.isStale(token)) return;
      this.config = config;
      this.isLoaded = true;
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  async saveConfig(): Promise<void> {
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_notebook_config', { config: this.config });
      if (this.guard.isStale(token)) return;
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  async createNote(title: string): Promise<void> {
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_notebook_note', { title });
      if (this.guard.isStale(token)) return;
      this.isLoaded = false; // Force reload
      await this.loadConfig();
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  async deleteNote(slug: string): Promise<void> {
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_notebook_note', { slug });
      if (this.guard.isStale(token)) return;
      delete this.noteContent[slug];
      if (this.activeNoteSlug === slug) {
        this.activeNoteSlug = null;
      }
      this.isLoaded = false;
      await this.loadConfig();
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  async loadNoteContent(slug: string): Promise<NoteContent> {
    const cached = this.noteContent[slug];
    if (cached) return cached;

    const token = this.guard.snapshot();
    try {
      const content = await invoke<NoteContent>('get_notebook_note', { slug });
      if (this.guard.isStale(token)) return content;
      this.noteContent[slug] = content;
      return content;
    } catch (e) {
      if (this.guard.isStale(token)) throw e;
      throw e;
    }
  }

  async saveNoteContent(slug: string, title: string, body: string): Promise<void> {
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_notebook_note', { slug, title, body });
      if (this.guard.isStale(token)) return;
      this.noteContent[slug] = { slug, title, body };
      this.config = {
        ...this.config,
        notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, title } : n)),
      };
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  async renameNote(slug: string, newTitle: string): Promise<void> {
    const token = this.guard.snapshot();
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('rename_notebook_note', { slug, newTitle });
      if (this.guard.isStale(token)) return;
      delete this.noteContent[slug];
      if (this.activeNoteSlug === slug) {
        this.activeNoteSlug = null;
      }
      this.isLoaded = false;
      await this.loadConfig();
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.error = String(e);
      throw e;
    } finally {
      if (!this.guard.isStale(token)) {
        this.isLoading = false;
      }
    }
  }

  // Corkboard operations — optimistic local updates

  updateCardPosition(slug: string, position: CorkboardPosition): void {
    this.config = {
      ...this.config,
      notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, position } : n)),
    };
  }

  updateCardColor(slug: string, color: string): void {
    this.config = {
      ...this.config,
      notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, color } : n)),
    };
  }

  updateCardLabel(slug: string, label: string): void {
    this.config = {
      ...this.config,
      notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, label } : n)),
    };
  }

  updateCardSize(slug: string, size: CorkboardSize): void {
    this.config = {
      ...this.config,
      notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, size } : n)),
    };
  }

  selectNote(slug: string): void {
    this.activeNoteSlug = slug;
  }

  /** Copy a notebook note to a project */
  async copyToProject(slug: string, projectPath: string): Promise<NoteContent> {
    const result = await invoke<NoteContent>('copy_notebook_to_project', { slug, projectPath });
    return result;
  }

  /** Move a notebook note to a project (removes from notebook) */
  async moveToProject(slug: string, projectPath: string): Promise<NoteContent> {
    const result = await invoke<NoteContent>('move_notebook_to_project', { slug, projectPath });
    // Refresh notebook config
    delete this.noteContent[slug];
    if (this.activeNoteSlug === slug) {
      this.activeNoteSlug = null;
    }
    this.isLoaded = false;
    await this.loadConfig();
    return result;
  }

  reset(): void {
    this.config = { notes: [] };
    this.noteContent = {};
    this.activeNoteSlug = null;
    this.isLoading = false;
    this.isLoaded = false;
    this.error = null;
    this.guard.reset();
  }
}

export const notebookStore = new NotebookStore();
