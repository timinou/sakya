import { test, expect } from "@playwright/test";
import {
  openMockProject,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Context Menus", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Chapter context menu
  // ---------------------------------------------------------------------------

  test("right-click chapter shows context menu with expected items", async ({
    page,
  }) => {
    await page.getByTitle("1. The Awakening").click({ button: "right" });

    // Context menu should appear
    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    // Should contain expected menu items
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Rename" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Move Up" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Move Down" }),
    ).toBeVisible();

    // Status items
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Status: Draft" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Status: Revised" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Status: Final" }),
    ).toBeVisible();
  });

  test("right-click entity shows context menu with Rename and Delete", async ({
    page,
  }) => {
    await page.getByTitle("Elena Blackwood").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await expect(
      page.getByRole("menuitem").filter({ hasText: "Rename" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete" }),
    ).toBeVisible();

    // Should NOT have chapter-specific items
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Move Up" }),
    ).not.toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Move Down" }),
    ).not.toBeVisible();
  });

  test("right-click note shows context menu with Rename and Delete", async ({
    page,
  }) => {
    await page.getByTitle("Magic System Rules").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await expect(
      page.getByRole("menuitem").filter({ hasText: "Rename" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete" }),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Dismiss behavior
  // ---------------------------------------------------------------------------

  test("pressing Escape closes the context menu", async ({ page }) => {
    await page.getByTitle("1. The Awakening").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await page.keyboard.press("Escape");

    await expect(menu).not.toBeVisible();
  });

  test("clicking outside closes the context menu", async ({ page }) => {
    await page.getByTitle("1. The Awakening").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    // Click on the editor area (outside the menu)
    await page.locator(".editor-area").click();

    await expect(menu).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Context menu on different items
  // ---------------------------------------------------------------------------

  test("right-click on place entity shows context menu", async ({ page }) => {
    await page.getByTitle("Ironhaven").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await expect(
      page.getByRole("menuitem").filter({ hasText: "Rename" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete" }),
    ).toBeVisible();
  });

  test("right-click on second chapter also shows context menu", async ({
    page,
  }) => {
    await page.getByTitle("2. Into the Woods").click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await expect(
      page.getByRole("menuitem").filter({ hasText: "Rename" }),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Only one context menu at a time
  // ---------------------------------------------------------------------------

  test("opening a new context menu on the same section closes the previous one", async ({
    page,
  }) => {
    // Open context menu on first chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Close it first
    await page.keyboard.press("Escape");
    await expect(page.locator('[role="menu"]')).not.toBeVisible();

    // Open context menu on second chapter (same ManuscriptSection)
    await page.getByTitle("2. Into the Woods").click({ button: "right" });

    // Only one menu should be visible
    const menus = page.locator('[role="menu"]');
    await expect(menus).toHaveCount(1);
  });
});
