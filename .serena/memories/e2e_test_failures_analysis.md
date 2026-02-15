# E2E Test Failures Analysis and Fixes

**Total Failures: 15 across 4 test files**

## Summary by Category

### Category 1: Override Function Serialization Issues (3 tests)
Tests that call `openMockProject(page, overrides)` with override functions that reference external variables fail because functions are serialized via `.toString()` and eval'd in browser, losing closure context.

**Tests:**
1. `entity-workflow.spec.ts:182` - "shows placeholder when entity type has no entities"
2. `writing-workflow.spec.ts:327` - "shows placeholder when manuscript has no chapters"
3. `notes-corkboard.spec.ts:305` - "shows placeholder when no notes exist"

**Root Cause:**
- Tests call `page.goto("about:blank")` before `openMockProject(page, overrides)` 
- `openMockProject` adds init script via `page.addInitScript()` which only runs on NEXT navigation
- The `page.goto("/")` inside `openMockProject` is supposed to trigger the init script
- However, after the `page.goto("about:blank")`, the overrides are then applied with `openMockProject`
- For entity test: override function `list_entities: (args) => { if (args?.schemaType === "character") return []; return MOCK_ENTITIES_BY_TYPE[(args?.schemaType)] ?? []; }` references external `MOCK_ENTITIES_BY_TYPE` which doesn't exist in browser context after eval
- The override functions for manuscript and notes (`() => ({ chapters: [] })` and `() => ({ notes: [] })`) are self-contained and should serialize fine

**Actual Issue:** The problem is that `page.goto("about:blank")` is redundant. When tests call it, they navigate AWAY from the app. Then `openMockProject` is called, which sets up mocks and navigates to "/". This should work. BUT the timeout at waiting for "Binder" text suggests the app never fully initializes after the second `openMockProject` call.

**Hypothesis:** When `addInitScript` is called a SECOND time (in the test after beforeEach already set it up once), something goes wrong with either:
1. The scripts conflicting
2. The init script not running properly on the second navigation
3. The overrides not being properly merged with defaults

**Fix Required:** 
- For `entity-workflow.spec.ts:182`: The override function must be self-contained. Instead of referencing `MOCK_ENTITIES_BY_TYPE`, inline the data as a plain value or use a self-contained function that doesn't reference external variables.
- For the other two tests: The overrides should work fine since they're self-contained. The issue is likely in how `openMockProject` handles being called a second time with overrides.
- Alternative fix: Don't call `page.goto("about:blank")` - it's unnecessary and causes issues. Just call `openMockProject(page, overrides)` directly (it already handles navigation).

---

### Category 2: IPC Call Failures - Create Note/Chapter (4 tests)
Tests that create notes/chapters fail because the create_note/create_chapter IPC calls aren't being recorded.

**Tests:**
1. `notes-corkboard.spec.ts:79` - "typing title and pressing Enter creates a new note"
2. `notes-corkboard.spec.ts:248` - "clicking New Note button in empty corkboard creates a note"
3. `writing-workflow.spec.ts:93` - "clicking chapter calls get_chapter for content loading"  
4. `writing-workflow.spec.ts:250` - "typing title and pressing Enter creates a new chapter"

**Root Cause:**
The inline input UI for creating notes/chapters isn't calling the IPC commands. Either:
1. The NotesSection.svelte or ManuscriptSection.svelte components don't actually call the create commands
2. The commands are called but with different naming (e.g., snake_case vs camelCase)
3. The mocks aren't set up to handle the actual arguments being passed

