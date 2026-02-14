import { invoke } from '@tauri-apps/api/core';
import type { ManuscriptConfig, Chapter, ChapterContent } from '$lib/types';

class ManuscriptStore {
  config = $state<ManuscriptConfig>({ chapters: [] });
  chapters = $state<Chapter[]>([]);
  activeChapterSlug = $state<string | null>(null);
  chapterContent = $state<Record<string, ChapterContent>>({});
  isLoading = $state(false);
  error = $state<string | null>(null);

  activeChapter = $derived(this.chapters.find((c) => c.slug === this.activeChapterSlug) ?? null);
  hasChapters = $derived(this.chapters.length > 0);
  chapterCount = $derived(this.chapters.length);

  async loadConfig(projectPath: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      this.config = await invoke<ManuscriptConfig>('get_manuscript_config', { projectPath });

      const chapterResults = await Promise.all(
        this.config.chapters.map((slug) =>
          invoke<ChapterContent>('get_chapter', { projectPath, slug }),
        ),
      );

      this.chapters = chapterResults.map((content, index) => ({
        ...content.frontmatter,
        order: index,
      }));
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async createChapter(projectPath: string, title: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('create_chapter', { projectPath, title });
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async deleteChapter(projectPath: string, slug: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('delete_chapter', { projectPath, slug });
      delete this.chapterContent[slug];
      if (this.activeChapterSlug === slug) {
        this.activeChapterSlug = null;
      }
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async reorderChapters(projectPath: string, chapterSlugs: string[]): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('reorder_chapters', { projectPath, chapterSlugs });
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async loadChapterContent(projectPath: string, slug: string): Promise<ChapterContent> {
    const cached = this.chapterContent[slug];
    if (cached) return cached;

    const content = await invoke<ChapterContent>('get_chapter', { projectPath, slug });
    this.chapterContent[slug] = content;
    return content;
  }

  async saveChapterContent(
    projectPath: string,
    slug: string,
    chapter: Chapter,
    body: string,
  ): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('save_chapter', { projectPath, slug, chapter, body });
      this.chapterContent[slug] = { slug, frontmatter: chapter, body };
      this.chapters = this.chapters.map((c) => (c.slug === slug ? { ...chapter } : c));
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async renameChapter(projectPath: string, slug: string, newTitle: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      await invoke('rename_chapter', { projectPath, slug, newTitle });
      delete this.chapterContent[slug];
      if (this.activeChapterSlug === slug) {
        this.activeChapterSlug = null;
      }
      await this.loadConfig(projectPath);
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.isLoading = false;
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
  }
}

export const manuscriptStore = new ManuscriptStore();
