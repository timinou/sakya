import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

/**
 * NavigationStore E2E Tests (IMP-008)
 *
 * Tests 5 behavioral gaps that only NavigationStore exercises:
 * 1. Tab click restores binder highlight (switchToTab)
 * 2. Cross-type selection clearing (navigateTo across types)
 * 3. Close tab restores fallback binder highlight
 * 4. Full cross-type round-trip
 * 5. Corkboard "Open in Tab" integration
 */

test.describe("NavigationStore - Tab switch restores binder selection", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("tab click restores chapter binder highlight", async ({ page }) => {
    // Open two chapters
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    // Second chapter should be highlighted, first should not
    await expect(page.getByTitle("2. Into the Woods")).toHaveClass(
      /selected|active/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );

    // Click the first tab — binder highlight should restore to first chapter
    await page.getByRole("tab", { name: "The Awakening" }).click();

    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );
    await expect(page.getByTitle("2. Into the Woods")).not.toHaveClass(
      /selected/,
    );
  });

  test("tab click restores note binder highlight", async ({ page }) => {
    // Open two notes
    await page.getByTitle("Magic System Rules").click();
    await page.getByRole("tab", { name: "Magic System Rules" }).waitFor();

    await page.getByTitle("Plot Outline").click();
    await page.getByRole("tab", { name: "Plot Outline" }).waitFor();

    // Second note should be highlighted
    await expect(page.getByTitle("Plot Outline")).toHaveClass(
      /selected|active/,
    );
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /selected/,
    );

    // Click the first note's tab — binder highlight should restore
    await page.getByRole("tab", { name: "Magic System Rules" }).click();

    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );
    await expect(page.getByTitle("Plot Outline")).not.toHaveClass(/selected/);
  });
});

test.describe("NavigationStore - Cross-type selection clearing", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("chapter -> note clears chapter highlight", async ({ page }) => {
    // Select a chapter
    await page.getByTitle("1. The Awakening").click();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );

    // Now select a note
    await page.getByTitle("Magic System Rules").click();
    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );

    // Chapter binder item should no longer be highlighted
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /active/,
    );
  });

  test("note -> chapter clears note highlight", async ({ page }) => {
    // Select a note
    await page.getByTitle("Magic System Rules").click();
    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );

    // Now select a chapter
    await page.getByTitle("1. The Awakening").click();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );

    // Note binder item should no longer be highlighted
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /active/,
    );
  });

  test("chapter -> entity clears chapter highlight", async ({ page }) => {
    // Select a chapter
    await page.getByTitle("1. The Awakening").click();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );

    // Now select an entity
    await page.getByTitle("Elena Blackwood").click();
    await page.getByRole("tab", { name: "Elena Blackwood" }).waitFor();

    // Chapter binder item should no longer be highlighted
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /active/,
    );
  });

  test("full round-trip: chapter -> note -> entity -> chapter", async ({
    page,
  }) => {
    // Step 1: Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );

    // Step 2: Switch to a note — chapter cleared
    await page.getByTitle("Magic System Rules").click();
    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );

    // Step 3: Switch to an entity — note cleared
    await page.getByTitle("Elena Blackwood").click();
    await page.getByRole("tab", { name: "Elena Blackwood" }).waitFor();
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /active/,
    );

    // Step 4: Switch back to chapter — entity tab is no longer active
    await page.getByTitle("1. The Awakening").click();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );
    const entityTab = page.getByRole("tab", { name: "Elena Blackwood" });
    await expect(entityTab).toHaveAttribute("aria-selected", "false");

    // Chapter tab should be active
    const chapterTab = page.getByRole("tab", { name: "The Awakening" });
    await expect(chapterTab).toHaveAttribute("aria-selected", "true");
  });
});

