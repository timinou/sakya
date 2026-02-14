import type { Theme, ViewMode, PaneConfig } from '$lib/types';

class UIState {
  theme = $state<Theme>('system');
  viewMode = $state<ViewMode>('editor');
  panes = $state<PaneConfig>({
    binderWidth: 260,
    inspectorWidth: 300,
    binderVisible: true,
    inspectorVisible: true,
  });

  effectiveTheme = $derived<'light' | 'dark'>(
    this.theme === 'system'
      ? (typeof window !== 'undefined' &&
          window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light')
      : this.theme
  );

  toggleBinder(): void {
    this.panes.binderVisible = !this.panes.binderVisible;
  }

  toggleInspector(): void {
    this.panes.inspectorVisible = !this.panes.inspectorVisible;
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
  }

  setViewMode(mode: ViewMode): void {
    this.viewMode = mode;
  }

  setBinderWidth(width: number): void {
    this.panes.binderWidth = Math.max(200, Math.min(400, width));
  }

  setInspectorWidth(width: number): void {
    this.panes.inspectorWidth = Math.max(200, Math.min(500, width));
  }
}

export const uiState = new UIState();
