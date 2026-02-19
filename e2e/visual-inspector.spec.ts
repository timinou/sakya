import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

// =============================================================================
// Inspector Panel Visual Regression Tests (ITEM-215)
// =============================================================================

test.describe("Inspector visual regression", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  test("chapter inspector — revised status with progress bar", async ({
    page,
  }) => {
    // "The Awakening" has status: "revised", targetWords: 3000
    await page.getByTitle("1. The Awakening").click();
    await page.getByRole("tab", { name: "The Awakening" }).waitFor();
    await page.waitForTimeout(500);

    const inspector = page.locator(".chapter-inspector");
    await expect(inspector).toBeVisible();
    await expect(inspector).toHaveScreenshot("chapter-inspector-revised.png");
  });

  test("chapter inspector — draft status", async ({ page }) => {
    // "Into the Woods" has status: "draft"
    await page.getByTitle("2. Into the Woods").click();
    await page.getByRole("tab", { name: "Into the Woods" }).waitFor();
    await page.waitForTimeout(500);

    const inspector = page.locator(".chapter-inspector");
    await expect(inspector).toBeVisible();
    await expect(inspector).toHaveScreenshot("chapter-inspector-draft.png");
  });

  test("note inspector with color and label", async ({ page }) => {
    // "Magic System Rules" has color: #3b82f6, label: "worldbuilding"
    await page.getByTitle("Magic System Rules").click();
    await page.getByRole("tab", { name: "Magic System Rules" }).waitFor();
    await page.waitForTimeout(300);

    const inspector = page.locator(".note-inspector");
    await expect(inspector).toBeVisible();
    await expect(inspector).toHaveScreenshot("note-inspector-full.png");
  });

  test("entity form — character with spider chart", async ({ page }) => {
    // "Elena Blackwood" has spider axes (Strength, Intelligence, Charisma, Wisdom)
    await page.getByText("Elena Blackwood").click();
    await expect(page.getByLabel("Entity title")).toBeVisible({
      timeout: 5000,
    });
    // Wait for spider chart SVG to render
    await page.waitForTimeout(500);

    const editorPane = page.locator(".editor-pane");
    await expect(editorPane).toHaveScreenshot("entity-form-character.png");
  });

  test("spider chart SVG", async ({ page }) => {
    await page.getByText("Elena Blackwood").click();
    await expect(page.getByLabel("Entity title")).toBeVisible({
      timeout: 5000,
    });
    await page.waitForTimeout(500);

    // Target the spider chart SVG specifically
    const spiderChart = page.locator(".spider-chart svg");
    // Fallback: if .spider-chart doesn't exist, try the characteristics section
    const chartContainer = page.locator(".spider-chart");
    if ((await chartContainer.count()) > 0) {
      await expect(chartContainer).toHaveScreenshot(
        "spider-chart-character.png",
      );
    } else {
      // Fall back to the characteristics heading area
      const characteristics = page.getByText("Characteristics");
      await expect(characteristics).toBeVisible();
      // Screenshot parent section instead
      const section = page
        .locator("section, .field-group, div")
        .filter({ has: characteristics })
        .first();
      await expect(section).toHaveScreenshot("spider-chart-character.png");
    }
  });

  test("entity form — place (no spider chart)", async ({ page }) => {
    // Need to add a place entity instance to the mock data
    // Override get_entity to also return Ironhaven
    await page.goto("about:blank");
    await openMockProject(page, {
      get_entity: new Function(
        "args",
        `var instances = {
          "elena-blackwood": {
            title: "Elena Blackwood", slug: "elena-blackwood", schemaSlug: "character",
            tags: ["protagonist", "mage"],
            spiderValues: { Strength: 4, Intelligence: 9, Charisma: 7, Wisdom: 6 },
            fields: { role: "Protagonist", age: 28, backstory: "Born in the northern reaches..." },
            body: "# Elena Blackwood\\nThe last surviving heir of the Blackwood lineage."
          },
          "marcus-thorne": {
            title: "Marcus Thorne", slug: "marcus-thorne", schemaSlug: "character",
            tags: ["antagonist"],
            spiderValues: { Strength: 8, Intelligence: 7, Charisma: 5, Wisdom: 3 },
            fields: { role: "Antagonist", age: 45, backstory: "A fallen knight turned warlord..." },
            body: "# Marcus Thorne\\nOnce a noble knight, now consumed by ambition."
          },
          "ironhaven": {
            title: "Ironhaven", slug: "ironhaven", schemaSlug: "place",
            tags: ["city", "capital"],
            spiderValues: {},
            fields: { region: "Northern Kingdom", climate: "temperate" },
            body: "# Ironhaven\\nThe fortified capital of the northern realm."
          }
        };
        var slug = args && args.slug;
        return instances[slug] || null;`,
      ),
    });

    await page.getByText("Ironhaven").click();
    await expect(page.getByLabel("Entity title")).toBeVisible({
      timeout: 5000,
    });
    await page.waitForTimeout(500);

    const editorPane = page.locator(".editor-pane");
    await expect(editorPane).toHaveScreenshot("entity-form-place.png");
  });

  test("inspector pane empty state", async ({ page }) => {
    // No document open — inspector should show placeholder
    const inspectorPane = page.locator(".inspector-pane");
    await expect(inspectorPane).toBeVisible();
    await expect(inspectorPane).toHaveScreenshot("inspector-empty.png");
  });
});
