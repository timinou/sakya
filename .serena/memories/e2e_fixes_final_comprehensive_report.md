# E2E Test Failures: Comprehensive Root Cause Analysis and Fix Guide

**Date:** February 14, 2026  
**Total Failing Tests:** 15  
**Test Files Affected:** 4

---

## Executive Summary

### Categories of Issues:

1. **Missing Corkboard View Implementation (5 tests)** - EditorArea doesn't render corkboard based on viewMode
2. **Tab Closure Logic Broken (3 tests)** - editorState.closeTab() doesn't properly update active tab
3. **Override Functions Break During Second openMockProject (3 tests)** - Serialization/conflict issues
4. **Create Commands Not Invoking IPC (4 tests)** - Likely mock or invocation issue

---

## Detailed Issue Breakdown

### ISSUE #1: Corkboard View Not Rendering (5 Tests)

#### Failing Tests:
- `notes-corkboard.spec.ts:167` - "switching to corkboard view renders note cards"
- `notes-corkboard.spec.ts:188` - "corkboard note cards show title and label badges"  
- `notes-corkboard.spec.ts:208` - "corkboard cards have color strips matching note colors"
- `notes-corkboard.spec.ts:227` - "corkboard shows empty state when no notes exist"
- (Partial) `search-and-links.spec.ts:78` - "typing in search field..." (search_project not called)

#### Root Cause:
**EditorArea.svelte doesn't check uiState.viewMode**

- Toolbar.svelte has 3 buttons: Editor, Corkboard, Split (lines 28-51)
- Buttons call `uiState.setViewMode(mode)` ✓
- EditorArea.svelte (lines 86-110) ONLY renders chapter editor
- NO conditional logic based on `uiState.viewMode`
- Corkboard.svelte exists and is fully implemented but never imported/used

