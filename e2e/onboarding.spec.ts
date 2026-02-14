import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

// =============================================================================
// Phase 1: Entity creation via binder "+" button
// =============================================================================

test.describe("Entity creation via binder + button", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("clicking + on Characters section shows inline input", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await characterHeader.hover();

    const addButton = page.getByLabel("Add Characters");
    await addButton.click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("typing name + Enter creates entity and refreshes list", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await characterHeader.hover();

    await page.getByLabel("Add Characters").click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();

    await clearIpcCalls(page);
    await input.fill("Elena");
    await input.press("Enter");

    // Input should disappear after creation
    await expect(input).not.toBeVisible();

    // Should have called create_entity with correct args
    const createCalls = await getIpcCallsByCommand(page, "create_entity");
    expect(createCalls.length).toBeGreaterThanOrEqual(1);
    const lastCall = createCalls[createCalls.length - 1];
    expect((lastCall.args as Record<string, unknown>)?.schemaType).toBe(
      "character",
    );
    expect((lastCall.args as Record<string, unknown>)?.title).toBe("Elena");
  });

  test("clicking + on Places section shows inline input with correct placeholder", async ({
    page,
  }) => {
    const placeHeader = page
      .locator(".section-header")
      .filter({ hasText: "Places" });
    await placeHeader.hover();

    await page.getByLabel("Add Places").click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Place name..."]',
    );
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("typing name in Places + Enter creates place entity", async ({
    page,
  }) => {
    const placeHeader = page
      .locator(".section-header")
      .filter({ hasText: "Places" });
    await placeHeader.hover();

    await page.getByLabel("Add Places").click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Place name..."]',
    );
    await input.fill("Rivendell");
    await input.press("Enter");

    await expect(input).not.toBeVisible();

    const createCalls = await getIpcCallsByCommand(page, "create_entity");
    expect(createCalls.length).toBeGreaterThanOrEqual(1);
    const lastCall = createCalls[createCalls.length - 1];
    expect((lastCall.args as Record<string, unknown>)?.schemaType).toBe(
      "place",
    );
    expect((lastCall.args as Record<string, unknown>)?.title).toBe("Rivendell");
  });

  test("pressing Escape cancels entity creation", async ({ page }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await characterHeader.hover();

    await page.getByLabel("Add Characters").click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();

    await input.fill("Temp");
    await input.press("Escape");

    await expect(input).not.toBeVisible();

    // No create_entity call should have been made
    const createCalls = await getIpcCallsByCommand(page, "create_entity");
    expect(createCalls.length).toBe(0);
  });

  test("blurring empty input cancels entity creation without creating", async ({
    page,
  }) => {
    const characterHeader = page
      .locator(".section-header")
      .filter({ hasText: "Characters" });
    await characterHeader.hover();

    await page.getByLabel("Add Characters").click();

    const input = page.locator(
      '.binder-tree .inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();

    // Click elsewhere to blur
    await page.locator(".binder-tree").click({ position: { x: 5, y: 5 } });

    await expect(input).not.toBeVisible();
  });
});

// =============================================================================
// Phase 2: WelcomeCard in EditorArea
// =============================================================================

