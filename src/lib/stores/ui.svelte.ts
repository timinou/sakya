import { writeTextFile, readTextFile, mkdir } from '@tauri-apps/plugin-fs';
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

  async persist(projectPath: string): Promise<void> {
    const state = {
      theme: this.theme,
      viewMode: this.viewMode,
      panes: { ...this.panes },
    };
    const dir = `${projectPath}/.sakya`;
    try {
      await mkdir(dir, { recursive: true });
    } catch {
      /* directory may already exist */
    }
    await writeTextFile(`${dir}/ui-state.json`, JSON.stringify(state, null, 2));
  }

  async restore(projectPath: string): Promise<void> {
    try {
      const content = await readTextFile(`${projectPath}/.sakya/ui-state.json`);
      const state = JSON.parse(content);
      if (state.theme) this.theme = state.theme;
      if (state.viewMode) this.viewMode = state.viewMode;
      if (state.panes) {
        if (state.panes.binderWidth) this.panes.binderWidth = state.panes.binderWidth;
        if (state.panes.inspectorWidth) this.panes.inspectorWidth = state.panes.inspectorWidth;
        if (state.panes.binderVisible !== undefined) this.panes.binderVisible = state.panes.binderVisible;
        if (state.panes.inspectorVisible !== undefined) this.panes.inspectorVisible = state.panes.inspectorVisible;
      }
    } catch {
      // File missing or corrupt — use defaults (do nothing)
    }
  }
}

export const uiState = new UIState();
