import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
  MOCK_SEARCH_RESULTS,
} from "./utils/tauri-mocks";

test.describe("Search and Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Search palette: Open / close
  // ---------------------------------------------------------------------------

  test("Cmd+K opens search palette", async ({ page }) => {
    // Search palette should not be visible initially
    await expect(
      page.getByRole("dialog", { name: "Search project" }),
    ).not.toBeVisible();

    // Press Cmd+K (or Ctrl+K for non-Mac)
    await page.keyboard.press("Control+k");

    // Search palette dialog should appear
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog).toBeVisible();

    // Search input should be focused
    const searchInput = dialog.getByLabel("Search query");
    await expect(searchInput).toBeFocused();
  });

  test("Escape closes search palette", async ({ page }) => {
    // Open search palette
    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog).toBeVisible();

    // Press Escape
    await page.keyboard.press("Escape");

    // Dialog should be gone
    await expect(dialog).not.toBeVisible();
  });

  test("clicking backdrop closes search palette", async ({ page }) => {
    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog).toBeVisible();

    // Click the backdrop (outside the modal content)
    await page.locator(".search-backdrop").click({ position: { x: 10, y: 10 } });

    await expect(dialog).not.toBeVisible();
  });

  test("Cmd+K toggles search palette", async ({ page }) => {
    // Open
    await page.keyboard.press("Control+k");
    await expect(
      page.getByRole("dialog", { name: "Search project" }),
    ).toBeVisible();

    // Toggle close
    await page.keyboard.press("Control+k");
    await expect(
      page.getByRole("dialog", { name: "Search project" }),
    ).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Search palette: Search and results
  // ---------------------------------------------------------------------------

  test("typing in search field triggers search and displays results", async ({
    page,
  }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    const searchInput = dialog.getByLabel("Search query");

    await clearIpcCalls(page);

    // Type a search query
    await searchInput.fill("Elena");

    // Wait for debounced search (300ms + network mock)
    await page.waitForTimeout(500);

    // search_project should have been called
    const searchCalls = await getIpcCallsByCommand(page, "search_project");
    expect(searchCalls.length).toBeGreaterThanOrEqual(1);
    const searchArgs = searchCalls[0].args as Record<string, unknown>;
    expect(searchArgs.query).toBe("Elena");

    // Results should appear grouped by type
    await expect(dialog.getByText("Chapters")).toBeVisible();
    await expect(dialog.getByText("Entities")).toBeVisible();
    await expect(dialog.getByText("Notes")).toBeVisible();
  });

  test("search results show titles and matching lines", async ({ page }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("test");

    // Wait for results
    await page.waitForTimeout(500);

    // Check that result titles appear
    for (const result of MOCK_SEARCH_RESULTS) {
      await expect(
        dialog.locator(".result-title", { hasText: result.title }),
      ).toBeVisible();
    }
  });

  test("search results show entity type badges for entity results", async ({
    page,
  }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("Elena");
    await page.waitForTimeout(500);

    // The entity result should have an entity type indicator
    const entityResult = MOCK_SEARCH_RESULTS.find(
      (r) => r.fileType === "entity",
    );
    if (entityResult?.entityType) {
      await expect(
        dialog.locator(".result-entity-type", {
          hasText: entityResult.entityType,
        }),
      ).toBeVisible();
    }
  });

  test("search shows 'no results' message for empty results", async ({
    page,
  }) => {
    await page.goto("about:blank");
    await openMockProject(page, {
      search_project: [],
    });

    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("nonexistent");

    await page.waitForTimeout(500);

    await expect(dialog.getByText(/No results for/)).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Search palette: Keyboard navigation
  // ---------------------------------------------------------------------------

  test("arrow keys navigate search results", async ({ page }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("test");
    await page.waitForTimeout(500);

    // First result should be selected by default
    const firstResult = dialog.locator(".result-item").first();
    await expect(firstResult).toHaveAttribute("data-selected", "true");

    // Press down arrow to select second result
    await page.keyboard.press("ArrowDown");
    const secondResult = dialog.locator(".result-item").nth(1);
    await expect(secondResult).toHaveAttribute("data-selected", "true");
    await expect(firstResult).toHaveAttribute("data-selected", "false");

    // Press up arrow to go back to first
    await page.keyboard.press("ArrowUp");
    await expect(firstResult).toHaveAttribute("data-selected", "true");
  });

  test("search footer shows keyboard hints when results present", async ({
    page,
  }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("test");
    await page.waitForTimeout(500);

    // Footer hints should be visible
    await expect(dialog.getByText("navigate")).toBeVisible();
    await expect(dialog.getByText("open")).toBeVisible();
    await expect(dialog.getByText("close")).toBeVisible();
  });

  test("Enter on selected result closes palette (selects result)", async ({
    page,
  }) => {
    await page.keyboard.press("Control+k");

    const dialog = page.getByRole("dialog", { name: "Search project" });
    await dialog.getByLabel("Search query").fill("Elena");
    await page.waitForTimeout(500);

    // Press Enter to select the first result
    await page.keyboard.press("Enter");

    // Dialog should close
    await expect(dialog).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Search palette: ESC shortcut badge
  // ---------------------------------------------------------------------------

  test("search palette shows ESC shortcut badge", async ({ page }) => {
    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog.locator(".search-shortcut")).toHaveText("ESC");
  });

  // ---------------------------------------------------------------------------
  // Keyboard shortcuts: Binder toggle
  // ---------------------------------------------------------------------------

  test("Cmd+\\ toggles binder visibility", async ({ page }) => {
    // Binder should be visible initially
    const binder = page.locator(".binder-pane");
    await expect(binder).toBeVisible();

    // Toggle binder off
    await page.keyboard.press("Control+\\");
    await expect(binder).not.toBeVisible();

    // Toggle binder back on
    await page.keyboard.press("Control+\\");
    await expect(binder).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Keyboard shortcuts: Inspector toggle
  // ---------------------------------------------------------------------------

  test("Cmd+Shift+\\ toggles inspector visibility", async ({ page }) => {
    // Inspector should be visible initially
    const inspector = page.locator(".inspector-pane");
    await expect(inspector).toBeVisible();

    // Toggle inspector off
    await page.keyboard.press("Control+Shift+\\");
    await expect(inspector).not.toBeVisible();

    // Toggle inspector back on
    await page.keyboard.press("Control+Shift+\\");
    await expect(inspector).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Toolbar: Toggle buttons for binder and inspector
  // ---------------------------------------------------------------------------

  test("toolbar toggle buttons control binder and inspector", async ({
    page,
  }) => {
    const binderToggle = page.getByLabel("Toggle binder panel");
    const inspectorToggle = page.getByLabel("Toggle inspector panel");

    // Both should be active initially
    await expect(binderToggle).toHaveAttribute("aria-pressed", "true");
    await expect(inspectorToggle).toHaveAttribute("aria-pressed", "true");

    // Toggle binder off via button
    await binderToggle.click();
    await expect(binderToggle).toHaveAttribute("aria-pressed", "false");
    await expect(page.locator(".binder-pane")).not.toBeVisible();

    // Toggle back on
    await binderToggle.click();
    await expect(binderToggle).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".binder-pane")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Toolbar: Theme cycling
  // ---------------------------------------------------------------------------

  test("toolbar theme button cycles through themes", async ({ page }) => {
    // Find the theme button - initially "system"
    const themeBtn = page.getByLabel(/cycle theme/i);
    await expect(themeBtn).toBeVisible();

    // Click to cycle: system -> light
    await themeBtn.click();
    await expect(themeBtn).toHaveAttribute("aria-label", /light/i);

    // Click to cycle: light -> dark
    await themeBtn.click();
    await expect(themeBtn).toHaveAttribute("aria-label", /dark/i);

    // Click to cycle: dark -> system
    await themeBtn.click();
    await expect(themeBtn).toHaveAttribute("aria-label", /system/i);
  });

  // ---------------------------------------------------------------------------
  // Keyboard shortcuts: Close tab (Cmd+W)
  // ---------------------------------------------------------------------------

  test("Cmd+W closes the active editor tab", async ({ page }) => {
    // Open a chapter to create a tab
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Cmd+W should close the tab
    await page.keyboard.press("Control+w");

    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).not.toBeVisible();
    await expect(
      page.locator(".welcome-card"),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // App layout: Three-pane structure
  // ---------------------------------------------------------------------------

  test("app shell renders three-pane layout with binder, editor, inspector", async ({
    page,
  }) => {
    // Binder pane
    const binderPane = page.locator(".binder-pane");
    await expect(binderPane).toBeVisible();
    await expect(binderPane.getByText("Binder")).toBeVisible();

    // Editor pane
    const editorPane = page.locator(".editor-pane");
    await expect(editorPane).toBeVisible();

    // Inspector pane
    const inspectorPane = page.locator(".inspector-pane");
    await expect(inspectorPane).toBeVisible();
    await expect(inspectorPane.getByText("Inspector")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // All binder sections rendered together
  // ---------------------------------------------------------------------------

  test("binder shows all sections: Manuscript, entity types, Notes", async ({
    page,
  }) => {
    const binder = page.locator(".binder-pane");

    await expect(binder.getByText("Manuscript")).toBeVisible();
    await expect(binder.getByText("Characters")).toBeVisible();
    await expect(binder.getByText("Places")).toBeVisible();
    await expect(binder.getByText("Notes")).toBeVisible();
  });
});
