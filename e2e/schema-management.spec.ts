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

  // ===========================================================================
  // BUG-007: SchemaEditor rendering + $state.snapshot() fixes
  // ===========================================================================

  test("SchemaEditor renders without freeze when creating new entity type", async ({
    page,
  }) => {
    // Click "New Entity Type" button (standalone, not context menu)
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await expect(newTypeBtn).toBeVisible();
    await newTypeBtn.click();

    // Tab should appear
    await expect(
      page.getByRole("tab", { name: "New Entity Type" }),
    ).toBeVisible({ timeout: 3000 });

    // SchemaEditor should be visible (not frozen)
    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Metadata inputs should be present
    await expect(schemaEditor.getByPlaceholder("e.g. Character, Location")).toBeVisible();
    await expect(schemaEditor.getByPlaceholder("auto-generated")).toBeVisible();
    await expect(schemaEditor.getByPlaceholder("e.g. user, map-pin")).toBeVisible();
    await expect(schemaEditor.getByPlaceholder("e.g. #7c4dbd")).toBeVisible();
    await expect(schemaEditor.getByPlaceholder("Describe this entity type...")).toBeVisible();
  });

  test("SchemaEditor name auto-generates slug for new entity type", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Type a name
    const nameInput = schemaEditor.getByPlaceholder("e.g. Character, Location");
    await nameInput.fill("Magic Item");

    // Slug should auto-generate
    const slugInput = schemaEditor.getByPlaceholder("auto-generated");
    await expect(slugInput).toHaveValue("magic-item");
  });

  test("SchemaEditor optional fields (icon, color, description) accept input", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Fill optional fields — these use bind:value which was causing freeze
    const iconInput = schemaEditor.getByPlaceholder("e.g. user, map-pin");
    const colorInput = schemaEditor.getByPlaceholder("e.g. #7c4dbd");
    const descInput = schemaEditor.getByPlaceholder("Describe this entity type...");

    await iconInput.fill("sword");
    await colorInput.fill("#ff5500");
    await descInput.fill("A magical artifact");

    // Values should persist
    await expect(iconInput).toHaveValue("sword");
    await expect(colorInput).toHaveValue("#ff5500");
    await expect(descInput).toHaveValue("A magical artifact");
  });

  test("SchemaEditor add field and add axis create cards", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Initially no field or axis cards
    await expect(schemaEditor.locator(".card")).toHaveCount(0);

    // Add a field
    const addFieldBtn = schemaEditor.getByRole("button", { name: "Add Field" });
    await addFieldBtn.click();
    await expect(schemaEditor.locator(".card")).toHaveCount(1);

    // Add an axis
    const addAxisBtn = schemaEditor.getByRole("button", { name: "Add Axis" });
    await addAxisBtn.click();
    await expect(schemaEditor.locator(".card")).toHaveCount(2);
  });

  test("SchemaEditor YAML preview toggle shows schema content", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Fill name so YAML has content
    const nameInput = schemaEditor.getByPlaceholder("e.g. Character, Location");
    await nameInput.fill("Weapon");

    // YAML preview should not be visible initially
    const yamlPre = schemaEditor.locator(".yaml-preview");
    await expect(yamlPre).not.toBeVisible();

    // Click YAML Preview toggle
    const yamlToggle = schemaEditor.locator(".section-toggle");
    await yamlToggle.click();

    // YAML preview should now be visible and contain the name
    await expect(yamlPre).toBeVisible();
    await expect(yamlPre).toContainText("Weapon");
  });

  test("SchemaEditor save calls save_schema IPC with correct data", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Fill name (which auto-generates entityType slug)
    await schemaEditor
      .getByPlaceholder("e.g. Character, Location")
      .fill("Artifact");

    await clearIpcCalls(page);

    // Click Save
    await schemaEditor.getByRole("button", { name: "Save Schema" }).click();

    // Wait for async IPC
    await page.waitForTimeout(500);

    // save_schema IPC should have been called
    const saveCalls = await getIpcCallsByCommand(page, "save_schema");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);

    const args = saveCalls[0].args as Record<string, unknown>;
    const schema = args.schema as Record<string, unknown>;
    expect(schema.name).toBe("Artifact");
    expect(schema.entityType).toBe("artifact");
  });

  test("SchemaEditor cancel closes tab without calling save_schema", async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await newTypeBtn.click();

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    await clearIpcCalls(page);

    // Click Cancel
    await schemaEditor.getByRole("button", { name: "Cancel" }).click();

    // Tab should be closed
    await expect(
      page.getByRole("tab", { name: "New Entity Type" }),
    ).not.toBeVisible();

    // No save_schema IPC
    const saveCalls = await getIpcCallsByCommand(page, "save_schema");
    expect(saveCalls.length).toBe(0);
  });

  test("standalone New Entity Type button opens schema tab", async ({
    page,
  }) => {
    // The standalone button is below entity sections, not in context menu
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await expect(newTypeBtn).toBeVisible();
    await expect(newTypeBtn).toContainText("New Entity Type");

    await newTypeBtn.click();

    // Tab opens
    await expect(
      page.getByRole("tab", { name: "New Entity Type" }),
    ).toBeVisible({ timeout: 3000 });

    // SchemaEditor renders
    await expect(page.locator(".schema-editor")).toBeVisible({ timeout: 3000 });
  });

  test("Edit Type populates SchemaEditor with existing schema data", async ({
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

    // Wait for schema tab
    await expect(
      page.getByRole("tab", { name: "Character (Schema)" }),
    ).toBeVisible({ timeout: 5000 });

    const schemaEditor = page.locator(".schema-editor");
    await expect(schemaEditor).toBeVisible({ timeout: 3000 });

    // Name should be populated from mock schema
    const nameInput = schemaEditor.getByPlaceholder("e.g. Character, Location");
    await expect(nameInput).toHaveValue("Character");

    // Icon should be populated
    const iconInput = schemaEditor.getByPlaceholder("e.g. user, map-pin");
    await expect(iconInput).toHaveValue("users");

    // Color should be populated
    const colorInput = schemaEditor.getByPlaceholder("e.g. #7c4dbd");
    await expect(colorInput).toHaveValue("#7c4dbd");

    // Should have field cards (mock character schema has 3 fields)
    const fieldCards = schemaEditor.locator(".card");
    await expect(fieldCards).toHaveCount(7); // 3 fields + 4 spider axes
  });

  test("creating entity does not cause redundant list_entities call", async ({
    page,
  }) => {
    // Wait for initial loads to settle
    await page.waitForTimeout(1000);
    await clearIpcCalls(page);

    // Create an entity via binder
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await expect(characterHeader).toBeVisible();

    // Click the add button for characters
    const addBtn = characterHeader.locator(".section-add-btn");
    await addBtn.click();

    // Type entity name and confirm
    const input = page.locator(".inline-input").first();
    await expect(input).toBeVisible();
    await input.fill("Test Hero");
    await input.press("Enter");

    // Wait for async operations
    await page.waitForTimeout(1000);

    // Count list_entities calls for character type
    const listCalls = await getIpcCallsByCommand(page, "list_entities");
    const characterListCalls = listCalls.filter(
      (c) => (c.args as Record<string, unknown>).schemaType === "character",
    );

    // Should be exactly 1 (from createEntity internally), not 2
    expect(characterListCalls.length).toBe(1);
  });
});
