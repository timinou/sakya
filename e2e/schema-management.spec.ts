import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Schema Management", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Section header context menu visibility
  // ---------------------------------------------------------------------------

  test("right-click entity section header shows context menu with Edit Type, New Entity Type, Delete Type", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await expect(characterHeader).toBeVisible();

    // Right-click the section header
    await characterHeader.click({ button: "right" });

    // Context menu should appear
    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    // Should contain all three schema management items
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Edit Type..." }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "New Entity Type..." }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete Type" }),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Section header context menu for different entity type
  // ---------------------------------------------------------------------------

  test("right-click Places section header also shows schema context menu", async ({
    page,
  }) => {
    const placeHeader = page
      .locator(".section-header")
      .filter({ hasText: "Places" });
    await expect(placeHeader).toBeVisible();

    await placeHeader.click({ button: "right" });

    const menu = page.locator('[role="menu"]');
    await expect(menu).toBeVisible();

    await expect(
      page.getByRole("menuitem").filter({ hasText: "Edit Type..." }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "New Entity Type..." }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem").filter({ hasText: "Delete Type" }),
    ).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Delete Type shows destructive confirm dialog
  // ---------------------------------------------------------------------------

  test("Delete Type shows destructive confirm dialog with entity type name", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });

    await characterHeader.click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete Type" }).click();

    // Confirm dialog should appear
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Delete Entity Type");
    await expect(dialog).toContainText("Characters");
    await expect(dialog).toContainText("cannot be undone");

    // Should have a destructive-styled confirm button labeled "Delete Type"
    const deleteBtn = dialog.getByRole("button", { name: "Delete Type" });
    await expect(deleteBtn).toBeVisible();

    // Should have a cancel button
    const cancelBtn = dialog.getByRole("button", { name: "Cancel" });
    await expect(cancelBtn).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Confirm delete calls delete_schema IPC
  // ---------------------------------------------------------------------------

  test("confirming Delete Type calls delete_schema IPC with correct schemaType", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });

    await characterHeader.click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete Type" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Click the "Delete Type" confirm button
    await dialog.getByRole("button", { name: "Delete Type" }).click();

    // Wait for async operation
    await page.waitForTimeout(500);

    // delete_schema IPC should have been called with the correct schema type
    const deleteCalls = await getIpcCallsByCommand(page, "delete_schema");
    expect(deleteCalls.length).toBeGreaterThanOrEqual(1);
    const args = deleteCalls[0].args as Record<string, unknown>;
    expect(args.schemaType).toBe("character");
  });

  // ---------------------------------------------------------------------------
  // Cancel delete does not call IPC
  // ---------------------------------------------------------------------------

  test("cancelling Delete Type does not call delete_schema IPC", async ({
    page,
  }) => {
    await clearIpcCalls(page);

    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });

    await characterHeader.click({ button: "right" });
    await page.getByRole("menuitem").filter({ hasText: "Delete Type" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Click Cancel
    await dialog.getByRole("button", { name: "Cancel" }).click();

    // Dialog should close
    await expect(dialog).not.toBeVisible();

    // No delete_schema IPC should have been called
    const deleteCalls = await getIpcCallsByCommand(page, "delete_schema");
    expect(deleteCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Dynamic icon/color from schema cache
  // ---------------------------------------------------------------------------

  test("entity section headers render dynamic icon color from schema cache", async ({
    page,
  }) => {
    // Character schema has color "#7c4dbd" — the section-icon should use this color
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await expect(characterHeader).toBeVisible();

    const characterIcon = characterHeader.locator(".section-icon");
    await expect(characterIcon).toBeVisible();
    await expect(characterIcon).toHaveCSS("color", "rgb(124, 77, 189)");

    // Place schema has color "#2e8b57" — the section-icon should use this color
    const placeHeader = page
      .locator(".section-header")
      .filter({ hasText: "Places" });
    await expect(placeHeader).toBeVisible();

    const placeIcon = placeHeader.locator(".section-icon");
    await expect(placeIcon).toBeVisible();
    await expect(placeIcon).toHaveCSS("color", "rgb(46, 139, 87)");
  });

  // ---------------------------------------------------------------------------
  // Edit Type dispatches sakya:edit-schema event and opens schema tab
  // ---------------------------------------------------------------------------

  test("Edit Type opens schema editor tab for the entity type", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });

    await characterHeader.click({ button: "right" });
    await page
      .getByRole("menuitem")
      .filter({ hasText: "Edit Type..." })
      .click();

    // The context menu should close
    await expect(page.locator('[role="menu"]')).not.toBeVisible();

    // A schema tab should open — tab title includes "(Schema)"
    await expect(
      page.getByRole("tab", { name: "Character (Schema)" }),
    ).toBeVisible({ timeout: 5000 });

    // get_schema IPC should have been called for the character type
    const schemaCalls = await getIpcCallsByCommand(page, "get_schema");
    const characterCalls = schemaCalls.filter(
      (c) => (c.args as Record<string, unknown>).schemaType === "character",
    );
    expect(characterCalls.length).toBeGreaterThanOrEqual(1);
  });

  // ---------------------------------------------------------------------------
  // New Entity Type opens a blank schema tab
  // ---------------------------------------------------------------------------

  test("New Entity Type opens a blank schema editor tab", async ({ page }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });

    await characterHeader.click({ button: "right" });
    await page
      .getByRole("menuitem")
      .filter({ hasText: "New Entity Type..." })
      .click();

    // The context menu should close
    await expect(page.locator('[role="menu"]')).not.toBeVisible();

    // A new schema tab should open with "New Entity Type" title
    await expect(
      page.getByRole("tab", { name: "New Entity Type" }),
    ).toBeVisible({ timeout: 5000 });
  });
});
