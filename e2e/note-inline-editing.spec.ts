import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Corkboard inline note editing", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
    // Switch to corkboard view
    await page.getByRole("button", { name: "Corkboard" }).click();
    await expect(page.locator(".corkboard")).toBeVisible({ timeout: 3000 });
  });

  test("double-click card shows textarea with loaded content", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    // Should see a textarea with note body content
    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    // get_note IPC should have been called
    const noteCalls = await getIpcCallsByCommand(page, "get_note");
    expect(noteCalls.length).toBeGreaterThanOrEqual(1);

    // Textarea should contain actual content (loaded from mock)
    const value = await textarea.inputValue();
    expect(value.length).toBeGreaterThan(0);
  });

  test("single-click card selects without entering edit mode", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.click();

    // Textarea should NOT appear
    const textarea = card.locator("textarea");
    await expect(textarea).not.toBeVisible({ timeout: 1000 });
  });

  test("typing in textarea triggers debounced save_note IPC", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    await clearIpcCalls(page);
    await textarea.fill("Updated content here");

    // Wait for debounce (1.5s in implementation)
    await page.waitForTimeout(2000);

    const saveCalls = await getIpcCallsByCommand(page, "save_note");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);
  });

  test("Escape exits edit mode and saves", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    await clearIpcCalls(page);
    await textarea.fill("Changed text");
    await textarea.press("Escape");

    // Textarea should disappear
    await expect(textarea).not.toBeVisible({ timeout: 2000 });

    // Wait for async save to complete
    await page.waitForTimeout(500);

    // save_note should have been called
    const saveCalls = await getIpcCallsByCommand(page, "save_note");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);
  });

  test("click on corkboard background exits edit mode", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    // Click on the corkboard background (not on a card)
    await page.locator(".corkboard").click({ position: { x: 5, y: 5 } });

    // Textarea should disappear
    await expect(textarea).not.toBeVisible({ timeout: 2000 });
  });

  test("get_note IPC called when entering edit mode", async ({ page }) => {
    await clearIpcCalls(page);

    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    const noteCalls = await getIpcCallsByCommand(page, "get_note");
    expect(noteCalls.length).toBeGreaterThanOrEqual(1);
  });

  test("'Open in Editor' button opens a tab", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    // Click "Open in Editor" button
    const openBtn = card.getByRole("button", { name: /open in editor/i });
    await expect(openBtn).toBeVisible();
    await openBtn.click();

    // Should switch to editor view mode with the note tab
    // The note should open as a tab
    const noteTab = page.locator(".tab").first();
    await expect(noteTab).toBeVisible({ timeout: 3000 });
  });

  test("Ctrl+Enter opens note in tab", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    await textarea.press("Control+Enter");

    // Should open a tab for the note
    const noteTab = page.locator(".tab").first();
    await expect(noteTab).toBeVisible({ timeout: 3000 });
  });

  test("card z-index elevated during edit", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    // Card should have editing class which sets z-index
    await expect(card).toHaveClass(/editing/);
  });

  test("Enter key inserts newline (does not exit edit mode)", async ({ page }) => {
    const card = page.locator(".note-card").first();
    await card.dblclick();

    const textarea = card.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 3000 });

    await textarea.fill("Line 1");
    await textarea.press("Enter");
    await textarea.type("Line 2");

    // Textarea should still be visible (Enter doesn't exit)
    await expect(textarea).toBeVisible();

    const value = await textarea.inputValue();
    expect(value).toContain("Line 2");
  });
});
