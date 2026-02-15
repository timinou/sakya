import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

// =============================================================================
// Distraction-Free, Typewriter, and Focus Mode E2E Tests (ITEM-098)
// =============================================================================

test.describe("Distraction-Free Mode", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Activating / deactivating distraction-free mode
  // ---------------------------------------------------------------------------

  test("Ctrl+Shift+F activates distraction-free mode and adds .distraction-free class", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");
    await expect(appShell).not.toHaveClass(/distraction-free/);

    await page.keyboard.press("Control+Shift+F");

    await expect(appShell).toHaveClass(/distraction-free/);
  });

  test("toolbar is hidden in distraction-free mode", async ({ page }) => {
    const toolbar = page.locator(".toolbar");
    await expect(toolbar).toBeVisible();

    await page.keyboard.press("Control+Shift+F");

    // Toolbar should be visually hidden (opacity 0, pointer-events none)
    await expect(toolbar).toHaveCSS("opacity", "0");
    await expect(toolbar).toHaveCSS("pointer-events", "none");
  });

  test("status bar is hidden in distraction-free mode", async ({ page }) => {
    const statusBar = page.locator(".status-bar");
    await expect(statusBar).toBeVisible();

    await page.keyboard.press("Control+Shift+F");

    await expect(statusBar).toHaveCSS("opacity", "0");
    await expect(statusBar).toHaveCSS("pointer-events", "none");
  });

  test("binder pane is hidden in distraction-free mode", async ({ page }) => {
    const binderPane = page.locator(".binder-pane");
    await expect(binderPane).toBeVisible();

    await page.keyboard.press("Control+Shift+F");

    await expect(binderPane).toHaveCSS("opacity", "0");
    await expect(binderPane).toHaveCSS("pointer-events", "none");
  });

  test("inspector pane is hidden in distraction-free mode", async ({
    page,
  }) => {
    const inspectorPane = page.locator(".inspector-pane");
    await expect(inspectorPane).toBeVisible();

    await page.keyboard.press("Control+Shift+F");

    await expect(inspectorPane).toHaveCSS("opacity", "0");
    await expect(inspectorPane).toHaveCSS("pointer-events", "none");
  });

  test("editor pane spans full grid in distraction-free mode", async ({
    page,
  }) => {
    await page.keyboard.press("Control+Shift+F");

    const editorPane = page.locator(".editor-pane");
    // In distraction-free mode, grid-column should span all columns (1 / -1)
    const gridColumn = await editorPane.evaluate((el) =>
      window.getComputedStyle(el).getPropertyValue("grid-column"),
    );
    // The computed value may vary by browser, but should span from 1 to end
    expect(gridColumn).toMatch(/1\s*\/\s*-1|1\s*\/\s*\d+/);
  });

  test("Escape key exits distraction-free mode", async ({ page }) => {
    const appShell = page.locator(".app-shell");

    // Enter distraction-free mode
    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).toHaveClass(/distraction-free/);

    // Press Escape to exit
    await page.keyboard.press("Escape");
    await expect(appShell).not.toHaveClass(/distraction-free/);
  });

  test("Ctrl+Shift+F toggles distraction-free mode off when already active", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    // Toggle on
    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).toHaveClass(/distraction-free/);

    // Toggle off
    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).not.toHaveClass(/distraction-free/);

    // Chrome should be visible again
    await expect(page.locator(".toolbar")).not.toHaveCSS("opacity", "0");
  });

  // ---------------------------------------------------------------------------
  // Edge hover to peek chrome in distraction-free mode
  // ---------------------------------------------------------------------------

  test("mouse at left edge reveals binder (peek-binder class)", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).toHaveClass(/distraction-free/);

    // Move mouse to left edge (x=5, y=300)
    await page.mouse.move(5, 300);

    await expect(appShell).toHaveClass(/peek-binder/);
  });

  test("mouse at right edge reveals inspector (peek-inspector class)", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).toHaveClass(/distraction-free/);

    // Move mouse to right edge
    const viewport = page.viewportSize();
    const rightEdge = (viewport?.width ?? 1280) - 5;
    await page.mouse.move(rightEdge, 300);

    await expect(appShell).toHaveClass(/peek-inspector/);
  });

  test("moving mouse away from edge hides peek chrome after delay", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    await page.keyboard.press("Control+Shift+F");
    await expect(appShell).toHaveClass(/distraction-free/);

    // Peek binder
    await page.mouse.move(5, 300);
    await expect(appShell).toHaveClass(/peek-binder/);

    // Move mouse to center
    await page.mouse.move(640, 300);

    // Wait for PEEK_HIDE_DELAY (300ms) + buffer
    await page.waitForTimeout(500);

    await expect(appShell).not.toHaveClass(/peek-binder/);
  });
});