test.describe("NavigationStore - Close tab selection management", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("Ctrl+W clears selection for closed type", async ({ page }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );

    // Close via Ctrl+W
    await page.keyboard.press("Control+w");

    // Tab gone
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).not.toBeVisible();

    // Binder highlight should be cleared
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /active/,
    );
  });

  test("close button clears selection for closed type", async ({ page }) => {
    // Open a note
    await page.getByTitle("Magic System Rules").click();
    const noteTab = page.getByRole("tab", { name: "Magic System Rules" });
    await noteTab.waitFor();
    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );

    // Close via X button (hover first to show the button)
    await noteTab.hover();
    await noteTab.getByTitle("Close tab").click();

    // Tab gone
    await expect(noteTab).not.toBeVisible();

    // Binder highlight should be cleared
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("Magic System Rules")).not.toHaveClass(
      /active/,
    );
  });

  test("close tab restores fallback binder highlight", async ({ page }) => {
    // Open two chapters
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();

    // Second is active and highlighted
    await expect(page.getByTitle("2. Into the Woods")).toHaveClass(
      /selected|active/,
    );

    // Close active tab via Ctrl+W
    await page.keyboard.press("Control+w");

    // "Into the Woods" tab is gone
    await expect(
      page.getByRole("tab", { name: "Into the Woods" }),
    ).not.toBeVisible();

    // The Awakening tab should now be active
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).toHaveAttribute("aria-selected", "true");

    // And its binder item should be highlighted (fallback restore)
    await expect(page.getByTitle("1. The Awakening")).toHaveClass(
      /selected|active/,
    );
  });

  test("close last tab clears all selections", async ({ page }) => {
    // Open a single chapter
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();

    // Close it
    await page.keyboard.press("Control+w");

    // Tab gone, welcome card shown
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).not.toBeVisible();
    await expect(page.locator(".welcome-card")).toBeVisible();

    // No binder items highlighted
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /selected/,
    );
    await expect(page.getByTitle("1. The Awakening")).not.toHaveClass(
      /active/,
    );
  });
});

test.describe("NavigationStore - Corkboard integration", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test('"Open in Editor" opens note tab from corkboard', async ({ page }) => {
    // Switch to corkboard view
    await page.getByRole("button", { name: "Corkboard" }).click();
    await expect(page.locator(".corkboard")).toBeVisible();

    // Double-click a note card to enter edit mode
    const card = page
      .locator(".note-card")
      .filter({ hasText: "Magic System Rules" });
    await card.dblclick();

    // Should be in edit mode with "Open in Editor" button visible
    await expect(page.locator(".note-card.editing")).toBeVisible();
    // Use specific CSS selector — multiple elements match the generic role
    const openBtn = page.locator(".note-card.editing .open-in-editor-btn");
    await expect(openBtn).toBeVisible();

    // Click "Open in Editor" — triggers navigateTo + setViewMode('editor')
    await openBtn.click();

    // Should switch back to editor mode and open a tab
    // (EditorArea only renders in editor mode, not corkboard)
    await expect(
      page.getByRole("tab", { name: "Magic System Rules" }),
    ).toBeVisible();
    await expect(
      page.getByRole("tab", { name: "Magic System Rules" }),
    ).toHaveAttribute("aria-selected", "true");

    // Note binder item should be highlighted
    await expect(page.getByTitle("Magic System Rules")).toHaveClass(
      /selected|active/,
    );
  });

  test("double-click existing tab switches to it from corkboard", async ({
    page,
  }) => {
    // Open a note in editor mode (creates a tab)
    await page.getByTitle("Magic System Rules").click();
    await page.getByRole("tab", { name: "Magic System Rules" }).waitFor();

    // Open a chapter too (so note tab becomes inactive)
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await expect(
      page.getByRole("tab", { name: "The Awakening" }),
    ).toHaveAttribute("aria-selected", "true");

    // Switch to corkboard view (EditorArea/tabs hidden, corkboard shown)
    await page.getByRole("button", { name: "Corkboard" }).click();
    await expect(page.locator(".corkboard")).toBeVisible();

    // Double-click the same note card — since it already has a tab,
    // handleEditStart calls navigationStore.switchToTab + setViewMode('editor')
    const card = page
      .locator(".note-card")
      .filter({ hasText: "Magic System Rules" });
    await card.dblclick();

    // Should switch back to editor mode with the note tab active
    const noteTab = page.getByRole("tab", { name: "Magic System Rules" });
    await expect(noteTab).toBeVisible();
    await expect(noteTab).toHaveAttribute("aria-selected", "true");
  });
});
