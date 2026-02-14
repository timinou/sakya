export interface WritingSession {
  id: string;
  start: string;           // ISO 8601
  end?: string;             // ISO 8601
  durationMinutes?: number;
  wordsWritten: number;
  chapterSlug: string;
  sprintGoal?: number;
}

export interface SessionStats {
  totalSessions: number;
  totalWords: number;
  totalMinutes: number;
  currentStreak: number;
  longestStreak: number;
  dailyAverage: number;
  weeklyAverage: number;
  monthlyAverage: number;
  bestDayWords: number;
  bestDayDate?: string;
}
