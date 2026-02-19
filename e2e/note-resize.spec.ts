import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
  MOCK_NOTES_CONFIG,
} from "./utils/tauri-mocks";

test.describe("Corkboard card resize", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
    // Switch to corkboard view
    await page.getByRole("button", { name: "Corkboard" }).click();
    await expect(page.locator(".corkboard")).toBeVisible({ timeout: 3000 });
  });

  test("resize handle visible on card hover", async ({ page }) => {
    const card = page.locator(".note-card").first();
    // Handle should not be visible initially
    const handle = card.locator(".resize-handle");
    await expect(handle).toBeAttached();
    // Hover to reveal
    await card.hover();
    await expect(handle).toBeVisible();
  });

  test("drag resize changes card dimensions", async ({ page }) => {
    const card = page.locator(".note-card").first();
    const handle = card.locator(".resize-handle");
    await card.hover();
    await expect(handle).toBeVisible();

    // Get initial card dimensions
    const initialBox = await card.boundingBox();
    expect(initialBox).toBeTruthy();

    // Drag handle right and down to increase size
    const handleBox = await handle.boundingBox();
    expect(handleBox).toBeTruthy();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2,
      handleBox!.y + handleBox!.height / 2
    );
    await page.mouse.down();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2 + 80,
      handleBox!.y + handleBox!.height / 2 + 60
    );
    await page.mouse.up();

    // Card should be larger now
    const newBox = await card.boundingBox();
    expect(newBox).toBeTruthy();
    expect(newBox!.width).toBeGreaterThan(initialBox!.width + 50);
    expect(newBox!.height).toBeGreaterThan(initialBox!.height + 30);
  });

  test("resize persists via save_notes_config IPC", async ({ page }) => {
    const card = page.locator(".note-card").first();
    const handle = card.locator(".resize-handle");
    await card.hover();

    await clearIpcCalls(page);

    const handleBox = await handle.boundingBox();
    expect(handleBox).toBeTruthy();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2,
      handleBox!.y + handleBox!.height / 2
    );
    await page.mouse.down();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2 + 50,
      handleBox!.y + handleBox!.height / 2 + 50
    );
    await page.mouse.up();

    // Wait for debounced save
    await page.waitForTimeout(700);

    const saveCalls = await getIpcCallsByCommand(page, "save_notes_config");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);

    // The saved config should contain size data
    const lastSave = saveCalls[saveCalls.length - 1];
    const config = lastSave.args?.config as { notes: Array<{ size?: { width: number; height: number } }> };
    expect(config).toBeTruthy();
    // At least one note should have a size set
    const notesWithSize = config.notes.filter((n) => n.size);
    expect(notesWithSize.length).toBeGreaterThanOrEqual(1);
  });

  test("minimum size enforced — can't shrink below 180×120", async ({ page }) => {
    const card = page.locator(".note-card").first();
    const handle = card.locator(".resize-handle");
    await card.hover();

    const handleBox = await handle.boundingBox();
    expect(handleBox).toBeTruthy();

    // Try to drag handle far to the upper-left (shrink card)
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2,
      handleBox!.y + handleBox!.height / 2
    );
    await page.mouse.down();
    await page.mouse.move(
      handleBox!.x - 300,
      handleBox!.y - 300
    );
    await page.mouse.up();

    const box = await card.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.width).toBeGreaterThanOrEqual(178); // small tolerance for border
    expect(box!.height).toBeGreaterThanOrEqual(118);
  });

  test("resize does not trigger card drag", async ({ page }) => {
    const card = page.locator(".note-card").first();
    const handle = card.locator(".resize-handle");
    await card.hover();

    // Get initial card position
    const initialStyle = await card.getAttribute("style");
    const leftMatch = initialStyle?.match(/left:\s*([\d.]+)%/);
    const topMatch = initialStyle?.match(/top:\s*([\d.]+)%/);
    const initialLeft = leftMatch ? parseFloat(leftMatch[1]) : null;
    const initialTop = topMatch ? parseFloat(topMatch[1]) : null;
    expect(initialLeft).not.toBeNull();
    expect(initialTop).not.toBeNull();

    // Drag the resize handle
    const handleBox = await handle.boundingBox();
    expect(handleBox).toBeTruthy();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2,
      handleBox!.y + handleBox!.height / 2
    );
    await page.mouse.down();
    await page.mouse.move(
      handleBox!.x + handleBox!.width / 2 + 60,
      handleBox!.y + handleBox!.height / 2 + 40
    );
    await page.mouse.up();

    // Card position (left/top %) should NOT have changed
    const afterStyle = await card.getAttribute("style");
    const afterLeftMatch = afterStyle?.match(/left:\s*([\d.]+)%/);
    const afterTopMatch = afterStyle?.match(/top:\s*([\d.]+)%/);
    const afterLeft = afterLeftMatch ? parseFloat(afterLeftMatch[1]) : null;
    const afterTop = afterTopMatch ? parseFloat(afterTopMatch[1]) : null;
    expect(afterLeft).toBeCloseTo(initialLeft!, 1);
    expect(afterTop).toBeCloseTo(initialTop!, 1);
  });

  test("cards without size use default dimensions", async ({ page }) => {
    // All mock notes have no `size` set — should render at default dimensions
    const cards = page.locator(".note-card");
    const count = await cards.count();
    expect(count).toBeGreaterThanOrEqual(1);

    for (let i = 0; i < count; i++) {
      const box = await cards.nth(i).boundingBox();
      expect(box).toBeTruthy();
      // Default min-width is 180, max-width is 260
      expect(box!.width).toBeGreaterThanOrEqual(178);
      expect(box!.width).toBeLessThanOrEqual(265);
    }
  });
});
