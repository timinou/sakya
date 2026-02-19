export interface CorkboardPosition {
  x: number;
  y: number;
}

export interface CorkboardSize {
  width: number;
  height: number;
}

export interface NoteEntry {
  slug: string;
  title: string;
  color?: string;
  label?: string;
  position?: CorkboardPosition;
  size?: CorkboardSize;
}

export interface NotesConfig {
  notes: NoteEntry[];
}

export interface NoteContent {
  slug: string;
  title: string;
  body: string;
}
