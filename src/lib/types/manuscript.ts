export type ChapterStatus = 'draft' | 'revised' | 'final';

export interface ManuscriptConfig {
  chapters: string[]; // ordered list of chapter slugs
}

export interface Chapter {
  slug: string;
  title: string;
  pov?: string;
  status: ChapterStatus;
  synopsis?: string;
  targetWords?: number;
  order: number;
}

export interface ChapterContent {
  slug: string;
  frontmatter: Chapter;
  body: string;
}
