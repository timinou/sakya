import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCalls,
  getIpcCallsByCommand,
  clearIpcCalls,
  setupDefaultTauriMocks,
  MOCK_CHAPTERS,
  MOCK_MANUSCRIPT_CONFIG,
} from "./utils/tauri-mocks";

test.describe("Writing Workflow - Manuscript", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Binder: Manuscript section renders chapters
  // ---------------------------------------------------------------------------

  test("manuscript section shows in binder with correct chapter count", async ({
    page,
  }) => {
    const manuscriptSection = page.locator(".section").filter({
      hasText: "Manuscript",
    });
    await expect(manuscriptSection).toBeVisible();

    // Chapter count badge should show 3
    await expect(manuscriptSection.locator(".section-count")).toHaveText("3");
  });

  test("chapters are listed in order with numbered titles", async ({
    page,
  }) => {
    // Chapters show as "N. Title" in the binder
    await expect(page.getByTitle("1. The Awakening")).toBeVisible();
    await expect(page.getByTitle("2. Into the Woods")).toBeVisible();
    await expect(page.getByTitle("3. The Siege")).toBeVisible();
  });

  test("get_manuscript_config and get_chapter are called on initialization", async ({
    page,
  }) => {
    const configCalls = await getIpcCallsByCommand(
      page,
      "get_manuscript_config",
    );
    expect(configCalls.length).toBeGreaterThanOrEqual(1);

    // get_chapter should be called for each chapter slug in the config
    const chapterCalls = await getIpcCallsByCommand(page, "get_chapter");
    expect(chapterCalls.length).toBeGreaterThanOrEqual(
      MOCK_MANUSCRIPT_CONFIG.chapters.length,
    );

    const slugsRequested = chapterCalls.map(
      (c) => (c.args as Record<string, unknown>)?.slug,
    );
    for (const slug of MOCK_MANUSCRIPT_CONFIG.chapters) {
      expect(slugsRequested).toContain(slug);
    }
  });

  // ---------------------------------------------------------------------------
  // Binder: Click chapter -> loads in editor area
  // ---------------------------------------------------------------------------

  test("clicking a chapter opens a tab in the editor area", async ({
    page,
  }) => {
    // Initially the editor should show the empty state
    await expect(
      page.locator(".welcome-card"),
    ).toBeVisible();

    // Click the first chapter
    await page.getByTitle("1. The Awakening").click();

    // A tab should appear in the editor tabs area
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(tabList).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "The Awakening" }),
    ).toBeVisible();

    // The empty state message should no longer be visible
    await expect(
      page.locator(".welcome-card"),
    ).not.toBeVisible();
  });

  test("clicking chapter calls get_chapter for content loading", async ({
    page,
  }) => {
    await clearIpcCalls(page);
    await page.getByTitle("1. The Awakening").click();

    // Wait for the tab to appear
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Wait for async content load
    await page.waitForTimeout(500);

    const chapterCalls = await getIpcCallsByCommand(page, "get_chapter");
    const awakening = chapterCalls.find(
      (c) => (c.args as Record<string, unknown>)?.slug === "the-awakening",
    );
    expect(awakening).toBeDefined();
  });

  // ---------------------------------------------------------------------------
  // Tabs: Multiple chapters
  // ---------------------------------------------------------------------------

  test("opening multiple chapters creates multiple tabs", async ({ page }) => {
    // Click first chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Click second chapter
    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    // Both tabs should exist
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(
      tabList.getByRole("tab", { name: "The Awakening" }),
    ).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "Into the Woods" }),
    ).toBeVisible();

    // The second tab should be active (most recently clicked)
    const activeTab = tabList.getByRole("tab", { name: "Into the Woods" });
    await expect(activeTab).toHaveAttribute("aria-selected", "true");
  });

  test("clicking an already-open tab switches to it without re-fetching", async ({
    page,
  }) => {
    // Open two chapters
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    // Clear IPC to track new calls
    await clearIpcCalls(page);

    // Click the Awakening tab directly (tab switching, not binder)
    await page.getByRole("tab", { name: "The Awakening" }).click();
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).toHaveAttribute("aria-selected", "true");

    // Should not have re-fetched (content was cached)
    const newChapterCalls = await getIpcCallsByCommand(page, "get_chapter");
    expect(newChapterCalls.length).toBe(0);
  });

  test("clicking the same chapter in binder again does not duplicate tabs", async ({
    page,
  }) => {
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Click the same chapter again
    await page.getByTitle("1. The Awakening").click();

    // Should still only have one tab
    const tabs = page.getByRole("tablist", { name: "Open documents" }).getByRole("tab");
    await expect(tabs).toHaveCount(1);
  });

  // ---------------------------------------------------------------------------
  // Tabs: Close tab
  // ---------------------------------------------------------------------------

  test("closing a tab removes it and shows empty state if last", async ({
    page,
  }) => {
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Close the active tab via Ctrl+W keyboard shortcut
    await page.keyboard.press("Control+w");

    // Tab should be gone
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).not.toBeVisible();

    // Empty state returns
    await expect(
      page.locator(".welcome-card"),
    ).toBeVisible();
  });

  test("closing one tab when multiple are open switches to remaining tab", async ({
    page,
  }) => {
    // Open two tabs
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    // Close the active tab (Into the Woods) via Ctrl+W
    await page.keyboard.press("Control+w");

    // The Awakening tab should now be the only tab and be active
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).toBeVisible();

    // Verify it becomes the active tab
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).toHaveAttribute("aria-selected", "true");
  });

  // ---------------------------------------------------------------------------
  // Binder: Chapter selection highlighting
  // ---------------------------------------------------------------------------

  test("selected chapter in binder is highlighted", async ({ page }) => {
    const chapterItem = page.getByTitle("1. The Awakening");
    await chapterItem.click();

    // The item button should have the 'selected' or 'active' class
    await expect(chapterItem).toHaveClass(/selected|active/);
  });

  // ---------------------------------------------------------------------------
  // Binder: Create chapter via "+" button
  // ---------------------------------------------------------------------------

  test("clicking add button on manuscript section shows inline input", async ({
    page,
  }) => {
    // Hover over Manuscript section to reveal the add button
    const manuscriptHeader = page
      .locator(".section-header")
      .filter({ hasText: "Manuscript" });
    await manuscriptHeader.hover();

    // The add button has aria-label "Add Manuscript"
    const addButton = page.getByLabel("Add Manuscript");
    await addButton.click();

    // An inline input for the chapter title should appear
    const input = page.getByPlaceholder("Chapter title...");
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("typing title and pressing Enter creates a new chapter", async ({
    page,
  }) => {
    // Open the inline input
    const manuscriptHeader = page
      .locator(".section-header")
      .filter({ hasText: "Manuscript" });
    await manuscriptHeader.hover();
    await page.getByLabel("Add Manuscript").click();

    const input = page.getByPlaceholder("Chapter title...");
    await expect(input).toBeVisible();

    await clearIpcCalls(page);

    // Type a chapter title and press Enter
    await input.fill("The Betrayal");
    await input.press("Enter");

    // Wait for the async create to complete
    await page.waitForTimeout(500);

    // create_chapter should have been called
    const createCalls = await getIpcCallsByCommand(page, "create_chapter");
    expect(createCalls.length).toBeGreaterThanOrEqual(1);
    const createArgs = createCalls[0].args as Record<string, unknown>;
    expect(createArgs.title).toBe("The Betrayal");

    // The inline input should be dismissed
    await expect(input).not.toBeVisible();
  });

  test("pressing Escape cancels chapter creation", async ({ page }) => {
    const manuscriptHeader = page
      .locator(".section-header")
      .filter({ hasText: "Manuscript" });
    await manuscriptHeader.hover();
    await page.getByLabel("Add Manuscript").click();

    const input = page.getByPlaceholder("Chapter title...");
    await expect(input).toBeVisible();

    await clearIpcCalls(page);

    // Press Escape
    await input.press("Escape");

    // Input should be gone
    await expect(input).not.toBeVisible();

    // No create_chapter call should have been made
    const createCalls = await getIpcCallsByCommand(page, "create_chapter");
    expect(createCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Binder: Collapse / expand manuscript section
  // ---------------------------------------------------------------------------

  test("collapsing manuscript section hides chapters", async ({ page }) => {
    const manuscriptHeader = page
      .locator(".section-header")
      .filter({ hasText: "Manuscript" });

    // Chapters visible
    await expect(page.getByTitle("1. The Awakening")).toBeVisible();

    // Collapse
    await manuscriptHeader.click();
    await expect(page.getByTitle("1. The Awakening")).not.toBeVisible();

    // Expand
    await manuscriptHeader.click();
    await expect(page.getByTitle("1. The Awakening")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Editor area: Empty state
  // ---------------------------------------------------------------------------

  test("editor area shows empty state before any chapter is selected", async ({
    page,
  }) => {
    await expect(
      page.locator(".welcome-card"),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Status bar integration
  // ---------------------------------------------------------------------------

  test("status bar shows word count and save status", async ({ page }) => {
    // Status bar should be visible
    await expect(page.getByText("0 words")).toBeVisible();
    await expect(page.getByText("Saved")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Toolbar: Project name
  // ---------------------------------------------------------------------------

  test("toolbar shows project name after opening", async ({ page }) => {
    await expect(page.getByText("Opened Project")).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Standalone test: empty manuscript state
// ---------------------------------------------------------------------------

test("shows placeholder when manuscript has no chapters", async ({ page }) => {
  // Use default mocks (which include chapters), then override and reset
  await openMockProject(page);

  // Verify chapters are loaded first
  await expect(page.getByTitle("1. The Awakening")).toBeVisible();

  // Override get_manuscript_config mock to return empty, then reset store
  await page.evaluate(async () => {
    // Patch the mock so reloading returns empty chapters
    const internals = (window as any).__TAURI_INTERNALS__;
    const origInvoke = internals.invoke.bind(internals);
    internals.invoke = function (
      cmd: string,
      args?: Record<string, unknown>,
      options?: unknown,
    ) {
      if (cmd === "get_manuscript_config") {
        return Promise.resolve({ chapters: [] });
      }
      return origInvoke(cmd, args, options);
    };

    // Reset the manuscript store (triggers effect which reloads empty data)
    const stores = await import("/src/lib/stores/index.ts");
    stores.manuscriptStore.reset();
  });
  await page.waitForTimeout(500);

  // The "Add first chapter" CTA should appear in the manuscript section
  await expect(page.locator(".placeholder-cta").filter({ hasText: /Add first chapter/i })).toBeVisible();
});
