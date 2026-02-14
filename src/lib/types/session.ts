export interface WritingSession {
  id: string;
  start: string;           // ISO 8601
  end?: string;             // ISO 8601
  duration_minutes?: number;
  words_written: number;
  chapter_slug: string;
  sprint_goal?: number;
}
