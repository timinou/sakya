import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Chapter Metadata Inspector", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Inspector shows chapter fields when chapter is active
  // ---------------------------------------------------------------------------

  test("opening a chapter shows inspector with status, POV, synopsis, and target fields", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Wait for content load
    await page.waitForTimeout(500);

    // Inspector should show chapter-specific fields
    const statusSelect = page.locator("#chapter-status");
    await expect(statusSelect).toBeVisible();

    const povSelect = page.locator("#chapter-pov");
    await expect(povSelect).toBeVisible();

    const synopsisTextarea = page.locator("#chapter-synopsis");
    await expect(synopsisTextarea).toBeVisible();

    const targetWordsInput = page.locator("#chapter-target-words");
    await expect(targetWordsInput).toBeVisible();
  });

  test("chapter status select shows the correct current status", async ({
    page,
  }) => {
    // The Awakening has status "revised"
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    const statusSelect = page.locator("#chapter-status");
    await expect(statusSelect).toHaveValue("revised");
  });

  test("chapter synopsis shows the correct content", async ({ page }) => {
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    const synopsisTextarea = page.locator("#chapter-synopsis");
    await expect(synopsisTextarea).toHaveValue(
      /Elena discovers her latent magical abilities/,
    );
  });

  test("chapter target words shows the correct value", async ({ page }) => {
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    const targetWordsInput = page.locator("#chapter-target-words");
    await expect(targetWordsInput).toHaveValue("3000");
  });

  // ---------------------------------------------------------------------------
  // Status change saves to backend
  // ---------------------------------------------------------------------------

  test("changing status dropdown calls save_chapter IPC", async ({ page }) => {
    // Open chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    await clearIpcCalls(page);

    // Change status to "final"
    const statusSelect = page.locator("#chapter-status");
    await statusSelect.selectOption("final");

    // Wait for save (may be synchronous or debounced)
    await page.waitForTimeout(500);

    // save_chapter should have been called
    const saveCalls = await getIpcCallsByCommand(page, "save_chapter");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);

    // Verify the status was changed to final
    const args = saveCalls[0].args as Record<string, unknown>;
    const chapter = args.chapter as Record<string, unknown>;
    expect(chapter.status).toBe("final");
  });

  // ---------------------------------------------------------------------------
  // Inspector empty state
  // ---------------------------------------------------------------------------

  test("inspector shows empty state when no tab is active", async ({
    page,
  }) => {
    // No chapter is opened, inspector should show placeholder
    await expect(page.getByText("No document selected")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Inspector switches on tab change
  // ---------------------------------------------------------------------------

  test("inspector updates when switching between chapter tabs", async ({
    page,
  }) => {
    // Open first chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(300);

    // Status should be "revised" for The Awakening
    await expect(page.locator("#chapter-status")).toHaveValue("revised");

    // Open second chapter
    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();
    await page.waitForTimeout(300);

    // Status should be "draft" for Into the Woods
    await expect(page.locator("#chapter-status")).toHaveValue("draft");
  });

  test("inspector shows NoteInspector when note tab is active", async ({
    page,
  }) => {
    // Open a note - should show NoteInspector instead of placeholder
    await page.getByTitle("Magic System Rules").click();
    await page.getByRole("tab", { name: "Magic System Rules" }).waitFor();
    await page.waitForTimeout(300);

    // Inspector should show the NoteInspector with "Open in Editor" button
    await expect(
      page.locator(".note-inspector"),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Open in Editor" }),
    ).toBeVisible();
  });
});
