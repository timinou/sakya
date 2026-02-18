import { invoke } from '@tauri-apps/api/core';
import type { ManuscriptConfig, Chapter, ChapterContent } from '$lib/types';
import { StaleGuard } from './stale-guard';

class ManuscriptStore {
  config = $state<ManuscriptConfig>({ chapters: [] });
  chapters = $state<Chapter[]>([]);
  activeChapterSlug = $state<string | null>(null);
  chapterContent = $state<Record<string, ChapterContent>>({});
  isLoading = $state(false);
  error = $state<string | null>(null);

  private guard = new StaleGuard();

  activeChapter = $derived(this.chapters.find((c) => c.slug === this.activeChapterSlug) ?? null);
  hasChapters = $derived(this.chapters.length > 0);
  chapterCount = $derived(this.chapters.length);

  async loadConfig(projectPath: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      const config = await invoke<ManuscriptConfig>('get_manuscript_config', { projectPath });
      if (this.guard.isStale(token)) return; // STALE GUARD

      this.config = config;

      const chapterResults = await Promise.all(
        this.config.chapters.map((slug) =>
          invoke<ChapterContent>('get_chapter', { projectPath, slug }),
        ),
      );
      if (this.guard.isStale(token)) return; // STALE GUARD

      this.chapters = chapterResults.map((content, index) => ({
        ...content.frontmatter,
        order: index,
      }));
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

  async createChapter(projectPath: string, title: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_chapter', { projectPath, title });
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

  async deleteChapter(projectPath: string, slug: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_chapter', { projectPath, slug });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.chapterContent[slug];
      if (this.activeChapterSlug === slug) {
        this.activeChapterSlug = null;
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

  async reorderChapters(projectPath: string, chapterSlugs: string[]): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('reorder_chapters', { projectPath, chapterSlugs });
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

  async loadChapterContent(projectPath: string, slug: string): Promise<ChapterContent> {
    const cached = this.chapterContent[slug];
    if (cached) return cached;

    const token = this.guard.snapshot(); // STALE GUARD
    try {
      const content = await invoke<ChapterContent>('get_chapter', { projectPath, slug });
      if (this.guard.isStale(token)) return content; // STALE GUARD — return value but don't cache
      this.chapterContent[slug] = content;
      return content;
    } catch (e) {
      if (this.guard.isStale(token)) throw e; // STALE GUARD
      throw e;
    }
  }

  async saveChapterContent(
    projectPath: string,
    slug: string,
    chapter: Chapter,
    body: string,
  ): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_chapter', { projectPath, slug, chapter, body });
      if (this.guard.isStale(token)) return; // STALE GUARD
      this.chapterContent[slug] = { slug, frontmatter: chapter, body };
      this.chapters = this.chapters.map((c) => (c.slug === slug ? { ...chapter } : c));
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

  async renameChapter(projectPath: string, slug: string, newTitle: string): Promise<void> {
    const token = this.guard.snapshot(); // STALE GUARD
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('rename_chapter', { projectPath, slug, newTitle });
      if (this.guard.isStale(token)) return; // STALE GUARD
      delete this.chapterContent[slug];
      if (this.activeChapterSlug === slug) {
        this.activeChapterSlug = null;
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

  async updateChapterMetadata(
    projectPath: string,
    slug: string,
    updates: Partial<Chapter>,
  ): Promise<void> {
    const chapter = this.chapters.find((c) => c.slug === slug);
    if (!chapter) return;
    const updated = { ...chapter, ...updates };
    const body = this.chapterContent[slug]?.body ?? '';
    await this.saveChapterContent(projectPath, slug, updated, body);
  }

  selectChapter(slug: string): void {
    this.activeChapterSlug = slug;
  }

  reset(): void {
    this.config = { chapters: [] };
    this.chapters = [];
    this.activeChapterSlug = null;
    this.chapterContent = {};
    this.isLoading = false;
    this.error = null;
    this.guard.reset();
  }
}

export const manuscriptStore = new ManuscriptStore();
