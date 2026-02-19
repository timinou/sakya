import { describe, it, expect, beforeEach, vi } from 'vitest';
import { navigationStore } from './navigation.svelte';
import { editorState } from './editor.svelte';
import { manuscriptStore } from './manuscript.svelte';
import { notesStore } from './notes.svelte';
import { entityStore } from './entities.svelte';
import type { EditorTab, NavigationTarget } from '$lib/types';

// Helper to set up store state without triggering real loads
function seedChapters(...slugs: string[]) {
  (manuscriptStore as any).chapters = slugs.map((slug, i) => ({
    slug,
    title: `Chapter ${slug}`,
    order: i,
    status: 'draft' as const,
  }));
}

function seedNotes(...slugs: string[]) {
  (notesStore as any).config = {
    notes: slugs.map((slug) => ({
      slug,
      title: `Note ${slug}`,
    })),
  };
}

function seedEntities(schemaType: string, ...slugs: string[]) {
  entityStore.entitiesByType[schemaType] = slugs.map((slug) => ({
    slug,
    title: `Entity ${slug}`,
    schemaType,
    tags: [],
  }));
}

describe('NavigationStore', () => {
  beforeEach(() => {
    editorState.reset();
    manuscriptStore.reset();
    notesStore.reset();
    entityStore.reset();
  });

  describe('navigateTo — chapter', () => {
    it('clears note selection and sets chapter selection', () => {
      seedChapters('ch-1');
      notesStore.selectNote('some-note');

      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      expect(manuscriptStore.activeChapterSlug).toBe('ch-1');
      expect(notesStore.activeNoteSlug).toBe('');
    });

    it('opens a tab for the chapter', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].id).toBe('chapter:ch-1');
      expect(editorState.tabs[0].documentType).toBe('chapter');
      expect(editorState.activeTabId).toBe('chapter:ch-1');
    });

    it('uses chapter title for the tab', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      expect(editorState.tabs[0].title).toBe('Chapter ch-1');
    });

    it('no-ops if slug not found', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'nonexistent' });

      expect(editorState.tabs).toHaveLength(0);
      expect(manuscriptStore.activeChapterSlug).toBe('');
    });

    it('switches to existing tab instead of duplicating', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      expect(editorState.tabs).toHaveLength(1);
    });
  });

  describe('navigateTo — note', () => {
    it('clears chapter selection and sets note selection', () => {
      seedNotes('note-1');
      manuscriptStore.selectChapter('some-chapter');

      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });

      expect(notesStore.activeNoteSlug).toBe('note-1');
      expect(manuscriptStore.activeChapterSlug).toBe('');
    });

    it('opens a tab for the note', () => {
      seedNotes('note-1');
      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].id).toBe('note:note-1');
      expect(editorState.tabs[0].documentType).toBe('note');
    });

    it('no-ops if slug not found', () => {
      seedNotes('note-1');
      navigationStore.navigateTo({ type: 'note', slug: 'missing' });

      expect(editorState.tabs).toHaveLength(0);
    });
  });

  describe('navigateTo — entity', () => {
    it('clears chapter and note selection', () => {
      seedEntities('character', 'hero');
      manuscriptStore.selectChapter('ch-1');
      notesStore.selectNote('note-1');

      navigationStore.navigateTo({ type: 'entity', schemaType: 'character', slug: 'hero' });

      expect(manuscriptStore.activeChapterSlug).toBe('');
      expect(notesStore.activeNoteSlug).toBe('');
    });

    it('opens a tab with entity title', () => {
      seedEntities('character', 'hero');
      navigationStore.navigateTo({ type: 'entity', schemaType: 'character', slug: 'hero' });

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].id).toBe('entity:character:hero');
      expect(editorState.tabs[0].title).toBe('Entity hero');
    });

    it('uses slug as fallback title if entity not found in list', () => {
      navigationStore.navigateTo({ type: 'entity', schemaType: 'character', slug: 'unknown' });

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].title).toBe('unknown');
    });
  });

  describe('navigateTo — stats', () => {
    it('opens a stats tab', () => {
      navigationStore.navigateTo({ type: 'stats' });

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].id).toBe('stats');
      expect(editorState.tabs[0].documentType).toBe('stats');
    });
  });

  describe('navigateTo — schema', () => {
    it('dispatches sakya:edit-schema event for existing type', () => {
      const handler = vi.fn();
      window.addEventListener('sakya:edit-schema', handler);

      navigationStore.navigateTo({ type: 'schema', entityType: 'character' });

      expect(handler).toHaveBeenCalledTimes(1);
      const detail = (handler.mock.calls[0][0] as CustomEvent).detail;
      expect(detail.entityType).toBe('character');

      window.removeEventListener('sakya:edit-schema', handler);
    });

    it('dispatches sakya:new-schema event for new schema', () => {
      const handler = vi.fn();
      window.addEventListener('sakya:new-schema', handler);

      navigationStore.navigateTo({ type: 'schema', isNew: true });

      expect(handler).toHaveBeenCalledTimes(1);

      window.removeEventListener('sakya:new-schema', handler);
    });
  });

  describe('cross-type invariant', () => {
    it('only one type is selected after any navigateTo', () => {
      seedChapters('ch-1');
      seedNotes('note-1');

      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      expect(manuscriptStore.activeChapterSlug).toBe('ch-1');
      expect(notesStore.activeNoteSlug).toBe('');

      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });
      expect(manuscriptStore.activeChapterSlug).toBe('');
      expect(notesStore.activeNoteSlug).toBe('note-1');
    });

    it('sequential navigates clear previous selection', () => {
      seedChapters('ch-1', 'ch-2');

      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-2' });

      // ch-2 is active, ch-1 was cleared by first clearSelections() then re-set
      expect(manuscriptStore.activeChapterSlug).toBe('ch-2');
      expect(editorState.tabs).toHaveLength(2);
    });
  });

  describe('switchToTab', () => {
    it('restores chapter selection from tab', () => {
      seedChapters('ch-1', 'ch-2');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-2' });

      navigationStore.switchToTab('chapter:ch-1');

      expect(editorState.activeTabId).toBe('chapter:ch-1');
      expect(manuscriptStore.activeChapterSlug).toBe('ch-1');
    });

    it('restores note selection from tab', () => {
      seedNotes('note-1');
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      navigationStore.switchToTab('note:note-1');

      expect(editorState.activeTabId).toBe('note:note-1');
      expect(notesStore.activeNoteSlug).toBe('note-1');
      expect(manuscriptStore.activeChapterSlug).toBe('');
    });

    it('clears previous cross-type selection', () => {
      seedChapters('ch-1');
      seedNotes('note-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });

      // Now switch back to chapter tab
      navigationStore.switchToTab('chapter:ch-1');

      expect(manuscriptStore.activeChapterSlug).toBe('ch-1');
      expect(notesStore.activeNoteSlug).toBe('');
    });

    it('no-ops if tabId not found', () => {
      const prevTabId = editorState.activeTabId;
      navigationStore.switchToTab('nonexistent');
      expect(editorState.activeTabId).toBe(prevTabId);
    });
  });

  describe('closeActive', () => {
    it('closes the active tab', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      navigationStore.closeActive();

      expect(editorState.tabs).toHaveLength(0);
      expect(editorState.activeTabId).toBeNull();
    });

    it('clears selection for the closed type', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      navigationStore.closeActive();

      expect(manuscriptStore.activeChapterSlug).toBe('');
    });

    it('no-ops if no active tab', () => {
      navigationStore.closeActive();
      expect(editorState.tabs).toHaveLength(0);
    });
  });

  describe('closeTab', () => {
    it('closes a specific tab', () => {
      seedChapters('ch-1', 'ch-2');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-2' });

      navigationStore.closeTab('chapter:ch-1');

      expect(editorState.tabs).toHaveLength(1);
      expect(editorState.tabs[0].id).toBe('chapter:ch-2');
    });

    it('restores selection of newly active tab after close', () => {
      seedChapters('ch-1', 'ch-2');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-2' });

      // Close ch-2 (active) — ch-1 becomes active
      navigationStore.closeTab('chapter:ch-2');

      expect(editorState.activeTabId).toBe('chapter:ch-1');
      expect(manuscriptStore.activeChapterSlug).toBe('ch-1');
    });

    it('clears note selection when closing a note tab', () => {
      seedNotes('note-1');
      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });

      navigationStore.closeTab('note:note-1');

      expect(notesStore.activeNoteSlug).toBe('');
    });

    it('clears entity selection when closing an entity tab', () => {
      seedEntities('character', 'hero');
      navigationStore.navigateTo({ type: 'entity', schemaType: 'character', slug: 'hero' });

      navigationStore.closeTab('entity:character:hero');

      expect(entityStore.currentEntity).toBeNull();
    });

    it('no-ops if tabId not found', () => {
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      navigationStore.closeTab('nonexistent');

      expect(editorState.tabs).toHaveLength(1);
    });

    it('restores note selection when closing chapter tab reveals note tab', () => {
      seedNotes('note-1');
      seedChapters('ch-1');
      navigationStore.navigateTo({ type: 'note', slug: 'note-1' });
      navigationStore.navigateTo({ type: 'chapter', slug: 'ch-1' });

      // Close chapter — note becomes active
      navigationStore.closeTab('chapter:ch-1');

      expect(editorState.activeTabId).toBe('note:note-1');
      expect(notesStore.activeNoteSlug).toBe('note-1');
      expect(manuscriptStore.activeChapterSlug).toBe('');
    });
  });
});
