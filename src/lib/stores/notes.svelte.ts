import { invoke } from '@tauri-apps/api/core';
import type { NotesConfig, NoteContent, CorkboardPosition, CorkboardSize } from '$lib/types';
import { StaleGuard } from './stale-guard';

class NotesStore {
  config = $state<NotesConfig>({ notes: [] });
  noteContent = $state<Record<string, NoteContent>>({});
  activeNoteSlug = $state<string | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);

  private guard = new StaleGuard();

  notes = $derived(this.config.notes);
  activeNote = $derived(this.config.notes.find((n) => n.slug === this.activeNoteSlug) ?? null);
  noteCount = $derived(this.config.notes.length);
  hasNotes = $derived(this.config.notes.length > 0);

  async loadConfig(projectPath: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      const config = await invoke<NotesConfig>('get_notes_config', { projectPath });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.config = config;
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

  async saveConfig(projectPath: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_notes_config', { projectPath, config: this.config });
      if (this.guard.isStale(token)) return; // STALE GUARD
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

  async createNote(projectPath: string, title: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_note', { projectPath, title });
      if (this.guard.isStale(token)) return; // STALE GUARD
      await this.loadConfig(projectPath);
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

  async deleteNote(projectPath: string, slug: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_note', { projectPath, slug });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.noteContent[slug];
      if (this.activeNoteSlug === slug) {
        this.activeNoteSlug = null;
      }
      await this.loadConfig(projectPath);
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

  async loadNoteContent(projectPath: string, slug: string): Promise<NoteContent> {
    const cached = this.noteContent[slug];
    if (cached) return cached;

    const token = this.guard.snapshot(); // STALE GUARD
    try {
      const content = await invoke<NoteContent>('get_note', { projectPath, slug });
      if (this.guard.isStale(token)) return content; // STALE GUARD — return value but don't cache
      this.noteContent[slug] = content;
      return content;
    } catch (e) {
      if (this.guard.isStale(token)) throw e; // STALE GUARD
      throw e;
    }
  }

  async saveNoteContent(
    projectPath: string,
    slug: string,
    title: string,
    body: string,
  ): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_note', { projectPath, slug, title, body });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.noteContent[slug] = { slug, title, body };
      // Update the title in the config entry if it changed
      this.config = {
        ...this.config,
        notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, title } : n)),
      };
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

  // Corkboard operations — optimistic local updates.
  // The caller is responsible for debouncing saves to the backend.

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

  async renameNote(projectPath: string, slug: string, newTitle: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('rename_note', { projectPath, slug, newTitle });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.noteContent[slug];
      if (this.activeNoteSlug === slug) {
        this.activeNoteSlug = null;
      }
      await this.loadConfig(projectPath);
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

  selectNote(slug: string): void {
    this.activeNoteSlug = slug;
  }

  reset(): void {
    this.config = { notes: [] };
    this.noteContent = {};
    this.activeNoteSlug = null;
    this.isLoading = false;
    this.error = null;
    this.guard.reset();
  }
}

export const notesStore = new NotesStore();
