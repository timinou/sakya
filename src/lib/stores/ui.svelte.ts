import { writeTextFile, readTextFile, mkdir } from '@tauri-apps/plugin-fs';
import type { Theme, ViewMode, PaneConfig } from '$lib/types';
import { StaleGuard } from './stale-guard';

class UIState {
  theme = $state<Theme>('system');
  viewMode = $state<ViewMode>('editor');
  typewriterMode = $state(false);
  distractionFreeMode = $state(false);
  focusMode = $state(false);
  panes = $state<PaneConfig>({
    binderWidth: 260,
    inspectorWidth: 300,
    binderVisible: true,
    inspectorVisible: true,
  });

  private guard = new StaleGuard();

  effectiveTheme = $derived<'light' | 'dark'>(
    this.theme === 'system'
      ? (typeof window !== 'undefined' &&
          window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light')
      : this.theme
  );

  toggleTypewriterMode(): void {
    this.typewriterMode = !this.typewriterMode;
  }

  toggleDistractionFreeMode(): void {
    this.distractionFreeMode = !this.distractionFreeMode;
  }

  toggleFocusMode(): void {
    this.focusMode = !this.focusMode;
  }

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
    const token = this.guard.begin(); // STALE GUARD
    const state = {
      theme: this.theme,
      viewMode: this.viewMode,
      panes: { ...this.panes },
      typewriterMode: this.typewriterMode,
      distractionFreeMode: this.distractionFreeMode,
      focusMode: this.focusMode,
    };
    const dir = `${projectPath}/.sakya`;
    try {
      await mkdir(dir, { recursive: true });
    } catch {
      /* directory may already exist */
    }
    if (this.guard.isStale(token)) return; // STALE GUARD
    await writeTextFile(`${dir}/ui-state.json`, JSON.stringify(state, null, 2));
  }

  async restore(projectPath: string): Promise<void> {
    const token = this.guard.begin(); // STALE GUARD
    try {
      const content = await readTextFile(`${projectPath}/.sakya/ui-state.json`);
      const state = JSON.parse(content);
      if (this.guard.isStale(token)) return; // STALE GUARD
      if (state.theme) this.theme = state.theme;
      if (state.viewMode) this.viewMode = state.viewMode;
      if (state.typewriterMode !== undefined) this.typewriterMode = state.typewriterMode;
      if (state.distractionFreeMode !== undefined) this.distractionFreeMode = state.distractionFreeMode;
      if (state.focusMode !== undefined) this.focusMode = state.focusMode;
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

  reset(): void {
    this.guard.reset();
    this.theme = 'system';
    this.viewMode = 'editor';
    this.typewriterMode = false;
    this.distractionFreeMode = false;
    this.focusMode = false;
    this.panes = {
      binderWidth: 260,
      inspectorWidth: 300,
      binderVisible: true,
      inspectorVisible: true,
    };
  }
}

export const uiState = new UIState();