// =============================================================================
// Focus Mode (Toolbar Toggle)
// =============================================================================

test.describe("Focus Mode", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("focus mode can be toggled via toolbar dropdown", async ({ page }) => {
    // Open the focus dropdown
    await page.getByLabel("Focus modes menu").click();

    // Find the Focus Mode menuitemcheckbox
    const focusModeItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Focus Mode" });
    await expect(focusModeItem).toBeVisible();
    await expect(focusModeItem).toHaveAttribute("aria-checked", "false");

    // Click to enable focus mode
    await focusModeItem.click();

    // Verify the store state directly (dropdown closes on click)
    const focusState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(focusState).toBe(true);
  });

  test("Ctrl+Shift+. toggles focus mode", async ({ page }) => {
    // Verify initial state via store
    const initialState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(initialState).toBe(false);

    // Toggle focus mode on
    await page.keyboard.press("Control+Shift+.");

    const afterToggle = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(afterToggle).toBe(true);

    // Toggle focus mode off
    await page.keyboard.press("Control+Shift+.");

    const afterSecondToggle = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(afterSecondToggle).toBe(false);
  });

  test("focus mode applies .editor-focus-enabled class when editor has content", async ({
    page,
  }) => {
    // Click a chapter to open the editor with content
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500); // Wait for editor to initialize

    // Enable focus mode
    await page.keyboard.press("Control+Shift+.");

    // The editor root should have the editor-focus-enabled class
    const editorRoot = page.locator(".editor-focus-enabled");
    await expect(editorRoot).toBeVisible({ timeout: 3000 });
  });
});

// =============================================================================
// Typewriter Mode
// =============================================================================

test.describe("Typewriter Mode", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("typewriter mode can be toggled via toolbar dropdown", async ({
    page,
  }) => {
    // Open the focus dropdown
    await page.getByLabel("Focus modes menu").click();

    // Find the Typewriter Mode menuitemcheckbox
    const typewriterItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Typewriter Mode" });
    await expect(typewriterItem).toBeVisible();
    await expect(typewriterItem).toHaveAttribute("aria-checked", "false");

    // Click to enable
    await typewriterItem.click();

    // Verify the store state directly (dropdown closes on click)
    const typewriterState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(typewriterState).toBe(true);
  });

  test("Ctrl+Shift+T toggles typewriter mode", async ({ page }) => {
    const initialState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(initialState).toBe(false);

    // Toggle on
    await page.keyboard.press("Control+Shift+T");

    const afterToggle = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(afterToggle).toBe(true);

    // Toggle off
    await page.keyboard.press("Control+Shift+T");

    const afterSecondToggle = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(afterSecondToggle).toBe(false);
  });

  test("distraction-free mode can be toggled via toolbar dropdown", async ({
    page,
  }) => {
    // Open the focus dropdown
    await page.getByLabel("Focus modes menu").click();

    // Find the Distraction-Free menuitemcheckbox
    const distractionFreeItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Distraction-Free" });
    await expect(distractionFreeItem).toBeVisible();
    await expect(distractionFreeItem).toHaveAttribute("aria-checked", "false");

    // Click to enable
    await distractionFreeItem.click();

    // The app-shell should have distraction-free class now
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
  });
});

