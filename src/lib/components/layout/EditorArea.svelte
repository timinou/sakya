<script lang="ts">
  import { manuscriptStore, notesStore, editorState, projectState, entityStore } from '$lib/stores';
  import type { EntitySchema } from '$lib/types';
  import SakyaEditor from '$lib/editor/SakyaEditor.svelte';
  import SchemaEditor from '$lib/components/entities/SchemaEditor.svelte';
  import EditorTabs from './EditorTabs.svelte';
  import WelcomeCard from './WelcomeCard.svelte';

  // Cached content shape — only slug and body are needed for the editor
  type CachedContent = { slug: string; body: string };

  // Track loaded content per tab
  let contentCache = $state<Record<string, CachedContent>>({});
  let isLoadingContent = $state(false);

  // Schema editor state — keyed by tab ID
  let schemaCache = $state<Record<string, { schema: EntitySchema; isNew: boolean }>>({});

  let activeContent = $derived(
    editorState.activeTab ? contentCache[editorState.activeTab.id] ?? null : null
  );

  // When activeChapterSlug changes, open a tab and load content
  $effect(() => {
    const slug = manuscriptStore.activeChapterSlug;
    const path = projectState.projectPath;
    if (!slug || !path) return;

    const tabId = `chapter:${slug}`;
    const chapter = manuscriptStore.activeChapter;
    if (!chapter) return;

    // Open tab (idempotent - if already open, just switches to it)
    editorState.openDocument({
      id: tabId,
      title: chapter.title,
      documentType: 'chapter',
      documentSlug: slug,
      isDirty: false,
    });

    // Load content if not cached
    if (!contentCache[tabId]) {
      loadContent(path, slug, tabId);
    }
  });

  // When activeNoteSlug changes, open a tab and load content
  $effect(() => {
    const slug = notesStore.activeNoteSlug;
    const path = projectState.projectPath;
    if (!slug || !path) return;

    const tabId = `note:${slug}`;
    const note = notesStore.activeNote;
    if (!note) return;

    editorState.openDocument({
      id: tabId,
      title: note.title,
      documentType: 'note',
      documentSlug: slug,
      isDirty: false,
    });

    if (!contentCache[tabId]) {
      loadNoteContent(path, slug, tabId);
    }
  });

  // Derive active schema for schema tabs
  let activeSchema = $derived(
    editorState.activeTab?.documentType === 'schema'
      ? schemaCache[editorState.activeTab.id] ?? null
      : null
  );

  // Listen for schema editing events
  $effect(() => {
    function handleEditSchema(e: Event) {
      const detail = (e as CustomEvent<{ entityType: string }>).detail;
      openSchemaTab(detail.entityType);
    }

    function handleNewSchema() {
      openNewSchemaTab();
    }

    window.addEventListener('sakya:edit-schema', handleEditSchema);
    window.addEventListener('sakya:new-schema', handleNewSchema);

    return () => {
      window.removeEventListener('sakya:edit-schema', handleEditSchema);
      window.removeEventListener('sakya:new-schema', handleNewSchema);
    };
  });

  async function openSchemaTab(entityType: string) {
    const path = projectState.projectPath;
    if (!path) return;

    const tabId = `schema:${entityType}`;

    // Load schema if not already cached
    if (!schemaCache[tabId]) {
      isLoadingContent = true;
      try {
        const schema = await entityStore.getSchema(path, entityType);
        schemaCache[tabId] = { schema, isNew: false };
      } catch (e) {
        console.error('[EditorArea] Failed to load schema:', e);
        return;
      } finally {
        isLoadingContent = false;
      }
    }

    editorState.openDocument({
      id: tabId,
      title: `${schemaCache[tabId].schema.name} (Schema)`,
      documentType: 'schema',
      documentSlug: entityType,
      isDirty: false,
    });
  }

  function openNewSchemaTab() {
    const tabId = 'schema:__new__';

    if (!schemaCache[tabId]) {
      schemaCache[tabId] = {
        schema: {
          name: '',
          entityType: '',
          fields: [],
          spiderAxes: [],
        },
        isNew: true,
      };
    }

    editorState.openDocument({
      id: tabId,
      title: 'New Entity Type',
      documentType: 'schema',
      documentSlug: '__new__',
      isDirty: false,
    });
  }

  async function handleSchemaSave(schema: EntitySchema) {
    const path = projectState.projectPath;
    if (!path) return;

    const tab = editorState.activeTab;
    if (!tab) return;

    try {
      await entityStore.saveSchema(path, schema);
      // Clean up schema cache and close tab
      delete schemaCache[tab.id];
      editorState.closeTab(tab.id);
    } catch (e) {
      console.error('[EditorArea] Failed to save schema:', e);
    }
  }

  function handleSchemaCancel() {
    const tab = editorState.activeTab;
    if (!tab) return;

    delete schemaCache[tab.id];
    editorState.closeTab(tab.id);
  }

  async function loadContent(projectPath: string, slug: string, tabId: string) {
    isLoadingContent = true;
    try {
      const content = await manuscriptStore.loadChapterContent(projectPath, slug);
      contentCache[tabId] = { slug: content.slug, body: content.body };
    } catch (e) {
      console.error('[EditorArea] Failed to load chapter content:', e);
    } finally {
      isLoadingContent = false;
    }
  }

  async function loadNoteContent(projectPath: string, slug: string, tabId: string) {
    isLoadingContent = true;
    try {
      const content = await notesStore.loadNoteContent(projectPath, slug);
      contentCache[tabId] = { slug: content.slug, body: content.body };
    } catch (e) {
      console.error('[EditorArea] Failed to load note content:', e);
    } finally {
      isLoadingContent = false;
    }
  }

  async function handleSave(markdown: string) {
    const tab = editorState.activeTab;
    const path = projectState.projectPath;
    if (!tab || !path) return;

    if (tab.documentType === 'chapter') {
      const slug = tab.documentSlug;
      const chapter = manuscriptStore.chapters.find((c) => c.slug === slug);
      if (!chapter) return;
      await manuscriptStore.saveChapterContent(path, slug, chapter, markdown);
    } else if (tab.documentType === 'note') {
      const slug = tab.documentSlug;
      const note = notesStore.notes.find((n) => n.slug === slug);
      if (!note) return;
      await notesStore.saveNoteContent(path, slug, note.title, markdown);
    }

    editorState.setDirty(tab.id, false);

    // Update cached content
    if (contentCache[tab.id]) {
      contentCache[tab.id] = { ...contentCache[tab.id], body: markdown };
    }
  }

  function handleCountChange(counts: {
    words: number;
    characters: number;
    charactersNoSpaces: number;
  }) {
    editorState.updateWordCount(counts);
  }

  function handleDirty() {
    if (editorState.activeTab) {
      editorState.setDirty(editorState.activeTab.id, true);
    }
  }
