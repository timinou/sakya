/** Matches Rust ChapterHeaderStyle enum (serde snake_case) */
export type ChapterHeaderStyle = 'numbered' | 'titled' | 'numbered_and_titled' | 'none';

/** Matches Rust ChapterSeparator enum (serde snake_case) */
export type ChapterSeparator = 'page_break' | 'three_stars' | 'horizontal_rule' | 'blank_lines';

/** Matches Rust OutputFormat enum (serde snake_case) */
export type OutputFormat = 'markdown' | 'html' | 'plain_text';

/** Matches Rust CompileConfig struct (serde camelCase fields, snake_case enum values) */
export interface CompileConfig {
  title: string;
  author: string;
  includeTitlePage: boolean;
  chapterHeaderStyle: ChapterHeaderStyle;
  chapterSeparator: ChapterSeparator;
  outputFormat: OutputFormat;
  includeSynopsis: boolean;
  frontMatter: string;
}

/** Matches Rust CompileOutput struct (serde camelCase fields) */
export interface CompileOutput {
  content: string;
  format: OutputFormat;
  chapterCount: number;
  wordCount: number;
}

/** Default compile config matching Rust Default impl */
export function defaultCompileConfig(): CompileConfig {
  return {
    title: '',
    author: '',
    includeTitlePage: true,
    chapterHeaderStyle: 'numbered_and_titled',
    chapterSeparator: 'page_break',
    outputFormat: 'markdown',
    includeSynopsis: false,
    frontMatter: '',
  };
}
