import type { NavigationTarget } from '$lib/types';
import { editorState } from './editor.svelte';
import { manuscriptStore } from './manuscript.svelte';
import { notesStore } from './notes.svelte';
import { entityStore } from './entities.svelte';
import { notebookStore } from './notebook.svelte';

/**
 * Stateless coordination layer for all navigation.
 * Centralizes cross-type selection clearing and tab management.
 * All mutable state stays in existing stores.
 */
class NavigationStore {
  /**
   * Navigate to a target: clear cross-type selections, set the right
   * store slug, and open/switch the tab.
   */
  navigateTo(target: NavigationTarget): void {
    this.clearSelections();

    switch (target.type) {
      case 'chapter': {
        const chapter = manuscriptStore.chapters.find((c) => c.slug === target.slug);
        if (!chapter) return;
        manuscriptStore.selectChapter(target.slug);
        editorState.openDocument({
          id: `chapter:${target.slug}`,
          title: chapter.title,
          documentType: 'chapter',
          documentSlug: target.slug,
          isDirty: false,
        });
        break;
      }
      case 'note': {
        const note = notesStore.notes.find((n) => n.slug === target.slug);
        if (!note) return;
        notesStore.selectNote(target.slug);
        editorState.openDocument({
          id: `note:${target.slug}`,
          title: note.title,
          documentType: 'note',
          documentSlug: target.slug,
          isDirty: false,
        });
        break;
      }
      case 'entity': {
        const entities = entityStore.entitiesByType[target.schemaType] ?? [];
        const entity = entities.find((e: { slug: string }) => e.slug === target.slug);
        const title = entity?.title ?? target.slug;
        editorState.openDocument({
          id: `entity:${target.schemaType}:${target.slug}`,
          title,
          documentType: 'entity',
          documentSlug: target.slug,
          isDirty: false,
        });
        break;
      }
      case 'schema': {
        if ('isNew' in target && target.isNew) {
          window.dispatchEvent(new CustomEvent('sakya:new-schema'));
        } else if ('entityType' in target) {
          window.dispatchEvent(new CustomEvent('sakya:edit-schema', { detail: { entityType: target.entityType } }));
        }
        break;
      }
      case 'notebook-note': {
        const note = notebookStore.notes.find((n) => n.slug === target.slug);
        if (!note) return;
        notebookStore.selectNote(target.slug);
        editorState.openDocument({
          id: `notebook-note:${target.slug}`,
          title: `[NB] ${note.title}`,
          documentType: 'notebook-note',
          documentSlug: target.slug,
          isDirty: false,
        });
        break;
      }
      case 'stats': {
        editorState.openDocument({
          id: 'stats',
          title: 'Writing Stats',
          documentType: 'stats',
          documentSlug: '',
          isDirty: false,
        });
        break;
      }
    }
  }

  /**
   * Switch to an existing tab and restore its store selection.
   * Fixes the hidden desync bug where clicking a tab in EditorTabs
   * sets activeTabId but doesn't restore manuscriptStore/notesStore selection.
   */
  switchToTab(tabId: string): void {
    const tab = editorState.tabs.find((t) => t.id === tabId);
    if (!tab) return;

    this.clearSelections();
    editorState.switchTab(tabId);

    // Restore store selection for binder highlighting and inspector
    switch (tab.documentType) {
      case 'chapter':
        manuscriptStore.selectChapter(tab.documentSlug);
        break;
      case 'note':
        notesStore.selectNote(tab.documentSlug);
        break;
      case 'notebook-note':
        notebookStore.selectNote(tab.documentSlug);
        break;
    }
  }

  /** Close the currently active tab and clear its selection. */
  closeActive(): void {
    const tab = editorState.activeTab;
    if (!tab) return;
    this.closeTab(tab.id);
  }

  /** Close a specific tab by ID and clear its selection. */
  closeTab(tabId: string): void {
    const tab = editorState.tabs.find((t) => t.id === tabId);
    if (!tab) return;

    editorState.closeTab(tabId);
    this.clearSelectionForTab(tab.documentType, tab.documentSlug);

    // Restore selection for the newly active tab
    const newActive = editorState.activeTab;
    if (newActive) {
      switch (newActive.documentType) {
        case 'chapter':
          manuscriptStore.selectChapter(newActive.documentSlug);
          break;
        case 'note':
          notesStore.selectNote(newActive.documentSlug);
          break;
        case 'notebook-note':
          notebookStore.selectNote(newActive.documentSlug);
          break;
      }
    }
  }

  /** Clear ALL cross-type selections. */
  private clearSelections(): void {
    manuscriptStore.selectChapter('');
    notesStore.selectNote('');
    notebookStore.selectNote('');
    entityStore.currentEntity = null;
  }

  /** Clear selection for a specific document type. */
  private clearSelectionForTab(documentType: string, _documentSlug: string): void {
    switch (documentType) {
      case 'chapter':
        manuscriptStore.selectChapter('');
        break;
      case 'note':
        notesStore.selectNote('');
        break;
      case 'notebook-note':
        notebookStore.selectNote('');
        break;
      case 'entity':
        entityStore.currentEntity = null;
        break;
    }
  }
}

export const navigationStore = new NavigationStore();
