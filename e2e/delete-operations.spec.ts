import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Delete Operations", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Delete chapter
  // ---------------------------------------------------------------------------

  test("delete chapter: right-click -> Delete -> confirm -> IPC called", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on first chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Delete
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    // Confirm dialog should appear
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Delete Chapter");
    await expect(dialog).toContainText("The Awakening");

    // Click confirm (the Delete button inside the dialog)
    await dialog.getByRole("button", { name: "Delete" }).click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // delete_chapter IPC should have been called
    const deleteCalls = await getIpcCallsByCommand(page, "delete_chapter");
    expect(deleteCalls.length).toBeGreaterThanOrEqual(1);
    const args = deleteCalls[0].args as Record<string, unknown>;
    expect(args.slug).toBe("the-awakening");
  });

  test("delete chapter: cancel path -> no IPC call", async ({ page }) => {
    await clearIpcCalls(page);

    // Right-click on first chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    // Confirm dialog should appear
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Click Cancel
    await dialog.getByRole("button", { name: "Cancel" }).click();

    // Dialog should close
    await expect(dialog).not.toBeVisible();

    // No delete_chapter IPC should have been called
    const deleteCalls = await getIpcCallsByCommand(page, "delete_chapter");
    expect(deleteCalls.length).toBe(0);
  });

  test("delete chapter: Escape closes confirm dialog", async ({ page }) => {
    // Right-click on chapter
    await page.getByTitle("1. The Awakening").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Press Escape
    await page.keyboard.press("Escape");

    // Dialog should close
    await expect(dialog).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Delete entity
  // ---------------------------------------------------------------------------

  test("delete entity: right-click -> Delete -> confirm -> IPC called with schemaType", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on a character entity
    await page.getByTitle("Elena Blackwood").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Delete
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    // Confirm dialog should appear
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Delete Entity");
    await expect(dialog).toContainText("Elena Blackwood");

    // Confirm deletion
    await dialog.getByRole("button", { name: "Delete" }).click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // delete_entity IPC should have been called with correct schemaType
    const deleteCalls = await getIpcCallsByCommand(page, "delete_entity");
    expect(deleteCalls.length).toBeGreaterThanOrEqual(1);
    const args = deleteCalls[0].args as Record<string, unknown>;
    expect(args.schemaType).toBe("character");
    expect(args.slug).toBe("elena-blackwood");
  });

  test("delete entity: cancel path -> no IPC call", async ({ page }) => {
    await clearIpcCalls(page);

    await page.getByTitle("Elena Blackwood").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Cancel
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).not.toBeVisible();

    const deleteCalls = await getIpcCallsByCommand(page, "delete_entity");
    expect(deleteCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Delete note
  // ---------------------------------------------------------------------------

  test("delete note: right-click -> Delete -> confirm -> IPC called", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    // Right-click on a note
    await page.getByTitle("Magic System Rules").click({ button: "right" });
    await expect(page.locator('[role="menu"]')).toBeVisible();

    // Click Delete
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    // Confirm dialog should appear
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Delete Note");
    await expect(dialog).toContainText("Magic System Rules");

    // Confirm deletion
    await dialog.getByRole("button", { name: "Delete" }).click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // delete_note IPC should have been called
    const deleteCalls = await getIpcCallsByCommand(page, "delete_note");
    expect(deleteCalls.length).toBeGreaterThanOrEqual(1);
    const args = deleteCalls[0].args as Record<string, unknown>;
    expect(args.slug).toBe("magic-system");
  });

  test("delete note: cancel path -> no IPC call", async ({ page }) => {
    await clearIpcCalls(page);

    await page.getByTitle("Magic System Rules").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Cancel
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).not.toBeVisible();

    const deleteCalls = await getIpcCallsByCommand(page, "delete_note");
    expect(deleteCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Delete place entity (different schema type)
  // ---------------------------------------------------------------------------

  test("delete place entity passes correct schemaType", async ({ page }) => {
    await clearIpcCalls(page);

    await page.getByTitle("Ironhaven").click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    await dialog.getByRole("button", { name: "Delete" }).click();
    await page.waitForTimeout(500);

    const deleteCalls = await getIpcCallsByCommand(page, "delete_entity");
    expect(deleteCalls.length).toBeGreaterThanOrEqual(1);
    const args = deleteCalls[0].args as Record<string, unknown>;
    expect(args.schemaType).toBe("place");
    expect(args.slug).toBe("ironhaven");
  });
});
