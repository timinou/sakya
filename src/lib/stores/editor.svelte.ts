import type { EditorTab, WordCount } from '$lib/types';

class EditorState {
  tabs = $state<EditorTab[]>([]);
  activeTabId = $state<string | null>(null);
  wordCount = $state<WordCount>({ words: 0, characters: 0, charactersNoSpaces: 0 });
  isSaving = $state(false);

  activeTab = $derived(this.tabs.find((t) => t.id === this.activeTabId) ?? null);
  hasDirtyTabs = $derived(this.tabs.some((t) => t.isDirty));

  openDocument(tab: EditorTab): void {
    const existing = this.tabs.find((t) => t.id === tab.id);
    if (existing) {
      this.activeTabId = existing.id;
      return;
    }
    this.tabs.push(tab);
    this.activeTabId = tab.id;
  }

  switchTab(tabId: string): void {
    const tab = this.tabs.find((t) => t.id === tabId);
    if (tab) {
      this.activeTabId = tab.id;
    }
  }

  closeTab(tabId: string): void {
    this.tabs = this.tabs.filter((t) => t.id !== tabId);
    if (this.activeTabId === tabId) {
      this.activeTabId = this.tabs.length > 0 ? this.tabs[this.tabs.length - 1].id : null;
    }
  }

  setDirty(tabId: string, dirty: boolean): void {
    const tab = this.tabs.find((t) => t.id === tabId);
    if (tab) tab.isDirty = dirty;
  }

  updateWordCount(count: WordCount): void {
    this.wordCount = count;
  }

  closeAll(): void {
    this.tabs = [];
    this.activeTabId = null;
  }

  reset(): void {
    this.tabs = [];
    this.activeTabId = null;
    this.wordCount = { words: 0, characters: 0, charactersNoSpaces: 0 };
    this.isSaving = false;
  }
}

export const editorState = new EditorState();
