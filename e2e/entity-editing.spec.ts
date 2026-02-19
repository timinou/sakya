import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

// =============================================================================
// Entity Editing E2E Tests (ITEM-115 / BUG-002)
// =============================================================================

test.describe("Entity Editing via Binder", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("clicking entity in binder opens entity tab", async ({ page }) => {
    // Click on "Elena Blackwood" in the binder (character entity)
    await page.getByText("Elena Blackwood").click();

    // Should open a tab with the entity title
    const tab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(tab).toBeVisible({ timeout: 3000 });
    await expect(tab).toHaveClass(/active/);
  });

  test("entity tab shows EntityForm with title and fields", async ({
    page,
  }) => {
    // Click on an entity in the binder
    await page.getByText("Elena Blackwood").click();

    // Wait for EntityForm to render — check for title input
    const titleInput = page.getByLabel("Entity title");
    await expect(titleInput).toBeVisible({ timeout: 5000 });
    await expect(titleInput).toHaveValue("Elena Blackwood");

    // Check for tags
    const tagsInput = page.locator("#entity-tags");
    await expect(tagsInput).toBeVisible();

    // Check for the Fields section heading
    const fieldsHeading = page.getByText("Fields", { exact: true });
    await expect(fieldsHeading).toBeVisible();
  });

  test("entity tab shows spider chart for character entities", async ({
    page,
  }) => {
    // Click on a character (has spider axes in mock data)
    await page.getByText("Elena Blackwood").click();

    // Wait for EntityForm
    await expect(page.getByLabel("Entity title")).toBeVisible({ timeout: 5000 });

    // Check for characteristics section
    const characteristicsHeading = page.getByText("Characteristics");
    await expect(characteristicsHeading).toBeVisible();
  });

  test("entity field editing triggers save_entity IPC", async ({ page }) => {
    // Click on entity
    await page.getByText("Elena Blackwood").click();

    // Wait for form to load
    const titleInput = page.getByLabel("Entity title");
    await expect(titleInput).toBeVisible({ timeout: 5000 });

    // Clear IPC calls before editing
    await clearIpcCalls(page);

    // Modify the title
    await titleInput.fill("Elena Blackwood-Updated");

    // Wait for debounced auto-save (1.5s + buffer)
    await page.waitForTimeout(2500);

    // Verify save_entity was called
    const saveCalls = await getIpcCallsByCommand(page, "save_entity");
    expect(saveCalls.length).toBeGreaterThan(0);
  });

  test("clicking different entity switches to its tab", async ({ page }) => {
    // Open first entity
    await page.getByText("Elena Blackwood").click();
    await expect(page.getByLabel("Entity title")).toBeVisible({ timeout: 5000 });

    // Open second entity
    await page.getByText("Marcus Thorne").click();

    // Should have two tabs, second one active
    const elenaTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    const marcusTab = page.locator('.tab').filter({ hasText: "Marcus Thorne" });
    await expect(elenaTab).toBeVisible();
    await expect(marcusTab).toBeVisible();
    await expect(marcusTab).toHaveClass(/active/);
  });

  test("closing entity tab removes it", async ({ page }) => {
    // Open an entity
    await page.getByText("Elena Blackwood").click();

    const tab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(tab).toBeVisible({ timeout: 3000 });

    // Close the tab
    const closeBtn = tab.getByTitle("Close tab");
    await closeBtn.click();

    // Tab should be gone
    await expect(tab).not.toBeVisible();
  });
});

test.describe("Entity Editing via Search Palette", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("search palette entity navigation opens entity tab", async ({
    page,
  }) => {
    // Open search palette
    await page.keyboard.press("Control+k");

    // Type a search query that matches an entity
    const searchDialog = page.locator('[aria-label="Search project"]');
    await expect(searchDialog).toBeVisible({ timeout: 3000 });
    const searchInput = searchDialog.getByRole("textbox");
    await searchInput.fill("Elena");

    // Wait for results to appear
    await page.waitForTimeout(500);

    // Click the entity result within the search dialog
    const entityResult = searchDialog.getByText("Elena Blackwood").first();
    await entityResult.click();

    // Should open a tab
    const tab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(tab).toBeVisible({ timeout: 3000 });
  });
});

// =============================================================================
// Cross-type tab switching (BUG-006)
// =============================================================================

