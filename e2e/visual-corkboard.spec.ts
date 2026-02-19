import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";
import { setTheme } from "./utils/screenshots";

// =============================================================================
// Corkboard & NoteCard Visual Regression Tests (ITEM-214)
// =============================================================================

test.describe("Corkboard visual regression", () => {
  test("corkboard with 3 note cards", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    // Wait for note cards to render
    await expect(page.locator(".note-card")).toHaveCount(3);

    await expect(page).toHaveScreenshot("corkboard-3-notes.png");
  });

  test("corkboard dark theme", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await expect(page.locator(".note-card")).toHaveCount(3);

    await setTheme(page, "dark");
    await expect(page).toHaveScreenshot("corkboard-3-notes-dark.png");
  });

  test("corkboard empty state", async ({ page }) => {
    // Navigate to blank first to avoid addInitScript accumulation
    await page.goto("about:blank");
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });

    const corkboard = page.locator(".corkboard");
    await expect(corkboard.locator(".empty-state")).toBeVisible();
    await expect(corkboard).toHaveScreenshot("corkboard-empty.png");
  });

  test("note card with label badge", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await expect(page.locator(".note-card")).toHaveCount(3);

    // "Magic System Rules" has label "worldbuilding"
    const labelCard = page.locator(".note-card").filter({ hasText: "Magic System" });
    await expect(labelCard).toBeVisible();
    await expect(labelCard.locator(".label-badge")).toBeVisible();
    await expect(labelCard).toHaveScreenshot("notecard-with-label.png");
  });

  test("note card without label", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await expect(page.locator(".note-card")).toHaveCount(3);

    // "Character Arcs" has label: null
    const noLabelCard = page.locator(".note-card").filter({ hasText: "Character Arcs" });
    await expect(noLabelCard).toBeVisible();
    await expect(noLabelCard).toHaveScreenshot("notecard-no-label.png");
  });

  test("note card in edit mode", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await expect(page.locator(".note-card")).toHaveCount(3);

    // Double-click to enter edit mode
    const card = page.locator(".note-card").first();
    await card.dblclick();
    await expect(page.locator(".note-card.editing")).toBeVisible();
    await expect(page.locator(".edit-textarea")).toBeVisible();

    const editingCard = page.locator(".note-card.editing");
    await expect(editingCard).toHaveScreenshot("notecard-editing.png");
  });

  test("note card color picker open", async ({ page }) => {
    await openMockProject(page);
    await page.getByRole("button", { name: "Corkboard" }).click();
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await expect(page.locator(".note-card")).toHaveCount(3);

    // NOTE: Direct click on the "Change color" button doesn't work reliably due
    // to Svelte 5 event delegation + stopPropagation issue in NoteCard.
    // Use dispatchEvent to trigger the handler directly on the button element.
    const card = page.locator(".note-card").first();
    await card.hover();
    await page.waitForTimeout(200);
    await card.locator('button[title="Change color"]').dispatchEvent("click");
    await expect(card.locator(".color-picker")).toBeVisible({ timeout: 3000 });

    // Screenshot the card area including the picker
    await expect(card).toHaveScreenshot("notecard-color-picker.png");
  });

  test("split view (editor + corkboard)", async ({ page }) => {
    await openMockProject(page);
    // Open a chapter first to populate the editor pane
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    // Switch to split view
    const viewGroup = page.getByRole("group", { name: "View mode" });
    await viewGroup.getByRole("button", { name: "Split" }).click();
    // Wait for corkboard to appear alongside editor
    await page.locator(".corkboard").waitFor({ state: "visible" });
    await page.waitForTimeout(500);

    await expect(page).toHaveScreenshot("appshell-split-view.png");
  });
});