#### Implementation Status:
- ✅ Corkboard.svelte: Complete (has `.corkboard` class, empty state, note cards, color strips, labels)
- ✅ NoteCard.svelte: Complete (all visual elements present)
- ✅ Toolbar.svelte: Complete (view mode buttons work)
- ❌ EditorArea.svelte: INCOMPLETE (doesn't render based on viewMode)

#### What Needs to Change:
**File:** `/home/user/code/personal/sakya/src/lib/components/layout/EditorArea.svelte`

Lines 86-110 currently show:
```svelte
<div class="editor-area">
  <EditorTabs />
  {#if isLoadingContent}
    ...
  {:else if activeContent}
    ...
  {:else if !editorState.activeTab}
    ...
  {/if}
</div>
```

**Must become:**
```svelte
<div class="editor-area">
  {#if uiState.viewMode === 'editor'}
    <EditorTabs />
    {#if isLoadingContent}
      ...
    {:else if activeContent}
      ...
    {:else if !editorState.activeTab}
      ...
    {/if}
  {:else if uiState.viewMode === 'corkboard'}
    <Corkboard {notes} {noteExcerpts} onSelectNote={handleSelectNote} />
  {:else if uiState.viewMode === 'split'}
    <!-- Split view: both editor and corkboard -->
    <div class="split-container">
      <div class="split-editor">
        <EditorTabs />
        ...
      </div>
      <div class="split-corkboard">
        <Corkboard {notes} {noteExcerpts} onSelectNote={handleSelectNote} />
      </div>
    </div>
  {/if}
</div>
```

---

### ISSUE #2: Tab Closure Doesn't Update Focus (3 Tests)

#### Failing Tests:
- `writing-workflow.spec.ts:177` - "closing a tab removes it and shows empty state if last"
- `writing-workflow.spec.ts:197` - "closing one tab when multiple are open switches to remaining tab"
- `search-and-links.spec.ts:316` - "Cmd+W closes the active editor tab"

#### Root Cause:
**editorState.closeTab() has broken logic**

- EditorTabs.svelte properly calls closeTab() with correct aria-label ✓
- But tabs aren't being removed or focus isn't switching
- Test expects remaining tab to have `aria-selected="true"` but gets `aria-selected="false"`

#### Implementation Status:
- ✅ EditorTabs.svelte: Properly wired (line 11 calls `editorState.closeTab()`)
- ❌ editorState store: closeTab() logic is broken

#### What Needs to Change:
**File:** Need to find and fix `/home/user/code/personal/sakya/src/lib/stores/editor.svelte.ts`

The closeTab() method must:
1. Remove the tab from the tabs array
2. If the closed tab was the active tab AND there are other tabs:
   - Activate the next tab (or previous if it was the last one)
3. If the closed tab was the only tab:
   - Clear activeTabId

**Verify current implementation does this correctly.**

---

### ISSUE #3: Override Functions Cause Second openMockProject to Fail (3 Tests)

#### Failing Tests:
- `entity-workflow.spec.ts:182` - "shows placeholder when entity type has no entities"
- `writing-workflow.spec.ts:327` - "shows placeholder when manuscript has no chapters"
- `notes-corkboard.spec.ts:305` - "shows placeholder when no notes exist"

#### Root Cause:
**Multiple causes:**

1. **Page.goto("about:blank") is unnecessary and breaks the flow**
   - Test navigates away from app
   - Then openMockProject() tries to reinitialize
   - Second page.addInitScript() may conflict with first

2. **Override function in entity test references external variable**
   - Line 188-191 of entity-workflow.spec.ts:
   ```typescript
   list_entities: (args: Record<string, unknown> | undefined) => {
       if ((args?.schemaType as string) === "character") return [];
       return MOCK_ENTITIES_BY_TYPE[(args?.schemaType as string)] ?? [];
   }
   ```
   - This function references `MOCK_ENTITIES_BY_TYPE` which doesn't exist in browser
   - When `.toString()` serializes it, the reference is lost
   - eval() in browser fails with undefined variable

#### Implementation Status:
- ✅ Override functions for notes/manuscript are self-contained: `() => ({ notes: [] })`
- ❌ Override function for entity references external data
- ⚠️ Test setup with page.goto("about:blank") is problematic

#### What Needs to Change:
**File:** `/home/user/code/personal/sakya/e2e/entity-workflow.spec.ts`

**Line 182-199:**
Remove `page.goto("about:blank")` OR make override function self-contained.

**Option 1: Remove the unnecessary navigation**
```typescript
test("shows placeholder when entity type has no entities", async ({ page }) => {
  // Don't call page.goto("about:blank") - it's unnecessary
  await openMockProject(page, {
    list_entities: (args: Record<string, unknown> | undefined) => {
      const entities = {
        character: [],
        place: [
          { title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: ["city", "capital"] },
          { title: "The Whispering Woods", slug: "the-whispering-woods", schemaType: "place", tags: ["forest", "enchanted"] }
        ]
      };
      return (entities as Record<string, unknown[]>)[(args?.schemaType as string)] ?? [];
    },
  });
  ...
});
```

**Option 2: Use a plain value instead of function**
Use the defaults and only override what's needed to be simpler.

**Files to Fix:**
- `entity-workflow.spec.ts:182-199`
- `writing-workflow.spec.ts:327-336`
- `notes-corkboard.spec.ts:305-312`

---

### ISSUE #4: Create Commands Not Recording IPC Calls (4 Tests)

#### Failing Tests:
- `notes-corkboard.spec.ts:79` - "typing title and pressing Enter creates a new note"
- `notes-corkboard.spec.ts:248` - "clicking New Note button in empty corkboard creates a note"
- `writing-workflow.spec.ts:93` - "clicking chapter calls get_chapter for content loading"
- `writing-workflow.spec.ts:250` - "typing title and pressing Enter creates a new chapter"

#### Root Cause:
**UNCERTAIN - Multiple possible causes:**

1. **NotesSection.svelte DOES call the store correctly** (line 35):
   ```typescript
   await notesStore.createNote(projectState.projectPath, title);
   ```

2. **Notes store DOES invoke the command** (verified):
   ```typescript
   await invoke('create_note', { projectPath, title });
   ```

3. **Mock IS registered** in setupDefaultTauriMocks() (line 492):
   ```typescript
   create_note: ((args: Record<string, unknown> | undefined) => ({...}))
   ```

4. **But test shows 0 IPC calls recorded**

#### Possible Root Causes:
1. The invoke() promise isn't being awaited or completes before test checks
2. The IPC call recording mechanism isn't working for dynamically called commands
3. The mock isn't properly set up for this command
4. There's an error in invoke() that's being swallowed
5. The test timing is off - needs longer wait

#### Implementation Status:
- ✅ NotesSection.svelte: Correctly calls createNote()
- ✅ Notes store: Correctly invokes 'create_note'
- ✅ Mocks: Registered in setupDefaultTauriMocks()
- ❌ Unknown: Why IPC call isn't being recorded

#### What Needs to Check/Fix:
1. **Debug the actual test:**
   - Add logging to see if createNote() is even called
   - Add logging inside the store to verify invoke() is reached
   - Check if there's an error being thrown

2. **Verify test assumptions:**
   - Are the notes actually being created (check if new note appears)?
   - Is the IPC recording mechanism working at all?
   - Test simpler case first (check if get_notes_config is recorded)

3. **Possible fixes:**
   - May need to add longer wait after pressing Enter
   - May need to verify input blur triggers the creation
   - May need to check NotesSection's input onblur vs onkeydown handlers

#### Investigation Steps:
```typescript
// Add to test to debug:
await page.waitForTimeout(200); // Wait after Enter
const allCalls = await getIpcCalls(page);
console.log('All IPC calls:', allCalls);
const createCalls = await getIpcCallsByCommand(page, "create_note");
console.log('Create calls:', createCalls);

// Also check if note appears in UI
const notes = await page.locator(".note-row").count();
console.log('Note count after create:', notes);
```

---

## Fix Priority and Implementation Order

### Priority 1 (Blocks 5+ tests):
**EditorArea viewMode rendering**
- Estimated effort: Medium (1-2 hours)
- Impact: Fixes 5 tests immediately
- Implementation: Add conditional rendering in EditorArea.svelte

### Priority 2 (Blocks 3 tests):
**Tab closure logic**
- Estimated effort: Low-Medium (30-60 minutes)
- Impact: Fixes 3 tests immediately  
- Implementation: Fix editorState.closeTab() method

### Priority 3 (Blocks 4 tests):
**Create commands IPC calls**
- Estimated effort: Low-Medium (1-2 hours)
- Impact: Fixes 4 tests
- Implementation: Debug and fix - likely small issue

### Priority 4 (Blocks 3 tests):
**Override functions and test setup**
- Estimated effort: Low (30-45 minutes)
- Impact: Fixes 3 tests
- Implementation: Fix override functions and remove page.goto("about:blank")

---

## Testing Verification Checklist

After implementing each fix:

1. **EditorArea viewMode fix:**
   - [ ] Corkboard renders when view mode is "corkboard"
   - [ ] Empty state appears when no notes
   - [ ] Note cards render with all visual elements
   - [ ] View mode toggle works

2. **Tab closure fix:**
   - [ ] Closing last tab shows empty state
   - [ ] Closing one of multiple tabs switches to next
   - [ ] Remaining tab has aria-selected="true"
   - [ ] Cmd+W works

3. **Create commands fix:**
   - [ ] create_note IPC call is recorded
   - [ ] create_chapter IPC call is recorded
   - [ ] New items appear in binder after creation
   - [ ] Correct title is passed to command

4. **Override functions fix:**
   - [ ] Empty state test for entities works
   - [ ] Empty state test for chapters works
   - [ ] Empty state test for notes works
   - [ ] No serialization errors

---

## Code Navigation Guide

**Key Files:**
- EditorArea: `/home/user/code/personal/sakya/src/lib/components/layout/EditorArea.svelte`
- Corkboard: `/home/user/code/personal/sakya/src/lib/components/notes/Corkboard.svelte`
- Toolbar: `/home/user/code/personal/sakya/src/lib/components/layout/Toolbar.svelte`
- Editor store: `/home/user/code/personal/sakya/src/lib/stores/editor.svelte.ts`
- Notes store: `/home/user/code/personal/sakya/src/lib/stores/notes.svelte.ts`
- Manuscript store: `/home/user/code/personal/sakya/src/lib/stores/manuscript.svelte.ts`
- EditorTabs: `/home/user/code/personal/sakya/src/lib/components/layout/EditorTabs.svelte`
- NotesSection: `/home/user/code/personal/sakya/src/lib/components/binder/NotesSection.svelte`
- Test utils: `/home/user/code/personal/sakya/e2e/utils/tauri-mocks.ts`

**Test Files:**
- Entity workflow: `/home/user/code/personal/sakya/e2e/entity-workflow.spec.ts`
- Notes corkboard: `/home/user/code/personal/sakya/e2e/notes-corkboard.spec.ts`
- Writing workflow: `/home/user/code/personal/sakya/e2e/writing-workflow.spec.ts`
- Search and links: `/home/user/code/personal/sakya/e2e/search-and-links.spec.ts`

---

## Running Tests During Development

```bash
# Run specific test file
bun run test:e2e -- e2e/notes-corkboard.spec.ts

# Run specific test pattern
bun run test:e2e -- --grep "corkboard"

# Run all tests
bun run test:e2e

# View test results
# Open test-results/*.html in browser
```

---

## Dependencies to Verify

- All components are already imported where needed
- uiState store already has viewMode property
- notesStore already has createNote() method
- manuscriptStore already has createChapter() method
- All mocks are properly registered in setupDefaultTauriMocks()

No external dependencies need to be added - all code infrastructure exists, just needs to be wired together.
