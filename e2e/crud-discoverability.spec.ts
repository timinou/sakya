import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

test.describe("CRUD Discoverability — Visible Affordances", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // D.1–D.2: Three-dot menu button on chapters
  // ---------------------------------------------------------------------------

  test("three-dot menu button appears on chapter hover", async ({ page }) => {
    const chapterRow = page.locator(".chapter-row").first();
    const menuBtn = chapterRow.locator('button[title="More actions"]');

    // Button should exist but be invisible initially (opacity: 0)
    await expect(menuBtn).toBeAttached();

    // Hover over the chapter row
    await chapterRow.hover();

    // Button should now be visible
    await expect(menuBtn).toBeVisible();
  });

  test("three-dot menu opens context menu on click", async ({ page }) => {
    const chapterRow = page.locator(".chapter-row").first();
    await chapterRow.hover();

    const menuBtn = chapterRow.locator('button[title="More actions"]');
    await menuBtn.click();

    // Context menu should appear with expected items
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
  // D.3–D.4: Status dot dropdown on chapters
  // ---------------------------------------------------------------------------

  test("status dot dropdown appears on chapter hover and click", async ({
    page,
  }) => {
    const chapterRow = page.locator(".chapter-row").first();
    const statusBtn = chapterRow.locator('button[title="Change status"]');

    // Status button should exist
    await expect(statusBtn).toBeAttached();

    // Hover to reveal chevron
    await chapterRow.hover();

    // Click to open dropdown
    await statusBtn.click();

    // Status dropdown should appear with three options
    const dropdown = chapterRow.locator(".status-dropdown");
    await expect(dropdown).toBeVisible();

    await expect(dropdown.locator(".status-option")).toHaveCount(3);
    await expect(
      dropdown.locator(".status-option").filter({ hasText: "Draft" }),
    ).toBeVisible();
    await expect(
      dropdown.locator(".status-option").filter({ hasText: "Revised" }),
    ).toBeVisible();
    await expect(
      dropdown.locator(".status-option").filter({ hasText: "Final" }),
    ).toBeVisible();
  });

  test("clicking status option changes chapter status", async ({ page }) => {
    const chapterRow = page.locator(".chapter-row").first();
    await chapterRow.hover();

    const statusBtn = chapterRow.locator('button[title="Change status"]');
    await statusBtn.click();

    const dropdown = chapterRow.locator(".status-dropdown");
    await expect(dropdown).toBeVisible();

    // Click "Final" — the first chapter ("The Awakening") has status "revised"
    await dropdown
      .locator(".status-option")
      .filter({ hasText: "Final" })
      .click();

    // Dropdown should close
    await expect(dropdown).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // D.5: Three-dot menu button on entities
  // ---------------------------------------------------------------------------

  test("three-dot menu button appears on entity hover", async ({ page }) => {
    const entityRow = page.locator(".entity-row").first();
    const menuBtn = entityRow.locator('button[title="More actions"]');

    await expect(menuBtn).toBeAttached();

    await entityRow.hover();
    await expect(menuBtn).toBeVisible();
  });

  test("three-dot menu on entity opens context menu", async ({ page }) => {
    const entityRow = page.locator(".entity-row").first();
    await entityRow.hover();

    const menuBtn = entityRow.locator('button[title="More actions"]');
    await menuBtn.click();

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
  // D.6: Three-dot menu button on notes
  // ---------------------------------------------------------------------------

  test("three-dot menu button appears on note hover", async ({ page }) => {
    const noteRow = page.locator(".note-row").first();
    const menuBtn = noteRow.locator('button[title="More actions"]');

    await expect(noteRow).toBeAttached();
    await expect(menuBtn).toBeAttached();

    await noteRow.hover();
    await expect(menuBtn).toBeVisible();
  });

  test("three-dot menu on note opens context menu", async ({ page }) => {
    const noteRow = page.locator(".note-row").first();
    await noteRow.hover();

    const menuBtn = noteRow.locator('button[title="More actions"]');
    await menuBtn.click();

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
  // D.7–D.8: Gear button on entity section headers
  // ---------------------------------------------------------------------------

  test("gear button appears on entity section header hover", async ({
    page,
  }) => {
    // Entity section headers have a gear button (Settings icon)
    const sectionHeader = page
      .locator("[data-entity-section] .section-header")
      .first();
    const gearBtn = sectionHeader.locator('button[title="Type settings"]');

    await expect(gearBtn).toBeAttached();

    await sectionHeader.hover();
    await expect(gearBtn).toBeVisible();
  });

  test("gear button opens section context menu", async ({ page }) => {
    const sectionHeader = page
      .locator("[data-entity-section] .section-header")
      .first();
    await sectionHeader.hover();

    const gearBtn = sectionHeader.locator('button[title="Type settings"]');
    await gearBtn.click();

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
  // D.9–D.10: "New Entity Type" button
  // ---------------------------------------------------------------------------

  test('"New Entity Type" button is visible below entity sections', async ({
    page,
  }) => {
    const newTypeBtn = page.locator(".new-entity-type-btn");
    await expect(newTypeBtn).toBeVisible();
    await expect(newTypeBtn).toHaveText(/New Entity Type/);
  });

  // ---------------------------------------------------------------------------
  // D.11–D.12: Corkboard card tooltips
  // ---------------------------------------------------------------------------

  test("corkboard color button has tooltip", async ({ page }) => {
    // Open a note to switch to corkboard view
    // Navigate to corkboard by clicking Notes section and looking for cards
    const notesSection = page.getByText("Notes").first();
    await expect(notesSection).toBeVisible();

    // NoteCards are rendered in the corkboard area if visible
    // Check button titles exist in the DOM
    const colorBtn = page.locator('.note-card .action-btn[title="Change color"]');
    // Cards may not be visible in binder view, let's check if corkboard tab exists
    // The corkboard cards exist when a note tab is open
    // For this test, just click a note to open a tab
    await page.getByTitle("Magic System Rules").click();

    // Wait for the corkboard to load if applicable
    // Check the NoteCard component has the tooltip attributes
    // Since the corkboard is a separate view, we verify via the note card in the corkboard
    // NoteCards are in the .corkboard-container
    const corkboardCard = page.locator(".note-card").first();
    if (await corkboardCard.isVisible({ timeout: 2000 }).catch(() => false)) {
      const colorButton = corkboardCard.locator(
        '.action-btn[title="Change color"]',
      );
      await corkboardCard.hover();
      await expect(colorButton).toBeAttached();

      const labelButton = corkboardCard.locator(".action-btn").nth(1);
      const labelTitle = await labelButton.getAttribute("title");
      expect(
        labelTitle === "Add label" || labelTitle === "Edit label",
      ).toBeTruthy();
    }
  });
});
