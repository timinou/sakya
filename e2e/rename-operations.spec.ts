import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Rename Operations", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Rename chapter
  // ---------------------------------------------------------------------------

  test("rename chapter: right-click -> Rename -> inline input appears pre-filled", async ({
    page,
  }) => {
    // Right-click on first chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Rename
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    // Context menu should close
    await expect(page.locator('[role="menu"]')).not.toBeVisible();

    // Inline rename input should appear with current title pre-filled
    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();
    await expect(renameInput).toHaveValue("The Awakening");
    await expect(renameInput).toBeFocused();
  });

  test("rename chapter: type new name -> Enter -> rename_chapter IPC called", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Open rename input
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();

    // Clear and type new name
    await renameInput.fill("The New Beginning");
    await renameInput.press("Enter");

    // Wait for async operation
    await page.waitForTimeout(500);

    // rename_chapter IPC should have been called
    const renameCalls = await getIpcCallsByCommand(page, "rename_chapter");
    expect(renameCalls.length).toBeGreaterThanOrEqual(1);
    const args = renameCalls[0].args as Record<string, unknown>;
    expect(args.slug).toBe("the-awakening");
    expect(args.newTitle).toBe("The New Beginning");
  });

  test("rename chapter: Escape cancels rename, original title preserved", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Open rename input
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();

    // Type something but then Escape
    await renameInput.fill("Something Else");
    await renameInput.press("Escape");

    // Input should be gone
    await expect(renameInput).not.toBeVisible();

    // Original title should still be present
    await expect(page.getByTitle("1. The Awakening")).toBeVisible();

    // No rename_chapter IPC should have been called
    const renameCalls = await getIpcCallsByCommand(page, "rename_chapter");
    expect(renameCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Rename entity
  // ---------------------------------------------------------------------------

  test("rename entity: right-click -> Rename -> inline input -> Enter -> rename_entity IPC called", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on an entity
    await page.getByTitle("Elena Blackwood").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    // Inline rename input should appear
    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();
    await expect(renameInput).toHaveValue("Elena Blackwood");
    await expect(renameInput).toBeFocused();

    // Type new name and confirm
    await renameInput.fill("Elena Nightshade");
    await renameInput.press("Enter");

    await page.waitForTimeout(500);

    // rename_entity IPC should have been called
    const renameCalls = await getIpcCallsByCommand(page, "rename_entity");
    expect(renameCalls.length).toBeGreaterThanOrEqual(1);
    const args = renameCalls[0].args as Record<string, unknown>;
    expect(args.schemaType).toBe("character");
    expect(args.oldSlug).toBe("elena-blackwood");
    expect(args.newTitle).toBe("Elena Nightshade");
  });

  test("rename entity: Escape cancels rename", async ({ page }) => {
    await clearIpcCalls(page);

    await page.getByTitle("Elena Blackwood").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();

    await renameInput.press("Escape");
    await expect(renameInput).not.toBeVisible();

    // Original title preserved
    await expect(page.getByTitle("Elena Blackwood")).toBeVisible();

    const renameCalls = await getIpcCallsByCommand(page, "rename_entity");
    expect(renameCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Rename note
  // ---------------------------------------------------------------------------

  test("rename note: right-click -> Rename -> inline input -> Enter -> rename_note IPC called", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on a note
    await page.getByTitle("Magic System Rules").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    // Inline rename input should appear
    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();
    await expect(renameInput).toHaveValue("Magic System Rules");
    await expect(renameInput).toBeFocused();

    // Type new name and confirm
    await renameInput.fill("Arcane System Rules");
    await renameInput.press("Enter");

    await page.waitForTimeout(500);

    // rename_note IPC should have been called
    const renameCalls = await getIpcCallsByCommand(page, "rename_note");
    expect(renameCalls.length).toBeGreaterThanOrEqual(1);
    const args = renameCalls[0].args as Record<string, unknown>;
    expect(args.slug).toBe("magic-system");
    expect(args.newTitle).toBe("Arcane System Rules");
  });

  test("rename note: Escape cancels rename", async ({ page }) => {
    await clearIpcCalls(page);

    await page.getByTitle("Magic System Rules").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();

    await renameInput.press("Escape");
    await expect(renameInput).not.toBeVisible();

    await expect(page.getByTitle("Magic System Rules")).toBeVisible();

    const renameCalls = await getIpcCallsByCommand(page, "rename_note");
    expect(renameCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Edge cases
  // ---------------------------------------------------------------------------

  test("rename chapter with same title does not call IPC", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Rename" }).click();

    const renameInput = page.locator("input.rename-input");
    await expect(renameInput).toBeVisible();

    // Press Enter without changing the title
    await renameInput.press("Enter");

    await page.waitForTimeout(500);

    // rename_chapter should NOT have been called (title unchanged)
    const renameCalls = await getIpcCallsByCommand(page, "rename_chapter");
    expect(renameCalls.length).toBe(0);
  });
});
