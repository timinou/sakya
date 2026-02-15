# ITEM-009: Project Launcher Page

## Summary
Replaced the Tauri+Svelte boilerplate page with a project launcher for the Sakya writing application.

## Key Files Modified
- `src/routes/+layout.svelte` - Added theme provider using `$effect` to set `data-theme` attribute
- `src/routes/+page.svelte` - Full launcher with Create/Open project flows, dynamic AppShell import
- `src/routes/page.test.ts` - 9 vitest unit tests covering launcher UI
- `src/tests/setup.ts` - Added `@tauri-apps/plugin-dialog` mock
- `vitest.config.ts` - Added `$lib` path alias for vitest
- `e2e/utils/tauri-mocks.ts` - Added project command and dialog plugin mocks
- `e2e/app.spec.ts` - 6 launcher E2E tests
- `e2e/visual.spec.ts` - 2 visual regression tests with updated baselines

## Architecture Notes
- The page uses `@tauri-apps/plugin-dialog` `open()` for native folder picker
- When `projectState.isOpen`, it dynamically imports `AppShell.svelte` with `{#await import(...)}`
- Create form is inline (not modal), toggles visibility via `showCreateForm` state
- All CSS uses the custom properties from `app.css`

## Remaining Work
- Recent projects list (not yet implemented - acceptance criteria unchecked)
