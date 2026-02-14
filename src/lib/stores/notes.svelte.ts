import { invoke } from '@tauri-apps/api/core';
import type { NotesConfig, NoteContent, CorkboardPosition } from '$lib/types';

class NotesStore {
  config = $state<NotesConfig>({ notes: [] });
  noteContent = $state<Record<string, NoteContent>>({});
  activeNoteSlug = $state<string | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);

  notes = $derived(this.config.notes);
  activeNote = $derived(this.config.notes.find((n) => n.slug === this.activeNoteSlug) ?? null);
  noteCount = $derived(this.config.notes.length);
  hasNotes = $derived(this.config.notes.length > 0);

  async loadConfig(projectPath: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      this.config = await invoke<NotesConfig>('get_notes_config', { projectPath });
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async saveConfig(projectPath: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_notes_config', { projectPath, config: this.config });
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async createNote(projectPath: string, title: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_note', { projectPath, title });
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async deleteNote(projectPath: string, slug: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_note', { projectPath, slug });
      delete this.noteContent[slug];
      if (this.activeNoteSlug === slug) {
        this.activeNoteSlug = null;
      }
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async loadNoteContent(projectPath: string, slug: string): Promise<NoteContent> {
    const cached = this.noteContent[slug];
    if (cached) return cached;

    const content = await invoke<NoteContent>('get_note', { projectPath, slug });
    this.noteContent[slug] = content;
    return content;
  }

  async saveNoteContent(
    projectPath: string,
    slug: string,
    title: string,
    body: string,
  ): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_note', { projectPath, slug, title, body });
      this.noteContent[slug] = { slug, title, body };
      // Update the title in the config entry if it changed
      this.config = {
        ...this.config,
        notes: this.config.notes.map((n) => (n.slug === slug ? { ...n, title } : n)),
      };
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
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

  selectNote(slug: string): void {
    this.activeNoteSlug = slug;
  }

  reset(): void {
    this.config = { notes: [] };
    this.noteContent = {};
    this.activeNoteSlug = null;
    this.isLoading = false;
    this.error = null;
  }
}

export const notesStore = new NotesStore();