// =============================================================================
// Keyboard Shortcuts
// =============================================================================

test.describe("Keyboard Shortcuts for Focus Modes", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("all three shortcuts toggle correct independent modes", async ({
    page,
  }) => {
    // Initially all modes off
    let state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(false);
    expect(state.focus).toBe(false);
    expect(state.distractionFree).toBe(false);

    // Enable typewriter
    await page.keyboard.press("Control+Shift+T");
    state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(false);
    expect(state.distractionFree).toBe(false);

    // Enable focus (typewriter should still be on)
    await page.keyboard.press("Control+Shift+.");
    state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(false);

    // Enable distraction-free (other two should still be on)
    await page.keyboard.press("Control+Shift+F");
    state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);
  });

  test("shortcuts work when distraction-free mode is active (toolbar hidden)", async ({
    page,
  }) => {
    // Enter distraction-free mode
    await page.keyboard.press("Control+Shift+F");
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);

    // Toggle typewriter while toolbar is hidden
    await page.keyboard.press("Control+Shift+T");
    const typewriterState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(typewriterState).toBe(true);

    // Toggle focus while toolbar is hidden
    await page.keyboard.press("Control+Shift+.");
    const focusState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(focusState).toBe(true);
  });
});

// =============================================================================
// Mode Composition
// =============================================================================

test.describe("Mode Composition", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("enabling all three modes simultaneously - no crashes, all classes present", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    // Enable all three
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // Verify all modes are active
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // Distraction-free class is applied
    await expect(appShell).toHaveClass(/distraction-free/);
  });

  test("distraction-free + focus mode both work together", async ({
    page,
  }) => {
    // Open a chapter first to have editor content
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Enable focus mode
    await page.keyboard.press("Control+Shift+.");
    // Enable distraction-free
    await page.keyboard.press("Control+Shift+F");

    // Both should be active
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
    const focusState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.focusMode;
    });
    expect(focusState).toBe(true);
  });

  test("disabling one mode while others are active only affects that mode", async ({
    page,
  }) => {
    // Enable all three
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // Disable only typewriter
    await page.keyboard.press("Control+Shift+T");

    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(false);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // Distraction-free class still present
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
  });

  test("disabling distraction-free via Escape does not affect other modes", async ({
    page,
  }) => {
    // Enable all three
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // Exit distraction-free via Escape
    await page.keyboard.press("Escape");

    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(false);

    // Chrome should be visible again
    await expect(page.locator(".app-shell")).not.toHaveClass(
      /distraction-free/,
    );
  });
});

// =============================================================================
// Persistence
// =============================================================================

