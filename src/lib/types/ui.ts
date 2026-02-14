export type Theme = 'light' | 'dark' | 'system';
export type ViewMode = 'editor' | 'corkboard' | 'split';

export interface PaneConfig {
  binderWidth: number;
  inspectorWidth: number;
  binderVisible: boolean;
  inspectorVisible: boolean;
}

export interface UIState {
  theme: Theme;
  viewMode: ViewMode;
  panes: PaneConfig;
  openTabs: string[];
  activeTab: string | null;
}
