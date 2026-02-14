export interface EditorTab {
  id: string;
  title: string;
  documentType: 'chapter' | 'entity' | 'note' | 'schema' | 'stats';
  documentSlug: string;
  isDirty: boolean;
}

export interface DocumentContent {
  frontmatter: Record<string, unknown>;
  body: string;
}

export interface WordCount {
  words: number;
  characters: number;
  charactersNoSpaces: number;
}