test.describe("Mode Persistence", () => {
  test("enabling modes triggers write to ui-state.json", async ({ page }) => {
    await openMockProject(page);

    // Clear initial IPC calls from project loading
    await clearIpcCalls(page);

    // Enable typewriter mode
    await page.keyboard.press("Control+Shift+T");

    // Verify store state
    const typewriterState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return uiState.typewriterMode;
    });
    expect(typewriterState).toBe(true);

    // Wait for the debounced persist (1s debounce + buffer)
    await page.waitForTimeout(1500);

    // Check that write_text_file was called (persistence triggered)
    const writeCalls = await getIpcCallsByCommand(
      page,
      "plugin:fs|write_text_file",
    );
    expect(writeCalls.length).toBeGreaterThan(0);
  });

  test("persisted state includes all three mode values", async ({ page }) => {
    await openMockProject(page);

    await clearIpcCalls(page);

    // Enable all three modes
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // Verify all modes are active in the store
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // Wait for debounced persist
    await page.waitForTimeout(1500);

    // Verify write_text_file was called for persistence
    const writeCalls = await getIpcCallsByCommand(
      page,
      "plugin:fs|write_text_file",
    );
    expect(writeCalls.length).toBeGreaterThan(0);

    // Also verify mkdir was called for .sakya directory
    const mkdirCalls = await getIpcCallsByCommand(page, "plugin:fs|mkdir");
    expect(mkdirCalls.length).toBeGreaterThan(0);
  });

  test("pre-loaded UI state restores modes on project open", async ({
    page,
  }) => {
    // Set up mock with pre-loaded UI state in plugin:fs|read_text_file
    // readTextFile expects byte array data from invoke, so return encoded bytes
    const uiStateJson = JSON.stringify({
      theme: "dark",
      viewMode: "editor",
      typewriterMode: true,
      focusMode: true,
      distractionFreeMode: false,
      panes: {
        binderWidth: 260,
        inspectorWidth: 300,
        binderVisible: true,
        inspectorVisible: true,
      },
    });

    // Convert to array of byte values for the Tauri FS plugin
    const bytes = Array.from(new TextEncoder().encode(uiStateJson));

    await openMockProject(page, {
      "plugin:fs|read_text_file": bytes,
    });

    // Wait for restore to complete
    await page.waitForTimeout(1000);

    // Verify modes are restored
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(false);
  });

  test("pre-loaded UI state with distraction-free restores correctly", async ({
    page,
  }) => {
    const uiStateJson = JSON.stringify({
      theme: "system",
      viewMode: "editor",
      typewriterMode: false,
      focusMode: false,
      distractionFreeMode: true,
      panes: {
        binderWidth: 260,
        inspectorWidth: 300,
        binderVisible: true,
        inspectorVisible: true,
      },
    });

    // Convert to array of byte values for the Tauri FS plugin
    const bytes = Array.from(new TextEncoder().encode(uiStateJson));

    await openMockProject(page, {
      "plugin:fs|read_text_file": bytes,
    });

    // Wait for restore to complete
    await page.waitForTimeout(1000);

    // Verify distraction-free mode is active
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
  });
});

// =============================================================================
// Focus Mode Visibility (BUG-004)
// =============================================================================

test.describe("Focus Mode Visibility", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("focus mode without selection highlights first block (not blank screen)", async ({
    page,
  }) => {
    // Open a chapter to get editor content
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Enable focus mode WITHOUT clicking into the editor (no selection)
    await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      uiState.focusMode = true;
    });

    // Wait for focus mode to apply
    await page.waitForTimeout(500);

    // The editor root should have focus-enabled class
    const editorRoot = page.locator(".editor-focus-enabled");
    await expect(editorRoot).toBeVisible({ timeout: 3000 });

    // At least one child should have editor-focus-active class (not all dimmed)
    const activeElement = page.locator(".editor-focus-active");
    await expect(activeElement.first()).toBeVisible({ timeout: 3000 });
  });

  test("focus mode with selection highlights selected block at full opacity", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Click into the editor to create a selection
    const editorContent = page.locator('[contenteditable="true"]');
    await editorContent.click();
    await page.waitForTimeout(200);

    // Enable focus mode
    await page.keyboard.press("Control+Shift+.");

    // Wait for focus mode to apply
    await page.waitForTimeout(500);

    // The active element should be at full opacity
    const activeElement = page.locator(".editor-focus-active");
    await expect(activeElement.first()).toBeVisible({ timeout: 3000 });
    await expect(activeElement.first()).toHaveCSS("opacity", "1");
  });

  test("focus highlight follows cursor when navigating between paragraphs", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Click into the editor to place cursor
    const editorContent = page.locator('[contenteditable="true"]');
    await editorContent.click();
    await page.waitForTimeout(200);

    // Enable focus mode
    await page.keyboard.press("Control+Shift+.");
    await page.waitForTimeout(300);

    // Verify initial active element exists
    const activeElements = page.locator(".editor-focus-active");
    await expect(activeElements.first()).toBeVisible({ timeout: 3000 });

    // Press End to go to end of line, then Enter to create a new paragraph
    await page.keyboard.press("End");
    await page.keyboard.press("Enter");
    await page.keyboard.type("Second paragraph text here");
    await page.waitForTimeout(300);

    // The active highlight should now be on the new (second) paragraph
    const activeAfterEnter = page.locator(".editor-focus-active");
    const secondParagraphText = await activeAfterEnter.first().textContent();
    expect(secondParagraphText).toContain("Second paragraph");

    // Now press ArrowUp to move cursor back to the first paragraph
    await page.keyboard.press("ArrowUp");
    await page.waitForTimeout(300);

    // The highlight should follow the cursor to the first paragraph
    const activeAfterArrowUp = page.locator(".editor-focus-active");
    const firstParagraphText = await activeAfterArrowUp.first().textContent();
    expect(firstParagraphText).toContain("morning light");
  });

  test("editor content not blank when focus mode activated in distraction-free mode", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Click into the editor to create a selection first
    const editorContent = page.locator('[contenteditable="true"]');
    await editorContent.click();
    await page.waitForTimeout(200);

    // Enable focus mode first, then distraction-free
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    await page.waitForTimeout(500);

    // Both modes should be active
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // The focus-enabled class should exist in the DOM
    const focusEnabled = await page.locator(".editor-focus-enabled").count();
    expect(focusEnabled).toBeGreaterThan(0);

    // At least one element should have the active class
    const activeCount = await page.locator(".editor-focus-active").count();
    expect(activeCount).toBeGreaterThan(0);
  });
});

