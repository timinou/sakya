# E2E Test Fixes - Detailed Analysis

## Issue 1: Missing Corkboard View in EditorArea
**Status:** ROOT CAUSE FOUND

**Tests Affected:**
- notes-corkboard.spec.ts:167 - "switching to corkboard view renders note cards"
- notes-corkboard.spec.ts:188 - "corkboard note cards show title and label badges"
- notes-corkboard.spec.ts:208 - "corkboard cards have color strips matching note colors"
- notes-corkboard.spec.ts:227 - "corkboard shows empty state when no notes exist"

**Problem:**
- EditorArea.svelte ONLY renders the chapter editor view
- It doesn't check `uiState.viewMode` at all
- Toolbar.svelte has buttons to switch view mode ("Editor", "Corkboard", "Split") which call `uiState.setViewMode()`
- But EditorArea.svelte never reads `uiState.viewMode` to conditionally render different content
- Corkboard.svelte component EXISTS and is properly implemented, but isn't being rendered

**Component Status:**
- EditorArea.svelte: Lines 86-110 - Shows editor tabs, loading state, editor, or empty state
- NO conditional rendering based on `uiState.viewMode`
- Corkboard.svelte: FULLY IMPLEMENTED (lines 75-164) with:
  - `.corkboard` container class
  - `.empty-state` with "Create your first note" message when notes.length === 0
  - NoteCard components for each note
  - Color strips, label badges, all working

**Required Fix:**
EditorArea.svelte needs to be modified to:
1. Import Corkboard component
2. Read `uiState.viewMode`
3. Conditionally render based on viewMode:
   - 'editor': Current behavior (editor with tabs)
   - 'corkboard': Show corkboard with notes
   - 'split': Show both side-by-side
4. Handle note selection in corkboard

**Code Locations:**
- EditorArea: /home/user/code/personal/sakya/src/lib/components/layout/EditorArea.svelte (lines 86-110)
- Corkboard: /home/user/code/personal/sakya/src/lib/components/notes/Corkboard.svelte (fully implemented)
- Toolbar: /home/user/code/personal/sakya/src/lib/components/layout/Toolbar.svelte (lines 27-52 - view mode buttons work)

---

## Issue 2: Tab Closure Not Working
**Status:** ROOT CAUSE LIKELY IN EDITORSTATE STORE

**Tests Affected:**
- writing-workflow.spec.ts:177 - "closing a tab removes it and shows empty state if last"
- writing-workflow.spec.ts:197 - "closing one tab when multiple are open switches to remaining tab"
- search-and-links.spec.ts:316 - "Cmd+W closes the active editor tab"

**Problem:**
- Tests click "Close The Awakening" button (aria-label from EditorTabs.svelte:37)
- The close button exists and has proper aria-label
- But tabs are NOT being removed
- When multiple tabs exist and one is closed, the remaining tab's aria-selected is "false" instead of "true"

**Component Status:**
- EditorTabs.svelte: PROPERLY IMPLEMENTED
  - Line 37: `aria-label="Close {tab.title}"` ✓
  - Line 11: `closeTab()` function calls `editorState.closeTab(tabId)` ✓
- EditorArea.svelte: Uses editorState but doesn't show if tabs are being removed

**Required Fix:**
Need to check and potentially fix editorState.closeTab() method:
1. Find the editorState store definition
2. Verify closeTab() method:
   - Removes tab from tabs array
   - If closed tab was active and other tabs exist, make next tab active
   - If was the only tab, clear activeTabId
3. If logic is wrong, fix it

**Likely Location:**
- /home/user/code/personal/sakya/src/lib/stores/ (editorState definition)

---

## Issue 3: Create Note/Chapter IPC Calls Not Recorded
**Status:** UNCERTAIN - LIKELY STORE ISSUE

**Tests Affected:**
- notes-corkboard.spec.ts:79 - "typing title and pressing Enter creates a new note"
- notes-corkboard.spec.ts:248 - "clicking New Note button in empty corkboard creates a note"
- writing-workflow.spec.ts:93 - "clicking chapter calls get_chapter for content loading"
- writing-workflow.spec.ts:250 - "typing title and pressing Enter creates a new chapter"

**Problem:**
- Tests clear IPC calls, then trigger create action
- Tests check if create_note or create_chapter IPC calls were made
- Result: No calls recorded (length === 0)

**Component Status:**
- NotesSection.svelte: Line 35 calls `await notesStore.createNote(projectState.projectPath, title)` ✓
- ManuscriptSection.svelte: Similar pattern (need to verify)
- Both should be calling the stores, which should invoke the commands

**Hypothesis:**
1. The notesStore.createNote() method might not be calling invoke('create_note', ...)
2. Or it's calling with wrong command name
3. Or the promise isn't properly set up

**Required Fix:**
Need to verify the stores are actually invoking the IPC commands:
1. Find notesStore.createNote() implementation
2. Verify it calls `invoke('create_note', { title, ... })`
3. Same for manuscript store's createChapter()

**Likely Location:**
- /home/user/code/personal/sakya/src/lib/stores/ (notesStore, manuscriptStore)

---

## Issue 4: Empty State Tests with Override Functions
**Status:** TEST SETUP ISSUE

**Tests Affected:**
- entity-workflow.spec.ts:182 - "shows placeholder when entity type has no entities"
- writing-workflow.spec.ts:327 - "shows placeholder when manuscript has no chapters"
- notes-corkboard.spec.ts:305 - "shows placeholder when no notes exist"