</script>

<div class="editor-area">
  <EditorTabs />

  {#if isLoadingContent}
    <div class="editor-loading">
      <span class="loading-spinner"></span>
      <span>Loading...</span>
    </div>
  {:else if activeSchema}
    {#key editorState.activeTab?.id}
      <div class="editor-container schema-editor-container">
        <SchemaEditor
          schema={activeSchema.schema}
          isNew={activeSchema.isNew}
          onSave={handleSchemaSave}
          onCancel={handleSchemaCancel}
        />
      </div>
    {/key}
  {:else if activeContent}
    {#key activeContent.slug}
      <div class="editor-container" oninput={handleDirty}>
        <SakyaEditor
          content={activeContent.body}
          onSave={handleSave}
          onCountChange={handleCountChange}
        />
      </div>
    {/key}
  {:else if !editorState.activeTab}
    <WelcomeCard
      onCreateChapter={() => {
        window.dispatchEvent(new CustomEvent('sakya:create-chapter'));
      }}
      onCreateNote={() => {
        window.dispatchEvent(new CustomEvent('sakya:create-note'));
      }}
      onCreateEntity={(entityType) => {
        window.dispatchEvent(new CustomEvent('sakya:create-entity', { detail: { entityType } }));
      }}
    />
  {/if}
</div>

<style>
  .editor-area {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .schema-editor-container {
    overflow-y: auto;
  }

  .editor-loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-primary);
    border-top-color: var(--accent-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