// =============================================================================
// Edge Cases
// =============================================================================

test.describe("Focus Mode Edge Cases", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("toggling modes with no document open does not crash", async ({
    page,
  }) => {
    // No chapter selected, editor area should be in empty state
    // Toggle all three modes
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // App should not crash — Binder should still be present (even if hidden)
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // Toggle them all back off
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Escape"); // Exit distraction-free

    const afterState = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(afterState.typewriter).toBe(false);
    expect(afterState.focus).toBe(false);
    expect(afterState.distractionFree).toBe(false);
  });

  test("toggling modes during active editing session does not corrupt editor state", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.waitForTimeout(500);

    // Toggle modes rapidly while editor is active
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Control+Shift+F");

    // Verify modes are set
    let state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(true);
    expect(state.focus).toBe(true);
    expect(state.distractionFree).toBe(true);

    // Toggle them back off rapidly
    await page.keyboard.press("Control+Shift+T");
    await page.keyboard.press("Control+Shift+.");
    await page.keyboard.press("Escape");

    state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(false);
    expect(state.focus).toBe(false);
    expect(state.distractionFree).toBe(false);

    // Editor should still have its content (no crash)
    await expect(page.locator(".editor-pane")).toBeVisible();
  });

  test("focus dropdown trigger highlights when any mode is active", async ({
    page,
  }) => {
    const focusButton = page.getByLabel("Focus modes menu");

    // Initially not active
    await expect(focusButton).not.toHaveClass(/active/);

    // Enable typewriter via shortcut
    await page.keyboard.press("Control+Shift+T");

    // Focus trigger should show as active
    await expect(focusButton).toHaveClass(/active/);

    // Disable typewriter
    await page.keyboard.press("Control+Shift+T");
    await expect(focusButton).not.toHaveClass(/active/);
  });

  test("Escape does not exit distraction-free when sprint is active", async ({
    page,
  }) => {
    // Enter distraction-free mode
    await page.keyboard.press("Control+Shift+F");
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);

    // Start a sprint by setting sprint store state
    await page.evaluate(async () => {
      const { sprintStore } = await import("/src/lib/stores/index.ts");
      sprintStore.start(500); // 500-word sprint
    });

    // Escape should show sprint stop confirmation, NOT exit distraction-free
    await page.keyboard.press("Escape");

    // Distraction-free should still be active
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
  });

  test("multiple rapid toggles of distraction-free mode do not break layout", async ({
    page,
  }) => {
    const appShell = page.locator(".app-shell");

    // Rapidly toggle distraction-free mode
    await page.keyboard.press("Control+Shift+F");
    await page.keyboard.press("Control+Shift+F");
    await page.keyboard.press("Control+Shift+F");
    await page.keyboard.press("Control+Shift+F");

    // Should be back to normal (even number of toggles)
    await expect(appShell).not.toHaveClass(/distraction-free/);

    // Chrome should be visible
    await expect(page.locator(".toolbar")).toBeVisible();
    await expect(page.locator(".binder-pane")).toBeVisible();
  });
});

