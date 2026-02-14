import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Chapter Reordering via Context Menu", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Move Up / Move Down disabled states
  // ---------------------------------------------------------------------------

  test("first chapter has Move Up disabled", async ({ page }) => {
    // Right-click on first chapter (index 0)
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Move Up should be disabled for the first chapter
    const moveUp = page
      .getByRole("menuitem")
      .filter({ hasText: "Move Up" });
    await expect(moveUp).toBeVisible();
    await expect(moveUp).toHaveAttribute("aria-disabled", "true");
  });

  test("last chapter has Move Down disabled", async ({ page }) => {
    // Right-click on last chapter (index 2)
    await page.getByTitle("3. The Siege").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Move Down should be disabled for the last chapter
    const moveDown = page
      .getByRole("menuitem")
      .filter({ hasText: "Move Down" });
    await expect(moveDown).toBeVisible();
    await expect(moveDown).toHaveAttribute("aria-disabled", "true");
  });

  test("middle chapter has both Move Up and Move Down enabled", async ({
    page,
  }) => {
    // Right-click on middle chapter (index 1)
    await page.getByTitle("2. Into the Woods").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    const moveUp = page
      .getByRole("menuitem")
      .filter({ hasText: "Move Up" });
    const moveDown = page
      .getByRole("menuitem")
      .filter({ hasText: "Move Down" });

    await expect(moveUp).toHaveAttribute("aria-disabled", "false");
    await expect(moveDown).toHaveAttribute("aria-disabled", "false");
  });

  // ---------------------------------------------------------------------------
  // Move Down calls reorder_chapters IPC
  // ---------------------------------------------------------------------------

  test("Move Down on middle chapter calls reorder_chapters with new order", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on middle chapter
    await page.getByTitle("2. Into the Woods").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Move Down
    await page
      .getByRole("menuitem")
      .filter({ hasText: "Move Down" })
      .click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // reorder_chapters IPC should have been called
    const reorderCalls = await getIpcCallsByCommand(
      page,
      "reorder_chapters",
    );
    expect(reorderCalls.length).toBeGreaterThanOrEqual(1);

    // The new order should have into-the-woods and the-siege swapped
    const args = reorderCalls[0].args as Record<string, unknown>;
    const slugs = args.chapterSlugs as string[];
    expect(slugs).toEqual([
      "the-awakening",
      "the-siege",
      "into-the-woods",
    ]);
  });

  // ---------------------------------------------------------------------------
  // Move Up calls reorder_chapters IPC
  // ---------------------------------------------------------------------------

  test("Move Up on middle chapter calls reorder_chapters with new order", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on middle chapter
    await page.getByTitle("2. Into the Woods").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Move Up
    await page
      .getByRole("menuitem")
      .filter({ hasText: "Move Up" })
      .click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // reorder_chapters IPC should have been called
    const reorderCalls = await getIpcCallsByCommand(
      page,
      "reorder_chapters",
    );
    expect(reorderCalls.length).toBeGreaterThanOrEqual(1);

    // The new order should have the-awakening and into-the-woods swapped
    const args = reorderCalls[0].args as Record<string, unknown>;
    const slugs = args.chapterSlugs as string[];
    expect(slugs).toEqual([
      "into-the-woods",
      "the-awakening",
      "the-siege",
    ]);
  });

  // ---------------------------------------------------------------------------
  // Move Down on last chapter is a no-op (disabled)
  // ---------------------------------------------------------------------------

  test("clicking disabled Move Down does not call reorder_chapters", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on last chapter
    await page.getByTitle("3. The Siege").click({ button: "right" });

    // Force click the disabled Move Down (Playwright blocks clicks on disabled elements)
    await page
      .getByRole("menuitem")
      .filter({ hasText: "Move Down" })
      .click({ force: true });

    await page.waitForTimeout(300);

    // No reorder IPC should have been called
    const reorderCalls = await getIpcCallsByCommand(
      page,
      "reorder_chapters",
    );
    expect(reorderCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // First chapter Move Up disabled - no IPC
  // ---------------------------------------------------------------------------

  test("clicking disabled Move Up does not call reorder_chapters", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on first chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });

    // Force click the disabled Move Up (Playwright blocks clicks on disabled elements)
    await page
      .getByRole("menuitem")
      .filter({ hasText: "Move Up" })
      .click({ force: true });

    await page.waitForTimeout(300);

    // No reorder IPC should have been called
    const reorderCalls = await getIpcCallsByCommand(
      page,
      "reorder_chapters",
    );
    expect(reorderCalls.length).toBe(0);
  });
});