**Problem:**
Tests do:
```typescript
await page.goto("about:blank");
await openMockProject(page, {
  get_notes_config: () => ({ notes: [] }),
});
```

Then wait for "Binder" to appear in the app, but it never does (TimeoutError).

**Root Cause Analysis:**
1. `page.goto("about:blank")` navigates away from the app
2. `openMockProject(page, overrides)` does:
   - Calls `setupDefaultTauriMocks(page, overrides)` which adds an init script
   - Calls `page.goto("/")` which should trigger the init script
   - Calls `page.getByRole("button", { name: /open project/i }).click()` and waits for "Binder" to appear

The problem: When init scripts are registered AFTER the page has already loaded (from previous beforeEach), adding a NEW init script via addInitScript() might not work correctly. The init scripts may conflict or not run properly.

**Specific Issue with entity-workflow.spec.ts:182:**
The override function is:
```typescript
list_entities: (args: Record<string, unknown> | undefined) => {
    if ((args?.schemaType as string) === "character") return [];
    return MOCK_ENTITIES_BY_TYPE[(args?.schemaType as string)] ?? [];
},
```

This references `MOCK_ENTITIES_BY_TYPE` - an external variable. When this function is serialized via `.toString()` and eval'd in the browser, `MOCK_ENTITIES_BY_TYPE` doesn't exist in that scope.

**Solution:**
1. For entity test: Replace the override with a self-contained function that doesn't reference external variables:
   ```typescript
   list_entities: (args: Record<string, unknown> | undefined) => {
     const entities = {
       character: [],
       place: [
         { title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: ["city", "capital"] },
         { title: "The Whispering Woods", slug: "the-whispering-woods", schemaType: "place", tags: ["forest", "enchanted"] }
       ]
     };
     return (entities as Record<string, unknown[]>)[(args?.schemaType as string)] ?? [];
   }
   ```

2. Better solution: Remove the `page.goto("about:blank")` call entirely. It's unnecessary. Instead, create a completely fresh test that doesn't use beforeEach's openMockProject, or have beforeEach NOT call openMockProject and handle it in each test individually.

3. For the notes/manuscript tests, the override functions ARE self-contained:
   - `() => ({ notes: [] })`
   - `() => ({ chapters: [] })`
   
   These should work fine. The issue is likely the second `openMockProject` call conflicting with the first one from beforeEach.

**Recommended Fix:**
- Remove `page.goto("about:blank")` from all three tests
- Instead, have each test call `openMockProject(page, overrides)` FIRST without the beforeEach setup
- OR modify beforeEach to NOT call openMockProject, and have individual tests call it as needed

---

## Issue 5: Search Results Not Appearing
**Status:** UNCERTAIN - LIKELY MOCK OR DEBOUNCE ISSUE

**Tests Affected:**
- search-and-links.spec.ts:78 - "typing in search field triggers search and displays results"
- (Indirectly affects other search tests)

**Problem:**
Test types "Elena" in search field, waits 500ms for debounced search, but `search_project` IPC command is never called (length === 0).

**Component Status:**
- SearchPalette.svelte: Properly implemented
  - Line 122-135: debounced search with 300ms timeout
  - Line 147: calls `invoke('search_project', { projectPath, query })`
  - Line 284-331: renders results grouped by type with headers

**Hypothesis:**
1. The invoke('search_project', ...) isn't being called
2. Or the mock isn't registered for search_project
3. Or projectState.projectPath is undefined/null
4. Or the debounce timer is being cleared

**Required Fix:**
1. Verify projectState.projectPath is set when openMockProject completes
2. Verify the mock for search_project is registered in setupDefaultTauriMocks()
3. Add logging to SearchPalette.svelte to debug if invoke() is actually being called
4. Check if there's a timing issue with when the app initializes

---

## Summary of Required Fixes

### Priority 1 (Breaking multiple tests):
1. **EditorArea missing corkboard view** - Add conditional rendering based on uiState.viewMode
   - Affects 4 tests
   - Component fully exists, just needs to be wired up

### Priority 2 (Breaking multiple tests):
2. **Override functions in empty state tests** - Fix serialization issues
   - Affects 3 tests
   - Either fix override function to be self-contained OR remove page.goto("about:blank")

### Priority 3 (Breaking multiple tests):
3. **Tab closure logic** - Fix editorState.closeTab() implementation
   - Affects 3 tests
   - Logic needs to properly remove tabs and switch focus

### Priority 4 (Breaking multiple tests):
4. **Create note/chapter not invoking IPC** - Verify stores are calling invoke()
   - Affects 4 tests
   - Stores need to properly invoke the commands

### Priority 5 (Breaking search functionality):
5. **Search project not being called** - Debug invoke() in SearchPalette
   - Affects 2 tests
   - Either mock isn't registered or projectPath is undefined

---

## Testing Strategy for Fixes

1. After each fix, run the affected tests:
   ```bash
   bun run test:e2e -- --grep "pattern"
   ```

2. Use visual tests (screenshots in test-results/) to verify UI is correct

3. Debug IPC calls using:
   ```typescript
   const calls = await getIpcCallsByCommand(page, "command_name");
   console.log(calls);
   ```

4. Verify component rendering with:
   ```typescript
   await page.screenshot({ path: "debug.png" });
   ```
