import { test, expect, type Page } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

// =============================================================================
// Overlays Visual Regression Tests (ITEM-216)
// =============================================================================

/** Open a chapter in the editor to enable sprint functionality. */
async function openChapter(page: Page): Promise<void> {
  await page.getByTitle("1. The Awakening").click();
  await page.getByRole("tab", { name: "The Awakening" }).waitFor();
}

/** Open the sprint panel popover from the toolbar. */
async function openSprintPanel(page: Page): Promise<void> {
  await page.getByLabel("Open sprint timer").click();
  await page
    .locator('[role="dialog"][aria-label="Sprint Timer"]')
    .waitFor({ state: "visible" });
}

test.describe("Overlays visual regression", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("search palette with results", async ({ page }) => {
    // Open search palette
    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog).toBeVisible();

    // Type a query to trigger search results (mock returns results for any query)
    const searchInput = dialog.getByLabel("Search query");
    await searchInput.fill("Elena");
    // Wait for search results to render
    await page.locator(".result-item").first().waitFor({ state: "visible" });

    const searchModal = page.locator(".search-modal");
    await expect(searchModal).toHaveScreenshot("search-palette-results.png");
  });

  test("search palette empty", async ({ page }) => {
    // Override search to return empty
    await page.goto("about:blank");
    await openMockProject(page, {
      search_project: [],
    });

    await page.keyboard.press("Control+k");
    const dialog = page.getByRole("dialog", { name: "Search project" });
    await expect(dialog).toBeVisible();

    const searchInput = dialog.getByLabel("Search query");
    await searchInput.fill("nonexistent query xyz");
    // Wait for empty state to appear
    await page.locator(".search-empty").waitFor({ state: "visible" });

    const searchModal = page.locator(".search-modal");
    await expect(searchModal).toHaveScreenshot("search-palette-empty.png");
  });

  test("sprint timer panel", async ({ page }) => {
    await openChapter(page);
    await openSprintPanel(page);

    const sprintDialog = page.locator(
      '[role="dialog"][aria-label="Sprint Timer"]',
    );
    await expect(sprintDialog).toBeVisible();
    await expect(sprintDialog).toHaveScreenshot("sprint-timer-panel.png");
  });

  test("sprint overlay bar (active)", async ({ page }) => {
    await openChapter(page);
    await openSprintPanel(page);

    // Start the sprint
    await page.getByText("Start Sprint").click();
    const sprintBar = page.locator(".sprint-bar");
    await expect(sprintBar).toBeVisible();

    await expect(sprintBar).toHaveScreenshot("sprint-bar-active.png");
  });
});