test.describe("Entity tab activation — cross-type switching", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("clicking entity when chapter is active switches to entity tab", async ({ page }) => {
    const binder = page.locator('.binder-content');

    // Open a chapter first
    await binder.getByText("The Awakening").click();
    const chapterTab = page.locator('.tab').filter({ hasText: "The Awakening" });
    await expect(chapterTab).toBeVisible({ timeout: 3000 });
    await expect(chapterTab).toHaveClass(/active/);

    // Now click an entity in the binder (scoped to avoid POV dropdown match)
    await binder.getByText("Elena Blackwood").click();

    // Entity tab should be active, chapter tab should NOT be active
    const entityTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(entityTab).toBeVisible({ timeout: 3000 });
    await expect(entityTab).toHaveClass(/active/);
    await expect(chapterTab).not.toHaveClass(/active/);
  });

  test("clicking entity when note is active switches to entity tab", async ({ page }) => {
    // Open a note first
    await page.getByText("Magic System Rules").click();
    const noteTab = page.locator('.tab').filter({ hasText: "Magic System Rules" });
    await expect(noteTab).toBeVisible({ timeout: 3000 });
    await expect(noteTab).toHaveClass(/active/);

    // Now click an entity
    await page.getByText("Elena Blackwood").click();

    // Entity tab should be active, note tab should NOT be active
    const entityTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(entityTab).toBeVisible({ timeout: 3000 });
    await expect(entityTab).toHaveClass(/active/);
    await expect(noteTab).not.toHaveClass(/active/);
  });

  test("clicking different entities switches active tab", async ({ page }) => {
    // Open entity A
    await page.getByText("Elena Blackwood").click();
    const elenaTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(elenaTab).toBeVisible({ timeout: 3000 });
    await expect(elenaTab).toHaveClass(/active/);

    // Open entity B
    await page.getByText("Marcus Thorne").click();
    const marcusTab = page.locator('.tab').filter({ hasText: "Marcus Thorne" });
    await expect(marcusTab).toBeVisible({ timeout: 3000 });
    await expect(marcusTab).toHaveClass(/active/);

    // Elena tab still visible but not active
    await expect(elenaTab).toBeVisible();
    await expect(elenaTab).not.toHaveClass(/active/);
  });

  test("can return to chapter tab after opening entity", async ({ page }) => {
    const binder = page.locator('.binder-content');

    // Open a chapter
    await binder.getByText("The Awakening").click();
    const chapterTab = page.locator('.tab').filter({ hasText: "The Awakening" });
    await expect(chapterTab).toBeVisible({ timeout: 3000 });

    // Open an entity (should become active) — scoped to binder to avoid POV dropdown
    await binder.getByText("Elena Blackwood").click();
    const entityTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(entityTab).toHaveClass(/active/, { timeout: 3000 });

    // Click back on the chapter tab
    await chapterTab.click();
    await expect(chapterTab).toHaveClass(/active/);
    await expect(entityTab).not.toHaveClass(/active/);
  });

  test("clicking entity with no prior tab makes it active", async ({ page }) => {
    // No tabs open initially — welcome card visible
    const welcome = page.locator('.welcome-card');
    await expect(welcome).toBeVisible({ timeout: 3000 });

    // Click entity
    await page.getByText("Elena Blackwood").click();
    const entityTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(entityTab).toBeVisible({ timeout: 3000 });
    await expect(entityTab).toHaveClass(/active/);

    // Welcome card should be gone
    await expect(welcome).not.toBeVisible();
  });

  test("activeChapterSlug is cleared after clicking entity", async ({ page }) => {
    const binder = page.locator('.binder-content');

    // Open a chapter first
    await binder.getByText("The Awakening").click();
    const chapterTab = page.locator('.tab').filter({ hasText: "The Awakening" });
    await expect(chapterTab).toBeVisible({ timeout: 3000 });

    // Click entity — scoped to binder to avoid POV dropdown
    await binder.getByText("Elena Blackwood").click();
    const entityTab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(entityTab).toHaveClass(/active/, { timeout: 3000 });

    // Verify store slug is cleared via page.evaluate
    const slug = await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      return (stores as any).manuscriptStore.activeChapterSlug;
    });
    expect(slug).toBe('');
  });
});

test.describe("Entity Tab ID Consistency", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("deleting entity from context menu closes correct tab", async ({
    page,
  }) => {
    // Open an entity to create a tab
    const binderEntity = page.locator('.binder-content').getByText("Elena Blackwood");
    await binderEntity.click();
    const tab = page.locator('.tab').filter({ hasText: "Elena Blackwood" });
    await expect(tab).toBeVisible({ timeout: 3000 });

    // Right-click entity in binder for context menu
    await binderEntity.click({ button: "right" });

    // Click delete in context menu
    const deleteItem = page.getByText("Delete", { exact: true });
    await expect(deleteItem).toBeVisible();
    await deleteItem.click();

    // Confirm deletion
    const confirmBtn = page.getByRole("button", { name: /delete/i }).last();
    await confirmBtn.click();

    // Tab should be closed
    await expect(tab).not.toBeVisible({ timeout: 3000 });
  });
});
