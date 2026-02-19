import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";
import { setTheme } from "./utils/screenshots";

// =============================================================================
// AppShell Layout Visual Regression Tests (ITEM-213)
// =============================================================================

test.describe("AppShell visual regression", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("welcome card (no document open)", async ({ page }) => {
    const welcomeCard = page.locator(".welcome-card");
    await expect(welcomeCard).toBeVisible();
    await expect(welcomeCard).toHaveScreenshot("welcome-card.png");
  });

  test("3-pane layout with chapter open", async ({ page }) => {
    // Open a chapter to get the 3-pane layout populated
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    // Wait for editor content to render
    await page.locator('[contenteditable="true"]').waitFor({ state: "visible" });
    await page.waitForTimeout(500);

    await expect(page).toHaveScreenshot("appshell-3pane-chapter.png");
  });

  test("3-pane layout dark theme", async ({ page }) => {
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.locator('[contenteditable="true"]').waitFor({ state: "visible" });
    await page.waitForTimeout(500);

    await setTheme(page, "dark");
    await expect(page).toHaveScreenshot("appshell-3pane-dark.png");
  });

  test("toolbar buttons", async ({ page }) => {
    const toolbar = page.locator(".toolbar");
    await expect(toolbar).toBeVisible();
    await expect(toolbar).toHaveScreenshot("toolbar-default.png");
  });

  test("status bar with word count", async ({ page }) => {
    // Open a chapter so status bar shows word count
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    const statusBar = page.locator(".status-bar");
    await expect(statusBar).toBeVisible();
    await expect(statusBar).toHaveScreenshot("statusbar-chapter.png");
  });

  test("distraction-free mode", async ({ page }) => {
    // Open a chapter first for content
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.locator('[contenteditable="true"]').waitFor({ state: "visible" });
    await page.waitForTimeout(500);

    // Enter distraction-free mode
    await page.keyboard.press("Control+Shift+F");
    await expect(page.locator(".app-shell")).toHaveClass(/distraction-free/);
    // Wait for transition to complete
    await page.waitForTimeout(500);

    await expect(page).toHaveScreenshot("appshell-distraction-free.png");
  });

  test("multiple tabs open", async ({ page }) => {
    // Open three chapters as tabs
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    await page.getByTitle("3. The Siege").click();
    await page.getByRole("tab", { name: "The Siege" }).waitFor();

    const editorTabs = page.locator(".editor-tabs");
    await expect(editorTabs).toBeVisible();
    await expect(editorTabs).toHaveScreenshot("editor-tabs-multiple.png");
  });

  test("binder with all sections", async ({ page }) => {
    const binderPane = page.locator(".binder-pane");
    await expect(binderPane).toBeVisible();
    // Wait for all sections to render (manuscript, entities, notes)
    await expect(page.getByText("Manuscript")).toBeVisible();
    await expect(page.getByText("Notes")).toBeVisible();

    await expect(binderPane).toHaveScreenshot("binder-all-sections.png");
  });
});