**Fix Required:** Check the actual components to see if they wire up the create functionality. The tests expect the create commands to be called but they're not being invoked. Either:
- Fix the components to actually call create_note/create_chapter
- OR fix the tests to match the actual behavior (if create functionality isn't implemented)

---

### Category 3: Corkboard Component Missing (5 tests)
Tests that expect a `.corkboard` element to exist fail - the corkboard isn't rendering.

**Tests:**
1. `notes-corkboard.spec.ts:167` - "switching to corkboard view renders note cards"
2. `notes-corkboard.spec.ts:188` - "corkboard note cards show title and label badges"
3. `notes-corkboard.spec.ts:208` - "corkboard cards have color strips matching note colors"
4. `notes-corkboard.spec.ts:227` - "corkboard shows empty state when no notes exist"
5. `search-and-links.spec.ts:78` - "typing in search field triggers search and displays results"

**Root Cause:**
- The Corkboard.svelte component EXISTS and IS properly implemented
- `.corkboard` class is defined in the component
- Tests click the "Corkboard" view button to switch view mode
- But the corkboard element isn't appearing

**Possible Causes:**
1. The EditorArea component that should show the corkboard isn't wired up to the view mode state
2. The corkboard button isn't actually setting the view mode
3. The view mode store isn't being used properly to show/hide corkboard

**Fix Required:** Need to check EditorArea.svelte to see if it conditionally renders Corkboard based on view mode. If not, wire up the view mode toggle.

---

### Category 4: Search Results Grouping (2 tests)
Tests that expect search results grouped by type (Chapters, Entities, Notes) fail.

**Tests:**
1. `search-and-links.spec.ts:78` - "typing in search field triggers search and displays results"
2. `search-and-links.spec.ts:106` - "search results show titles and matching lines"

**Root Cause:**
The SearchPalette.svelte component DOES implement grouping with result-group-header divs that contain group labels. The tests look for `page.getByText("Chapters")` etc.

BUT: The search is failing because:
1. Either the search_project mock isn't being called
2. Or the mock returns empty results
3. Or the search debounce isn't completing in time

The test has `await page.waitForTimeout(500)` to wait for debounced search (300ms + overhead). This should be sufficient.

**Fix Required:**
- Check that search_project is being called at all (test checks `searchCalls.length >= 1`)
- The error shows searchCalls.length is 0, meaning search_project wasn't called
- This could be because:
  1. The invoke() in SearchPalette isn't executing
  2. The mock isn't registered
  3. The test setup isn't completing properly

---

### Category 5: Tab Closure and Navigation Issues (3 tests)
Tests that close tabs or navigate away fail because tabs aren't being removed or focus isn't switching.

**Tests:**
1. `writing-workflow.spec.ts:177` - "closing a tab removes it and shows empty state if last"
2. `writing-workflow.spec.ts:197` - "closing one tab when multiple are open switches to remaining tab"
3. `search-and-links.spec.ts:316` - "Cmd+W closes the active editor tab"

**Root Cause:**
- The tab close button uses `aria-label="Close {tab.title}"`
- Tests use `page.getByLabel("Close The Awakening")` which should work
- But the tab doesn't actually close, or if it does, the next tab isn't becoming active
- Tests expect `aria-selected="true"` on remaining tab, but it has `aria-selected="false"`

**Possible Causes:**
1. The closeTab function in EditorTabs.svelte isn't actually removing the tab
2. The editorState.closeTab() method isn't implemented or is buggy
3. The logic to activate the next tab after closing isn't working

**Fix Required:** Check editorState.closeTab() logic to ensure:
1. Tab is removed from the list
2. If it was the active tab and there are other tabs, make the next one active
3. If it was the last tab, show empty state

---

## Implementation Status

Component checks completed:
- ✅ Corkboard.svelte - PROPERLY IMPLEMENTED (has .corkboard class, empty state, note cards)
- ✅ NoteCard.svelte - PROPERLY IMPLEMENTED (has .note-card, .color-strip, .label-badge classes)
- ✅ EditorTabs.svelte - PROPERLY IMPLEMENTED (has close button with `aria-label="Close {tab.title}"`)
- ✅ SearchPalette.svelte - PROPERLY IMPLEMENTED (has result grouping with headers for "Chapters", "Entities", "Notes")

## Recommended Fix Order

1. **First:** Fix the `page.goto("about:blank")` issue by removing these calls - they're redundant
2. **Second:** Fix entity empty state test override function to not reference external variables
3. **Third:** Debug and fix the IPC call recording for create_note/create_chapter
4. **Fourth:** Debug why corkboard isn't rendering when view mode is switched
5. **Fifth:** Debug why search_project isn't being called
6. **Sixth:** Debug tab closure and focus switching logic

All components appear to be implemented correctly. The issues are likely in:
- Test setup (openMockProject with overrides on already-opened project)
- View mode switching logic in EditorArea
- IPC mock registration or invocation
- Tab state management in editorState store