// =============================================================================
// Toolbar Focus Dropdown Interaction
// =============================================================================

test.describe("Focus Dropdown Interaction", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("focus dropdown opens and shows all three mode items", async ({
    page,
  }) => {
    await page.getByLabel("Focus modes menu").click();

    const dropdown = page.locator(".focus-dropdown");
    await expect(dropdown).toBeVisible();

    const items = page.locator(".focus-dropdown-item");
    await expect(items).toHaveCount(3);

    // Verify labels
    await expect(items.nth(0)).toContainText("Typewriter Mode");
    await expect(items.nth(1)).toContainText("Focus Mode");
    await expect(items.nth(2)).toContainText("Distraction-Free");
  });

  test("focus dropdown shows keyboard shortcut hints", async ({ page }) => {
    await page.getByLabel("Focus modes menu").click();

    const shortcuts = page.locator(".focus-dropdown-shortcut");
    await expect(shortcuts.nth(0)).toHaveText("Ctrl+Shift+T");
    await expect(shortcuts.nth(1)).toHaveText("Ctrl+Shift+.");
    await expect(shortcuts.nth(2)).toHaveText("Ctrl+Shift+F");
  });

  test("Escape key closes focus dropdown without toggling modes", async ({
    page,
  }) => {
    // Open dropdown
    await page.getByLabel("Focus modes menu").click();
    const dropdown = page.locator(".focus-dropdown");
    await expect(dropdown).toBeVisible();

    // Focus the dropdown element so Escape is handled by its keydown handler
    await dropdown.focus();

    // Press Escape
    await page.keyboard.press("Escape");

    // Dropdown should close
    await expect(dropdown).not.toBeVisible();

    // No modes should have been toggled
    const state = await page.evaluate(async () => {
      const { uiState } = await import("/src/lib/stores/index.ts");
      return {
        typewriter: uiState.typewriterMode,
        focus: uiState.focusMode,
        distractionFree: uiState.distractionFreeMode,
      };
    });
    expect(state.typewriter).toBe(false);
    expect(state.focus).toBe(false);
    expect(state.distractionFree).toBe(false);
  });

  test("clicking outside closes focus dropdown", async ({ page }) => {
    await page.getByLabel("Focus modes menu").click();
    const dropdown = page.locator(".focus-dropdown");
    await expect(dropdown).toBeVisible();

    // Click on the editor area (outside dropdown)
    await page.locator(".editor-pane").click();

    await expect(dropdown).not.toBeVisible();
  });

  test("focus dropdown reflects externally toggled modes", async ({
    page,
  }) => {
    // Toggle typewriter via keyboard shortcut
    await page.keyboard.press("Control+Shift+T");

    // Open dropdown and verify it reflects the change
    await page.getByLabel("Focus modes menu").click();

    const typewriterItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Typewriter Mode" });
    await expect(typewriterItem).toHaveAttribute("aria-checked", "true");

    // Other modes should still be off
    const focusItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Focus Mode" });
    await expect(focusItem).toHaveAttribute("aria-checked", "false");

    const dfItem = page
      .locator('[role="menuitemcheckbox"]')
      .filter({ hasText: "Distraction-Free" });
    await expect(dfItem).toHaveAttribute("aria-checked", "false");
  });
});
