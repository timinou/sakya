/**
 * Project Switch E2E Tests (ITEM-195)
 *
 * Verifies that rapid project switching works correctly with StaleGuard race
 * condition protection and project-switch reset orchestration.
 *
 * Strategy: `addInitScript` is called only once (via `openMockProject`).
 * To simulate a second project switch we install path-conditional mock handlers
 * up front using `new Function(...)` with inlined JSON, then call
 * `projectState.open('/mock/project-b')` via `page.evaluate`.
 */

import { test, expect } from "@playwright/test";
import {
  setupDefaultTauriMocks,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";
import { takeStepScreenshot } from "./utils/screenshots";

// ---------------------------------------------------------------------------
// Project fixture data
// ---------------------------------------------------------------------------

/** Project A — the default project opened via the launcher button */
const PROJECT_A_PATH = "/mock/project/path"; // path the dialog mock returns
const PROJECT_A_MANIFEST = {
  name: "Project Alpha",
  version: "0.1.0",
  author: null,
  description: null,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};
const PROJECT_A_CHAPTERS = {
  "the-awakening": {
    slug: "the-awakening",
    frontmatter: {
      slug: "the-awakening",
      title: "The Awakening",
      status: "revised",
      pov: "Elena",
      synopsis: "Elena discovers her latent magical abilities.",
      targetWords: 3000,
      order: 0,
    },
    body: "The morning light filtered through the cracked window pane.",
  },
  "into-the-woods": {
    slug: "into-the-woods",
    frontmatter: {
      slug: "into-the-woods",
      title: "Into the Woods",
      status: "draft",
      pov: "Elena",
      synopsis: "Elena ventures into the Whispering Woods.",
      targetWords: 4000,
      order: 1,
    },
    body: "The trees stood like sentinels at the forest's edge.",
  },
  "the-siege": {
    slug: "the-siege",
    frontmatter: {
      slug: "the-siege",
      title: "The Siege",
      status: "draft",
      pov: "Marcus",
      synopsis: "Marcus leads his forces.",
      targetWords: 5000,
      order: 2,
    },
    body: "The war drums echoed across the valley.",
  },
};
const PROJECT_A_MANUSCRIPT_CONFIG = {
  chapters: ["the-awakening", "into-the-woods", "the-siege"],
};

/** Project B — the project we switch to programmatically */
const PROJECT_B_PATH = "/mock/project-b";
const PROJECT_B_MANIFEST = {
  name: "Project Beta",
  version: "0.1.0",
  author: null,
  description: null,
  createdAt: "2026-02-01T00:00:00Z",
  updatedAt: "2026-02-01T00:00:00Z",
};
const PROJECT_B_CHAPTERS = {
  "the-beginning": {
    slug: "the-beginning",
    frontmatter: {
      slug: "the-beginning",
      title: "The Beginning",
      status: "draft",
      pov: "Zara",
      synopsis: "Zara sets out on her journey.",
      targetWords: 2000,
      order: 0,
    },
    body: "In the city of Emberveil, Zara counted her coins.",
  },
};
const PROJECT_B_MANUSCRIPT_CONFIG = {
  chapters: ["the-beginning"],
};
const PROJECT_B_NOTES_CONFIG = {
  notes: [
    {
      slug: "worldbuilding-notes",
      title: "Worldbuilding Notes",
      color: "#e11d48",
      label: "worldbuilding",
      position: { x: 30, y: 40 },
    },
  ],
};
const PROJECT_B_SCHEMAS = [
  {
    name: "Faction",
    entityType: "faction",
    fieldCount: 1,
    axisCount: 0,
  },
];
const PROJECT_B_ENTITIES: Record<string, unknown[]> = {
  faction: [
    {
      title: "The Iron Guild",
      slug: "the-iron-guild",
      schemaType: "faction",
      tags: ["merchant", "power"],
    },
  ],
};

// ---------------------------------------------------------------------------
// Helper: build path-conditional mocks for all store commands
// ---------------------------------------------------------------------------

/**
 * Returns overrides for `setupDefaultTauriMocks` that make IPC handlers return
 * Project-A data for PROJECT_A_PATH and Project-B data for PROJECT_B_PATH.
 * All functions are `new Function(...)` with inlined JSON (no outer scope refs).
 */
function buildDualProjectMocks() {
  return {
    // open_project — returns manifest based on path
    open_project: new Function(
      "args",
      `var path = args && args.path;
       var manifestA = ${JSON.stringify(PROJECT_A_MANIFEST)};
       var manifestB = ${JSON.stringify(PROJECT_B_MANIFEST)};
       if (path === ${JSON.stringify(PROJECT_B_PATH)}) return manifestB;
       return manifestA;`,
    ),

    // get_manuscript_config — config changes per project
    get_manuscript_config: new Function(
      "args",
      `var path = args && args.projectPath;
       var configA = ${JSON.stringify(PROJECT_A_MANUSCRIPT_CONFIG)};
       var configB = ${JSON.stringify(PROJECT_B_MANUSCRIPT_CONFIG)};
       if (path === ${JSON.stringify(PROJECT_B_PATH)}) return configB;
       return configA;`,
    ),

    // get_chapter — different chapter sets per project
    get_chapter: new Function(
      "args",
      `var path = args && args.projectPath;
       var slug = args && args.slug;
       var chaptersA = ${JSON.stringify(PROJECT_A_CHAPTERS)};
       var chaptersB = ${JSON.stringify(PROJECT_B_CHAPTERS)};
       var map = (path === ${JSON.stringify(PROJECT_B_PATH)}) ? chaptersB : chaptersA;
       return map[slug] || null;`,
    ),

    // get_notes_config — different notes per project
    get_notes_config: new Function(
      "args",
      `var path = args && args.projectPath;
       var defaultNotes = {
         notes: [
           { slug: "magic-system", title: "Magic System Rules", color: "#3b82f6", label: "worldbuilding", position: { x: 20, y: 30 } },
           { slug: "plot-outline", title: "Plot Outline", color: "#22c55e", label: "planning", position: { x: 60, y: 25 } }
         ]
       };
       var notesB = ${JSON.stringify(PROJECT_B_NOTES_CONFIG)};
       if (path === ${JSON.stringify(PROJECT_B_PATH)}) return notesB;
       return defaultNotes;`,
    ),

    // list_schemas — Project B has a different schema set
    list_schemas: new Function(
      "args",
      `var path = args && args.projectPath;
       var schemasA = [
         { name: "Character", entityType: "character", fieldCount: 3, axisCount: 4 },
         { name: "Place", entityType: "place", fieldCount: 2, axisCount: 0 }
       ];
       var schemasB = ${JSON.stringify(PROJECT_B_SCHEMAS)};
       if (path === ${JSON.stringify(PROJECT_B_PATH)}) return schemasB;
       return schemasA;`,
    ),

    // list_entities — different entities per project
    list_entities: new Function(
      "args",
      `var path = args && args.projectPath;
       var schemaType = args && args.schemaType;
       var entitiesA = {
         character: [
           { title: "Elena Blackwood", slug: "elena-blackwood", schemaType: "character", tags: ["protagonist"] },
           { title: "Marcus Thorne", slug: "marcus-thorne", schemaType: "character", tags: ["antagonist"] }
         ],
         place: [
           { title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: ["city"] }
         ]
       };
       var entitiesB = ${JSON.stringify(PROJECT_B_ENTITIES)};
       var map = (path === ${JSON.stringify(PROJECT_B_PATH)}) ? entitiesB : entitiesA;
       return map[schemaType] || [];`,
    ),
  };
}

// ---------------------------------------------------------------------------
// Helper: trigger project switch in browser context
// ---------------------------------------------------------------------------

/**
 * Calls `projectState.open(path)` in the browser context to simulate a
 * project switch. Waits for the new project manifest to be reflected in the
 * store before resolving.
 */
async function switchToProject(
  page: import("@playwright/test").Page,
  path: string,
  expectedProjectName: string,
): Promise<void> {
  await page.evaluate(async (switchPath) => {
    const { projectState } = await import("/src/lib/stores/index.ts");
    await projectState.open(switchPath);
  }, path);

  // Wait until the toolbar reflects the new project name
  await page
    .getByText(expectedProjectName)
    .waitFor({ state: "visible", timeout: 10000 });
}

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

const PREFIX = "item195";

test.describe("Project Switch — store reset and data isolation (ITEM-195)", () => {
  // =========================================================================
  // Test 1: Binder shows new project's chapters after switch, not old ones
  // =========================================================================
  test.describe("binder chapter list is replaced on project switch", () => {
    test("Project A chapters disappear; Project B chapters appear after switch", async ({
      page,
    }) => {
      // Set up mocks that return different data based on path, BEFORE the page loads
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page
        .getByRole("button", { name: /open project/i })
        .click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });

      // Confirm Project A loaded
      await expect(page.getByText("Project Alpha")).toBeVisible();
      await expect(page.getByTitle("1. The Awakening")).toBeVisible();
      await expect(page.getByTitle("2. Into the Woods")).toBeVisible();
      await expect(page.getByTitle("3. The Siege")).toBeVisible();

      await takeStepScreenshot(page, PREFIX, "test1-project-a-loaded");

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");

      // Wait for new project data to load
      await page.waitForTimeout(1500);

      await takeStepScreenshot(page, PREFIX, "test1-project-b-loaded");

      // Project B chapters should appear
      await expect(page.getByTitle("1. The Beginning")).toBeVisible();

      // Project A chapters must NOT be visible
      await expect(page.getByTitle("1. The Awakening")).not.toBeVisible();
      await expect(page.getByTitle("2. Into the Woods")).not.toBeVisible();
      await expect(page.getByTitle("3. The Siege")).not.toBeVisible();

      // Toolbar should show Project B's name
      await expect(page.getByText("Project Beta")).toBeVisible();
    });

    test("chapter count badge updates correctly after switch", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      // Project A has 3 chapters
      const manuscriptSectionA = page.locator(".section").filter({ hasText: "Manuscript" });
      await expect(manuscriptSectionA.locator(".section-count")).toHaveText("3");

      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      // Project B has 1 chapter
      const manuscriptSectionB = page.locator(".section").filter({ hasText: "Manuscript" });
      await expect(manuscriptSectionB.locator(".section-count")).toHaveText("1");

      await takeStepScreenshot(page, PREFIX, "test1-chapter-count-updated");
    });
  });

  // =========================================================================
  // Test 2: Editor tabs cleared on project switch
  // =========================================================================
  test.describe("editor tabs are cleared when switching projects", () => {
    test("an open chapter tab from Project A is gone after switching to Project B", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });

      // Open a chapter tab in Project A
      await page.getByTitle("1. The Awakening").click();
      const tabList = page.getByRole("tablist", { name: "Open documents" });
      await expect(tabList.getByRole("tab", { name: "The Awakening" })).toBeVisible();

      await takeStepScreenshot(page, PREFIX, "test2-tab-open-project-a");

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      await takeStepScreenshot(page, PREFIX, "test2-after-switch-to-b");

      // The Project A tab should no longer exist
      await expect(tabList.getByRole("tab", { name: "The Awakening" })).not.toBeVisible();

      // The editor should show the empty/welcome state (no active document)
      await expect(page.locator(".welcome-card")).toBeVisible();
    });

    test("multiple tabs from Project A are all cleared on switch", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });

      // Open two tabs in Project A
      await page.getByTitle("1. The Awakening").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Awakening" })
        .waitFor();
      await page.getByTitle("2. Into the Woods").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "Into the Woods" })
        .waitFor();

      // Confirm two tabs are open
      const tabList = page.getByRole("tablist", { name: "Open documents" });
      const tabsBefore = tabList.getByRole("tab");
      await expect(tabsBefore).toHaveCount(2);

      await takeStepScreenshot(page, PREFIX, "test2-two-tabs-open");

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      await takeStepScreenshot(page, PREFIX, "test2-after-switch-two-tabs-gone");

      // All tabs from Project A should be gone — welcome card shown
      await expect(page.locator(".welcome-card")).toBeVisible();

      // Verify via store that editorState.tabs is empty
      const editorStoreState = await page.evaluate(async () => {
        const { editorState } = await import("/src/lib/stores/index.ts");
        return {
          tabCount: editorState.tabs.length,
          activeTabId: editorState.activeTabId,
        };
      });
      expect(editorStoreState.tabCount).toBe(0);
      expect(editorStoreState.activeTabId).toBeNull();
    });

    test("tabs opened in Project B do not carry Project A identity", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });

      // Open a tab in Project A
      await page.getByTitle("1. The Awakening").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Awakening" })
        .waitFor();

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      // Open the only chapter in Project B
      await page.getByTitle("1. The Beginning").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Beginning" })
        .waitFor();

      await takeStepScreenshot(page, PREFIX, "test2-project-b-tab-opened");

      // Only Project B's tab should be visible — no Project A tab present
      const tabList = page.getByRole("tablist", { name: "Open documents" });
      await expect(tabList.getByRole("tab", { name: "The Beginning" })).toBeVisible();
      await expect(
        tabList.getByRole("tab", { name: "The Awakening" }),
      ).not.toBeVisible();

      const tabCount = await tabList.getByRole("tab").count();
      expect(tabCount).toBe(1);
    });
  });

  // =========================================================================
  // Test 3: Data isolation — entity sections reflect new project after switch
  // =========================================================================
  test.describe("entity data is isolated between projects", () => {
    test("Project A entities are replaced by Project B entities after switch", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1500);

      // Project A should show Characters and Places sections
      await expect(page.getByText("Characters")).toBeVisible();
      await expect(page.getByText("Places")).toBeVisible();

      // Project A entities should be present
      await expect(page.getByTitle("Elena Blackwood")).toBeVisible();
      await expect(page.getByTitle("Marcus Thorne")).toBeVisible();

      await takeStepScreenshot(page, PREFIX, "test3-project-a-entities");

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(2000);

      await takeStepScreenshot(page, PREFIX, "test3-project-b-entities");

      // Project A entities MUST NOT be visible
      await expect(page.getByTitle("Elena Blackwood")).not.toBeVisible();
      await expect(page.getByTitle("Marcus Thorne")).not.toBeVisible();

      // Project A entity sections should be gone (Characters, Places)
      await expect(page.getByText("Characters")).not.toBeVisible();
      await expect(page.getByText("Places")).not.toBeVisible();

      // Project B entity section (Factions) should appear with its entity
      await expect(page.getByText("Factions")).toBeVisible();
      await expect(page.getByTitle("The Iron Guild")).toBeVisible();
    });

    test("entity schemas are reset: schemasLoadedPath updates to Project B after switch", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1500);

      // Verify Project A state in store
      const stateA = await page.evaluate(async () => {
        const { entityStore } = await import("/src/lib/stores/index.ts");
        return {
          schemasLoadedPath: entityStore.schemasLoadedPath,
          schemaCount: entityStore.schemaSummaries.length,
        };
      });
      expect(stateA.schemasLoadedPath).toBe(PROJECT_A_PATH);
      expect(stateA.schemaCount).toBe(2); // character + place

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(2000);

      // Verify Project B state in store
      const stateB = await page.evaluate(async () => {
        const { entityStore } = await import("/src/lib/stores/index.ts");
        return {
          schemasLoadedPath: entityStore.schemasLoadedPath,
          schemaCount: entityStore.schemaSummaries.length,
          entityTypes: Object.keys(entityStore.entitiesByType),
        };
      });
      expect(stateB.schemasLoadedPath).toBe(PROJECT_B_PATH);
      expect(stateB.schemaCount).toBe(1); // faction only

      // Old entity type keys from Project A should not be present
      expect(stateB.entityTypes).not.toContain("character");
      expect(stateB.entityTypes).not.toContain("place");

      await takeStepScreenshot(page, PREFIX, "test3-store-path-verified");
    });
  });

  // =========================================================================
  // Test 4: No stale IPC writes after a rapid second switch (StaleGuard)
  // =========================================================================
  test.describe("StaleGuard prevents stale data from landing after rapid switches", () => {
    test("rapid double switch: only the last project's data is visible", async ({
      page,
    }) => {
      // Build mocks that support 3 project paths:
      //   PROJECT_A_PATH => Project Alpha (3 chapters)
      //   PROJECT_B_PATH => Project Beta  (1 chapter)
      //   /mock/project-c => Project Gamma (1 chapter: "The End")
      const PROJECT_C_PATH = "/mock/project-c";

      const rapidSwitchOverrides = {
        open_project: new Function(
          "args",
          `var path = args && args.path;
           if (path === "/mock/project-b") {
             return { name: "Project Beta", version: "0.1.0", author: null, description: null, createdAt: "2026-02-01T00:00:00Z", updatedAt: "2026-02-01T00:00:00Z" };
           }
           if (path === "/mock/project-c") {
             return { name: "Project Gamma", version: "0.1.0", author: null, description: null, createdAt: "2026-03-01T00:00:00Z", updatedAt: "2026-03-01T00:00:00Z" };
           }
           return { name: "Project Alpha", version: "0.1.0", author: null, description: null, createdAt: "2026-01-01T00:00:00Z", updatedAt: "2026-01-01T00:00:00Z" };`,
        ),
        get_manuscript_config: new Function(
          "args",
          `var path = args && args.projectPath;
           if (path === "/mock/project-b") return { chapters: ["the-beginning"] };
           if (path === "/mock/project-c") return { chapters: ["the-end"] };
           return { chapters: ["the-awakening", "into-the-woods", "the-siege"] };`,
        ),
        get_chapter: new Function(
          "args",
          `var path = args && args.projectPath;
           var slug = args && args.slug;
           var chaptersA = ${JSON.stringify(PROJECT_A_CHAPTERS)};
           var chaptersB = ${JSON.stringify(PROJECT_B_CHAPTERS)};
           var chaptersC = { "the-end": { slug: "the-end", frontmatter: { slug: "the-end", title: "The End", status: "draft", pov: null, synopsis: null, targetWords: null, order: 0 }, body: "And so it ended." } };
           if (path === "/mock/project-b") return chaptersB[slug] || null;
           if (path === "/mock/project-c") return chaptersC[slug] || null;
           return chaptersA[slug] || null;`,
        ),
        list_schemas: new Function(
          "args",
          `var path = args && args.projectPath;
           if (path === "/mock/project-b") return [{ name: "Faction", entityType: "faction", fieldCount: 1, axisCount: 0 }];
           if (path === "/mock/project-c") return [];
           return [
             { name: "Character", entityType: "character", fieldCount: 3, axisCount: 4 },
             { name: "Place", entityType: "place", fieldCount: 2, axisCount: 0 }
           ];`,
        ),
        list_entities: new Function(
          "args",
          `var path = args && args.projectPath;
           var schemaType = args && args.schemaType;
           var entitiesA = {
             character: [{ title: "Elena Blackwood", slug: "elena-blackwood", schemaType: "character", tags: [] }],
             place: [{ title: "Ironhaven", slug: "ironhaven", schemaType: "place", tags: [] }]
           };
           var entitiesB = { faction: [{ title: "The Iron Guild", slug: "the-iron-guild", schemaType: "faction", tags: [] }] };
           if (path === "/mock/project-b") return entitiesB[schemaType] || [];
           if (path === "/mock/project-c") return [];
           return entitiesA[schemaType] || [];`,
        ),
      };

      await setupDefaultTauriMocks(page, rapidSwitchOverrides);
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      await takeStepScreenshot(page, PREFIX, "test4-project-a-initial");

      // Fire two project switches back-to-back (no await between them)
      // to stress the StaleGuard — the second one should win.
      await page.evaluate(
        async ([pathB, pathC]) => {
          const { projectState } = await import("/src/lib/stores/index.ts");
          // Start both opens concurrently — only the second should land
          const openB = projectState.open(pathB);
          const openC = projectState.open(pathC);
          await Promise.all([openB, openC]).catch(() => {});
        },
        [PROJECT_B_PATH, PROJECT_C_PATH],
      );

      // Wait for Project Gamma to settle
      await page.getByText("Project Gamma").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1500);

      await takeStepScreenshot(page, PREFIX, "test4-after-rapid-double-switch");

      // Only Project Gamma data should be visible — Project B chapters must not appear
      await expect(page.getByTitle("1. The End")).toBeVisible();
      await expect(page.getByTitle("1. The Beginning")).not.toBeVisible();
      await expect(page.getByTitle("1. The Awakening")).not.toBeVisible();

      // Verify store projectPath is Project C, not Project B or A
      const storePath = await page.evaluate(async () => {
        const { projectState } = await import("/src/lib/stores/index.ts");
        return projectState.projectPath;
      });
      expect(storePath).toBe(PROJECT_C_PATH);
    });
  });

  // =========================================================================
  // Test 5: Active chapter selection (manuscriptStore.activeChapterSlug) is
  //         cleared on switch so no stale $effect re-opens a closed tab
  // =========================================================================
  test.describe("active selection state is cleared on project switch", () => {
    test("activeChapterSlug is null after switching projects", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });

      // Select a chapter to set activeChapterSlug
      await page.getByTitle("1. The Awakening").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Awakening" })
        .waitFor();

      // Verify selection before switch
      const slugBefore = await page.evaluate(async () => {
        const { manuscriptStore } = await import("/src/lib/stores/index.ts");
        return manuscriptStore.activeChapterSlug;
      });
      expect(slugBefore).toBe("the-awakening");

      // Switch projects
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      // activeChapterSlug must be null (reset cleared it)
      const slugAfter = await page.evaluate(async () => {
        const { manuscriptStore } = await import("/src/lib/stores/index.ts");
        return manuscriptStore.activeChapterSlug;
      });
      expect(slugAfter).toBeNull();

      await takeStepScreenshot(page, PREFIX, "test5-active-slug-cleared");
    });

    test("activeNoteSlug is null after switching projects", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      // Select a note in Project A by calling selectNote directly
      await page.evaluate(async () => {
        const { notesStore } = await import("/src/lib/stores/index.ts");
        notesStore.selectNote("magic-system");
      });

      const slugBefore = await page.evaluate(async () => {
        const { notesStore } = await import("/src/lib/stores/index.ts");
        return notesStore.activeNoteSlug;
      });
      expect(slugBefore).toBe("magic-system");

      // Switch projects
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1000);

      const slugAfter = await page.evaluate(async () => {
        const { notesStore } = await import("/src/lib/stores/index.ts");
        return notesStore.activeNoteSlug;
      });
      expect(slugAfter).toBeNull();
    });

    test("currentEntity is null in entityStore after switching projects", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1500);

      // Click on an entity to load it
      await page.getByTitle("Elena Blackwood").click();
      await expect(page.getByLabel("Entity title")).toBeVisible({ timeout: 5000 });

      // currentEntity should be set
      const entityBefore = await page.evaluate(async () => {
        const { entityStore } = await import("/src/lib/stores/index.ts");
        return entityStore.currentEntity?.slug ?? null;
      });
      expect(entityBefore).toBe("elena-blackwood");

      // Switch projects
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1000);

      // currentEntity should be cleared
      const entityAfter = await page.evaluate(async () => {
        const { entityStore } = await import("/src/lib/stores/index.ts");
        return entityStore.currentEntity;
      });
      expect(entityAfter).toBeNull();

      await takeStepScreenshot(page, PREFIX, "test5-current-entity-cleared");
    });
  });

  // =========================================================================
  // Test 6: IPC calls are directed to the new project path after switch
  // =========================================================================
  test.describe("IPC calls target the new project path after switch", () => {
    test("get_manuscript_config is called with Project B's path after switch", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      // Clear calls to isolate post-switch calls
      await clearIpcCalls(page);

      // Switch to Project B
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      // All get_manuscript_config calls after switch should use Project B's path
      const configCalls = await getIpcCallsByCommand(page, "get_manuscript_config");
      expect(configCalls.length).toBeGreaterThanOrEqual(1);
      for (const call of configCalls) {
        const args = call.args as Record<string, unknown> | undefined;
        expect(args?.projectPath).toBe(PROJECT_B_PATH);
      }

      await takeStepScreenshot(page, PREFIX, "test6-ipc-calls-targeted");
    });

    test("no IPC calls use Project A's path after switching to Project B", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      await clearIpcCalls(page);

      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(2000);

      // Audit all IPC calls — none should reference Project A's path
      const allCalls = await page.evaluate(
        () => (window as unknown as Record<string, unknown>).__TAURI_IPC_CALLS__ ?? [],
      );
      const pathALeaks = (allCalls as Array<{ cmd: string; args?: Record<string, unknown> }>).filter(
        (c) =>
          c.args &&
          typeof c.args === "object" &&
          Object.values(c.args).includes(PROJECT_A_PATH),
      );

      expect(pathALeaks).toHaveLength(0);

      await takeStepScreenshot(page, PREFIX, "test6-no-stale-ipc-leaks");
    });
  });

  // =========================================================================
  // Test 7: No console errors during project switch
  // =========================================================================
  test.describe("no console errors during project switch", () => {
    test("zero console errors across open -> interact -> switch -> interact journey", async ({
      page,
    }) => {
      const consoleErrors: string[] = [];
      const pageErrors: string[] = [];

      page.on("console", (msg) => {
        if (msg.type() === "error") {
          const text = msg.text();
          if (!text.includes("[tauri-mock]")) {
            consoleErrors.push(text);
          }
        }
      });

      page.on("pageerror", (err) => {
        pageErrors.push(err.message);
      });

      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1000);

      await takeStepScreenshot(page, PREFIX, "test7-step1-project-a-loaded");

      // Interact: open a chapter tab
      await page.getByTitle("1. The Awakening").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Awakening" })
        .waitFor();

      await takeStepScreenshot(page, PREFIX, "test7-step2-tab-opened");

      // Switch project
      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(1500);

      await takeStepScreenshot(page, PREFIX, "test7-step3-switched-to-b");

      // Interact: open the Project B chapter
      await page.getByTitle("1. The Beginning").click();
      await page
        .getByRole("tablist", { name: "Open documents" })
        .getByRole("tab", { name: "The Beginning" })
        .waitFor();

      await page.waitForTimeout(500);

      await takeStepScreenshot(page, PREFIX, "test7-step4-project-b-tab");

      expect(consoleErrors).toEqual([]);
      expect(pageErrors).toEqual([]);
    });
  });

  // =========================================================================
  // Test 8: Idle IPC audit — no spurious calls 3 seconds after switch settles
  // =========================================================================
  test.describe("no spurious IPC calls during idle period after switch", () => {
    test("zero IPC calls during 3-second idle after project switch settles", async ({
      page,
    }) => {
      await setupDefaultTauriMocks(page, buildDualProjectMocks());
      await page.goto("/");
      await page.getByRole("button", { name: /open project/i }).click();
      await page.getByText("Binder").waitFor({ state: "visible", timeout: 10000 });
      await page.waitForTimeout(1500);

      await switchToProject(page, PROJECT_B_PATH, "Project Beta");
      await page.waitForTimeout(2000); // wait for all loads to finish

      await takeStepScreenshot(page, PREFIX, "test8-before-idle-measurement");

      // Now measure 3 seconds of idle
      await clearIpcCalls(page);
      await page.waitForTimeout(3000);

      await takeStepScreenshot(page, PREFIX, "test8-after-3s-idle");

      const schemasCalls = await getIpcCallsByCommand(page, "list_schemas");
      const entityCalls = await getIpcCallsByCommand(page, "list_entities");
      const configCalls = await getIpcCallsByCommand(page, "get_manuscript_config");
      const chapterCalls = await getIpcCallsByCommand(page, "get_chapter");

      expect(schemasCalls.length).toBe(0);
      expect(entityCalls.length).toBe(0);
      expect(configCalls.length).toBe(0);
      expect(chapterCalls.length).toBe(0);
    });
  });
});
