import { test, expect, type Page } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

// =============================================================================
// Stats Visual Regression Tests (ITEM-216)
// =============================================================================

/** Open the writing stats tab via store. */
async function openStatsTab(page: Page): Promise<void> {
  await page.evaluate(async () => {
    const { editorState } = await import("/src/lib/stores/index.ts");
    editorState.openDocument({
      id: "stats:writing",
      title: "Writing Stats",
      documentType: "stats",
      documentSlug: "writing",
      isDirty: false,
    });
  });
  await page.getByRole("tab", { name: "Writing Stats" }).waitFor();
  await page.locator(".writing-stats").waitFor({ state: "visible" });
  // Wait for session data to load and render
  await page.waitForTimeout(500);
}

test.describe("Stats visual regression", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("writing stats full view", async ({ page }) => {
    await openStatsTab(page);

    const writingStats = page.locator(".writing-stats");
    await expect(writingStats).toBeVisible();
    await expect(writingStats).toHaveScreenshot("writing-stats-full.png");
  });

  test("calendar heatmap", async ({ page }) => {
    await openStatsTab(page);

    const heatmap = page.locator(".calendar-heatmap");
    await expect(heatmap).toBeVisible();
    await expect(heatmap).toHaveScreenshot("calendar-heatmap.png");
  });
});
