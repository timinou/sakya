import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

test.describe("Note Editing in Tabs", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // Opening notes in tabs
  // ---------------------------------------------------------------------------

  test("clicking note in binder opens a tab with note title", async ({
    page,
  }) => {
    // Click on a note in the binder
    await page.getByTitle("Magic System Rules").click();

    // A tab should appear with the note title
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(tabList).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "Magic System Rules" }),
    ).toBeVisible();
  });

  test("note content loads from get_note IPC call", async ({ page }) => {
    await clearIpcCalls(page);

    // Click on a note
    await page.getByTitle("Magic System Rules").click();

    // Wait for the tab and content to load
    await page
      .getByRole("tab", { name: "Magic System Rules" })
      .waitFor();
    await page.waitForTimeout(500);

    // get_note should have been called
    const noteCalls = await getIpcCallsByCommand(page, "get_note");
    const magicCall = noteCalls.find(
      (c) => (c.args as Record<string, unknown>)?.slug === "magic-system",
    );
    expect(magicCall).toBeDefined();
  });

  test("close note tab removes it", async ({ page }) => {
    // Open a note tab
    await page.getByTitle("Magic System Rules").click();
    await page
      .getByRole("tab", { name: "Magic System Rules" })
      .waitFor();

    // Close the tab via Ctrl+W
    await page.keyboard.press("Control+w");

    // Tab should be gone
    await expect(
      page.getByRole("tab", { name: "Magic System Rules" }),
    ).not.toBeVisible();

    // Welcome card should return
    await expect(page.locator(".welcome-card")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Multiple note tabs
  // ---------------------------------------------------------------------------

  test("opening multiple notes creates multiple tabs", async ({ page }) => {
    // Open first note
    await page.getByTitle("Magic System Rules").click();
    await page
      .getByRole("tab", { name: "Magic System Rules" })
      .waitFor();

    // Open second note
    await page.getByTitle("Plot Outline").click();
    await page.getByRole("tab", { name: "Plot Outline" }).waitFor();

    // Both tabs should exist
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(
      tabList.getByRole("tab", { name: "Magic System Rules" }),
    ).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "Plot Outline" }),
    ).toBeVisible();

    // Second tab should be active
    await expect(
      tabList.getByRole("tab", { name: "Plot Outline" }),
    ).toHaveAttribute("aria-selected", "true");
  });

  // ---------------------------------------------------------------------------
  // Mixed note and chapter tabs
  // ---------------------------------------------------------------------------

  test("note and chapter tabs coexist", async ({ page }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page
      .getByRole("tab", { name: "The Awakening" })
      .waitFor();

    // Open a note
    await page.getByTitle("Magic System Rules").click();
    await page
      .getByRole("tab", { name: "Magic System Rules" })
      .waitFor();

    // Both tabs should exist
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(
      tabList.getByRole("tab", { name: "The Awakening" }),
    ).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "Magic System Rules" }),
    ).toBeVisible();
  });

  test("clicking same note again does not duplicate tab", async ({ page }) => {
    // Open note
    await page.getByTitle("Magic System Rules").click();
    await page
      .getByRole("tab", { name: "Magic System Rules" })
      .waitFor();

    // Click same note again
    await page.getByTitle("Magic System Rules").click();

    // Should still only have one tab
    const tabs = page
      .getByRole("tablist", { name: "Open documents" })
      .getByRole("tab");
    await expect(tabs).toHaveCount(1);
  });
});
