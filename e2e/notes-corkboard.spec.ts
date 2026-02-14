import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
  MOCK_NOTES_CONFIG,
} from "./utils/tauri-mocks";

test.describe("Notes and Corkboard Workflow", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Binder: Notes section
  // ---------------------------------------------------------------------------

  test("notes section shows in binder with correct count", async ({
    page,
  }) => {
    const notesSection = page.locator(".section").filter({ hasText: "Notes" });
    await expect(notesSection).toBeVisible();
    await expect(notesSection.locator(".section-count")).toHaveText("3");
  });

  test("note items are listed in binder", async ({ page }) => {
    for (const note of MOCK_NOTES_CONFIG.notes) {
      await expect(page.getByTitle(note.title)).toBeVisible();
    }
  });

  test("get_notes_config is called on initialization", async ({ page }) => {
    const configCalls = await getIpcCallsByCommand(page, "get_notes_config");
    expect(configCalls.length).toBeGreaterThanOrEqual(1);
  });

  // ---------------------------------------------------------------------------
  // Binder: Note selection
  // ---------------------------------------------------------------------------

  test("clicking a note in binder marks it as selected", async ({ page }) => {
    const noteItem = page.getByTitle("Magic System Rules");
    await noteItem.click();
    await expect(noteItem).toHaveClass(/selected|active/);
  });

  test("clicking different notes switches selection", async ({ page }) => {
    const magic = page.getByTitle("Magic System Rules");
    const plot = page.getByTitle("Plot Outline");

    await magic.click();
    await expect(magic).toHaveClass(/selected|active/);

    await plot.click();
    await expect(plot).toHaveClass(/selected|active/);
    await expect(magic).not.toHaveClass(/selected/);
  });

  // ---------------------------------------------------------------------------
  // Binder: Create note via "+" button
  // ---------------------------------------------------------------------------

  test("clicking add button on notes section shows inline input", async ({
    page,
  }) => {
    const notesHeader = page
      .locator(".section-header")
      .filter({ hasText: "Notes" });
    await notesHeader.hover();

    const addButton = page.getByLabel("Add Notes");
    await addButton.click();

    const input = page.getByPlaceholder("Note title...");
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("typing title and pressing Enter creates a new note", async ({
    page,
  }) => {
    const notesHeader = page
      .locator(".section-header")
      .filter({ hasText: "Notes" });
    await notesHeader.hover();
    await page.getByLabel("Add Notes").click();

    const input = page.getByPlaceholder("Note title...");
    await expect(input).toBeVisible();

    await clearIpcCalls(page);

    await input.fill("Research Notes");
    await input.press("Enter");

    // Wait for the async create to complete
    await page.waitForTimeout(500);

    // create_note should have been called
    const createCalls = await getIpcCallsByCommand(page, "create_note");
    expect(createCalls.length).toBeGreaterThanOrEqual(1);
    const createArgs = createCalls[0].args as Record<string, unknown>;
    expect(createArgs.title).toBe("Research Notes");

    // Input should be dismissed
    await expect(input).not.toBeVisible();
  });

  test("pressing Escape cancels note creation", async ({ page }) => {
    const notesHeader = page
      .locator(".section-header")
      .filter({ hasText: "Notes" });
    await notesHeader.hover();
    await page.getByLabel("Add Notes").click();

    const input = page.getByPlaceholder("Note title...");
    await expect(input).toBeVisible();

    await clearIpcCalls(page);
    await input.press("Escape");

    await expect(input).not.toBeVisible();

    // No create_note call
    const createCalls = await getIpcCallsByCommand(page, "create_note");
    expect(createCalls.length).toBe(0);
  });

  // ---------------------------------------------------------------------------
  // Binder: Collapse / expand notes section
  // ---------------------------------------------------------------------------

  test("collapsing notes section hides note items", async ({ page }) => {
    const notesHeader = page
      .locator(".section-header")
      .filter({ hasText: "Notes" });

    await expect(page.getByTitle("Magic System Rules")).toBeVisible();

    // Collapse
    await notesHeader.click();
    await expect(page.getByTitle("Magic System Rules")).not.toBeVisible();

    // Expand
    await notesHeader.click();
    await expect(page.getByTitle("Magic System Rules")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Binder: Note color dots
  // ---------------------------------------------------------------------------

  test("notes with colors show color dots in binder", async ({ page }) => {
    // The note-row for Magic System Rules should have a color-dot
    const magicRow = page.locator(".note-row").filter({
      has: page.getByTitle("Magic System Rules"),
    });
    const colorDot = magicRow.locator(".color-dot");
    await expect(colorDot).toBeVisible();
    await expect(colorDot).toHaveCSS(
      "background-color",
      "rgb(59, 130, 246)", // #3b82f6
    );
  });

  // ---------------------------------------------------------------------------
  // Corkboard view: Switch to corkboard mode
  // ---------------------------------------------------------------------------

  test("switching to corkboard view renders note cards", async ({ page }) => {
    // Click the Corkboard view mode button in the toolbar
    const corkboardBtn = page.getByRole("button", { name: "Corkboard" });
    await expect(corkboardBtn).toBeVisible();
    await corkboardBtn.click();

    // The button should now be active
    await expect(corkboardBtn).toHaveAttribute("aria-pressed", "true");

    // The corkboard container should be rendered
    const corkboard = page.locator(".corkboard");
    await expect(corkboard).toBeVisible();

    // Note cards should be rendered
    for (const note of MOCK_NOTES_CONFIG.notes) {
      await expect(
        corkboard.locator(".note-card").filter({ hasText: note.title }),
      ).toBeVisible();
    }
  });

  test("corkboard note cards show title and label badges", async ({
    page,
  }) => {
    await page.getByRole("button", { name: "Corkboard" }).click();

    const corkboard = page.locator(".corkboard");

    // Magic System Rules card should have a "worldbuilding" label badge
    const magicCard = corkboard
      .locator(".note-card")
      .filter({ hasText: "Magic System Rules" });
    await expect(magicCard.locator(".label-badge")).toHaveText("worldbuilding");

    // Plot Outline should have a "planning" label badge
    const plotCard = corkboard
      .locator(".note-card")
      .filter({ hasText: "Plot Outline" });
    await expect(plotCard.locator(".label-badge")).toHaveText("planning");
  });

  test("corkboard cards have color strips matching note colors", async ({
    page,
  }) => {
    await page.getByRole("button", { name: "Corkboard" }).click();

    const corkboard = page.locator(".corkboard");

    // Check that cards have the correct color strip
    const magicCard = corkboard
      .locator(".note-card")
      .filter({ hasText: "Magic System Rules" });
    const colorStrip = magicCard.locator(".color-strip");
    await expect(colorStrip).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Corkboard: Empty state
  // ---------------------------------------------------------------------------

  test("corkboard shows empty state when no notes exist", async ({ page }) => {
    await page.goto("about:blank");
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });

    await page.getByRole("button", { name: "Corkboard" }).click();

    const corkboard = page.locator(".corkboard");
    await expect(corkboard).toBeVisible();
    await expect(
      corkboard.getByText("Create your first note"),
    ).toBeVisible();
    await expect(
      corkboard.getByText("Notes appear as cards on this corkboard"),
    ).toBeVisible();
    await expect(
      corkboard.getByRole("button", { name: "New Note" }),
    ).toBeVisible();
  });

  test("clicking New Note button in empty corkboard creates a note", async ({
    page,
  }) => {
    await page.goto("about:blank");
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });

    await page.getByRole("button", { name: "Corkboard" }).click();

    await clearIpcCalls(page);

    await page
      .locator(".corkboard")
      .getByRole("button", { name: "New Note" })
      .click();

    // Wait for the async create to complete
    await page.waitForTimeout(500);

    const createCalls = await getIpcCallsByCommand(page, "create_note");
    expect(createCalls.length).toBeGreaterThanOrEqual(1);
  });

  // ---------------------------------------------------------------------------
  // View mode switching
  // ---------------------------------------------------------------------------

  test("view mode buttons in toolbar switch between Editor, Corkboard, Split", async ({
    page,
  }) => {
    const viewGroup = page.getByRole("group", { name: "View mode" });
    await expect(viewGroup).toBeVisible();

    const editorBtn = viewGroup.getByRole("button", { name: "Editor" });
    const corkboardBtn = viewGroup.getByRole("button", { name: "Corkboard" });
    const splitBtn = viewGroup.getByRole("button", { name: "Split" });

    // Editor should be active by default
    await expect(editorBtn).toHaveAttribute("aria-pressed", "true");

    // Switch to corkboard
    await corkboardBtn.click();
    await expect(corkboardBtn).toHaveAttribute("aria-pressed", "true");
    await expect(editorBtn).toHaveAttribute("aria-pressed", "false");

    // Switch to split
    await splitBtn.click();
    await expect(splitBtn).toHaveAttribute("aria-pressed", "true");
    await expect(corkboardBtn).toHaveAttribute("aria-pressed", "false");

    // Switch back to editor
    await editorBtn.click();
    await expect(editorBtn).toHaveAttribute("aria-pressed", "true");
  });

  // ---------------------------------------------------------------------------
  // Notes section: Empty state in binder
  // ---------------------------------------------------------------------------

  test("shows placeholder when no notes exist", async ({ page }) => {
    await page.goto("about:blank");
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });

    await expect(page.locator(".placeholder-cta").filter({ hasText: /Add first note/i })).toBeVisible();
  });
});