test.describe("WelcomeCard empty state", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("shows WelcomeCard with heading and action buttons when no tabs open", async ({
    page,
  }) => {
    // WelcomeCard should be visible in the editor area
    await expect(page.locator(".welcome-card")).toBeVisible();
    await expect(page.locator(".welcome-heading")).toBeVisible();

    // Primary CTA
    await expect(
      page.locator(".welcome-primary-cta"),
    ).toBeVisible();

    // Ghost buttons for entity types
    await expect(page.locator(".welcome-ghost-btn")).toHaveCount(3);
  });

  test("clicking 'Write first chapter' triggers chapter creation in binder", async ({
    page,
  }) => {
    await page.locator(".welcome-primary-cta").click();

    // The ManuscriptSection inline input should appear
    const chapterInput = page.locator(
      '.inline-input[placeholder="Chapter title..."]',
    );
    await expect(chapterInput).toBeVisible();
    await expect(chapterInput).toBeFocused();
  });

  test("clicking Character ghost button triggers entity creation in binder", async ({
    page,
  }) => {
    const characterBtn = page
      .locator(".welcome-ghost-btn")
      .filter({ hasText: /Character/i });
    await characterBtn.click();

    const input = page.locator(
      '.inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();
  });

  test("clicking Note ghost button triggers note creation in binder", async ({
    page,
  }) => {
    const noteBtn = page
      .locator(".welcome-ghost-btn")
      .filter({ hasText: /Note/i });
    await noteBtn.click();

    const noteInput = page.locator(
      '.inline-input[placeholder="Note title..."]',
    );
    await expect(noteInput).toBeVisible();
  });

  test("WelcomeCard disappears when a chapter is opened", async ({ page }) => {
    // Click on first chapter in binder to open it
    await page.getByTitle("1. The Awakening").click();

    // WelcomeCard should disappear
    await expect(page.locator(".welcome-card")).not.toBeVisible();
  });

  test("WelcomeCard reappears when all tabs are closed", async ({ page }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await expect(page.locator(".welcome-card")).not.toBeVisible();

    // Close the tab via Cmd+W
    await page.keyboard.press("ControlOrMeta+w");

    // WelcomeCard should reappear
    await expect(page.locator(".welcome-card")).toBeVisible();
  });

  test("keyboard hints are visible in WelcomeCard", async ({ page }) => {
    const hints = page.locator(".welcome-hints");
    await expect(hints).toBeVisible();
    await expect(hints).toContainText("K");
  });
});

// =============================================================================
// Phase 3: Binder actionable empty states (CTAs)
// =============================================================================

test.describe("Binder empty state CTAs", () => {
  test("shows 'Add first chapter' CTA in empty manuscript section", async ({
    page,
  }) => {
    // Open with empty manuscript
    await openMockProject(page, {
      get_manuscript_config: { chapters: [] },
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first chapter/i });
    await expect(cta).toBeVisible();
  });

  test("clicking 'Add first chapter' CTA triggers inline input", async ({
    page,
  }) => {
    await openMockProject(page, {
      get_manuscript_config: { chapters: [] },
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first chapter/i });
    await cta.click();

    const input = page.locator(
      '.inline-input[placeholder="Chapter title..."]',
    );
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("shows 'Add first character' CTA in empty character section", async ({
    page,
  }) => {
    // Open with empty entities for character type
    await page.goto("about:blank");
    await openMockProject(page, {
      list_entities: new Function(
        "args",
        `var schemaType = args && args.schemaType;
         if (schemaType === "place") return [
           { title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: [] }
         ];
         return [];`,
      ),
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first character/i });
    await expect(cta).toBeVisible();
  });

  test("clicking 'Add first character' CTA triggers inline input", async ({
    page,
  }) => {
    await page.goto("about:blank");
    await openMockProject(page, {
      list_entities: new Function(
        "args",
        `var schemaType = args && args.schemaType;
         if (schemaType === "place") return [
           { title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: [] }
         ];
         return [];`,
      ),
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first character/i });
    await cta.click();

    const input = page.locator(
      '.inline-input[placeholder="Character name..."]',
    );
    await expect(input).toBeVisible();
  });

  test("shows 'Add first note' CTA in empty notes section", async ({
    page,
  }) => {
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first note/i });
    await expect(cta).toBeVisible();
  });

  test("clicking 'Add first note' CTA triggers inline input", async ({
    page,
  }) => {
    await openMockProject(page, {
      get_notes_config: { notes: [] },
    });

    const cta = page.locator(".placeholder-cta").filter({ hasText: /Add first note/i });
    await cta.click();

    const input = page.locator('.inline-input[placeholder="Note title..."]');
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });
});
