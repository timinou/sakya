import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
  MOCK_SCHEMA_SUMMARIES,
  MOCK_ENTITIES_BY_TYPE,
  MOCK_ENTITY_INSTANCES,
} from "./utils/tauri-mocks";
import { takeStepScreenshot } from "./utils/screenshots";

// Screenshot prefix for all tests in this file
const PREFIX = "bug005";

test.describe("Infinite Loop Guards (BUG-005)", () => {
  // =========================================================================
  // Bug 1: Empty schema project should not cause infinite list_schemas calls
  // =========================================================================
  test.describe("empty schema project", () => {
    test("list_schemas called exactly once when it returns empty array + no infinite loop", async ({
      page,
    }) => {
      // Override list_schemas to return empty
      await openMockProject(page, {
        list_schemas: [],
        list_entities: [],
      });

      // Wait for the app to fully settle after initial load
      await page.waitForTimeout(1500);

      // Screenshot: App opened with empty schemas — binder visible, no entity sections
      await takeStepScreenshot(page, PREFIX, "bug1-empty-schema-settled");

      const initialSchemaCalls = await getIpcCallsByCommand(
        page,
        "list_schemas",
      );
      // Should be called exactly once (on initial load), not looping
      expect(initialSchemaCalls.length).toBe(1);

      // Wait another 3 seconds to confirm no loop triggers
      await clearIpcCalls(page);
      await page.waitForTimeout(3000);

      // Screenshot: App after 3s idle — should look identical to settled state
      await takeStepScreenshot(page, PREFIX, "bug1-empty-schema-after-3s-idle");

      const loopCheckCalls = await getIpcCallsByCommand(page, "list_schemas");
      expect(loopCheckCalls.length).toBe(0);
    });

    test("no entity sections shown and no loading spinner stuck for empty schema project", async ({
      page,
    }) => {
      await openMockProject(page, {
        list_schemas: [],
        list_entities: [],
      });

      // Wait for app to settle
      await page.waitForTimeout(1500);

      // Screenshot: Binder with empty schemas
      await takeStepScreenshot(
        page,
        PREFIX,
        "bug1-binder-no-entity-sections",
      );

      // Manuscript and Notes sections should still be present
      await expect(page.getByText("Manuscript")).toBeVisible();
      await expect(page.getByText("Notes")).toBeVisible();

      // Should NOT show any entity sections (Characters, Places are absent)
      await expect(page.getByText("Characters")).not.toBeVisible();
      await expect(page.getByText("Places")).not.toBeVisible();

      // No loading spinner should be stuck visible
      await expect(page.locator(".loading-indicator")).not.toBeVisible();

      // The "New Entity Type" button SHOULD appear since schemasLoaded is true
      const newEntityTypeBtn = page.getByText("New Entity Type");
      await expect(newEntityTypeBtn).toBeVisible();

      // Screenshot: Confirming "New Entity Type" button is visible
      await takeStepScreenshot(
        page,
        PREFIX,
        "bug1-new-entity-type-btn-visible",
      );
    });
  });

  // =========================================================================
  // Bug 2: Multi-type load — each list_entities called once per type
  // =========================================================================
  test.describe("multi-type entity loading", () => {
    test("list_entities called exactly once per schema type with 3 types (one empty)", async ({
      page,
    }) => {
      // Use a 3-schema setup: character (3 entities), place (2 entities), item (0 entities)
      const threeSchemas = [
        ...MOCK_SCHEMA_SUMMARIES,
        { name: "Item", entityType: "item", fieldCount: 1, axisCount: 0 },
      ];

      const entitiesData = {
        ...MOCK_ENTITIES_BY_TYPE,
        item: [],
      };

      await openMockProject(page, {
        list_schemas: threeSchemas,
        list_entities: new Function(
          "args",
          `var entities = ${JSON.stringify(entitiesData)};
           var schemaType = args && args.schemaType;
           return entities[schemaType] || [];`,
        ),
      });

      // Wait for all entity loading to complete
      await page.waitForTimeout(2000);

      // Screenshot: All 3 entity sections visible in binder
      await takeStepScreenshot(
        page,
        PREFIX,
        "bug2-three-entity-sections-loaded",
      );

      // Verify all 3 entity sections appear in binder
      await expect(page.getByText("Characters")).toBeVisible();
      await expect(page.getByText("Places")).toBeVisible();
      await expect(page.getByText("Items")).toBeVisible();

      // Verify counts are correct
      const charSection = page
        .locator(".section")
        .filter({ hasText: "Characters" });
      await expect(charSection.locator(".section-count")).toHaveText("3");

      const placeSection = page
        .locator(".section")
        .filter({ hasText: "Places" });
      await expect(placeSection.locator(".section-count")).toHaveText("2");

      const itemSection = page
        .locator(".section")
        .filter({ hasText: "Items" });
      await expect(itemSection.locator(".section-count")).toHaveText("0");

      // Screenshot: Empty "Items" section with placeholder CTA
      await takeStepScreenshot(
        page,
        PREFIX,
        "bug2-items-section-empty-placeholder",
      );

      // Verify the empty Items section shows the "Add first item" placeholder CTA
      await expect(
        page
          .locator(".placeholder-cta")
          .filter({ hasText: /Add first item/i }),
      ).toBeVisible();

      // Verify specific entity names appear in binder
      for (const entity of MOCK_ENTITIES_BY_TYPE.character as Array<{
        title: string;
      }>) {
        await expect(page.getByTitle(entity.title)).toBeVisible();
      }
      for (const entity of MOCK_ENTITIES_BY_TYPE.place as Array<{
        title: string;
      }>) {
        await expect(page.getByTitle(entity.title)).toBeVisible();
      }

      // Now verify IPC call counts — each type called exactly once
      const entityCalls = await getIpcCallsByCommand(page, "list_entities");
      const callsByType: Record<string, number> = {};
      for (const call of entityCalls) {
        const args = call.args as { schemaType?: string } | undefined;
        const type = args?.schemaType ?? "unknown";
        callsByType[type] = (callsByType[type] ?? 0) + 1;
      }

      expect(callsByType["character"]).toBe(1);
      expect(callsByType["place"]).toBe(1);
      expect(callsByType["item"]).toBe(1);

      // Verify all 3 calls happened BEFORE the 2s wait (no delayed re-fires).
      // Clear and wait 2 more seconds — no new list_entities calls should appear.
      await clearIpcCalls(page);
      await page.waitForTimeout(2000);

      const delayedCalls = await getIpcCallsByCommand(page, "list_entities");
      expect(delayedCalls.length).toBe(0);
    });
  });

  // =========================================================================
  // Bug 3: Entity save should NOT trigger list_entities
  // =========================================================================
  test.describe("entity save does not reload list", () => {
    test("saving entity via UI auto-save does not trigger list_entities IPC call", async ({
      page,
    }) => {
      await openMockProject(page);

      // Wait for initial load to settle
      await page.waitForTimeout(1500);

      // Click on an entity to open it
      await page.getByTitle("Elena Blackwood").click();

      // Wait for entity form to load
      const titleInput = page.getByLabel("Entity title");
      await expect(titleInput).toBeVisible({ timeout: 5000 });
      await expect(titleInput).toHaveValue("Elena Blackwood");

      // Screenshot: Entity form before editing
      await takeStepScreenshot(page, PREFIX, "bug3-entity-form-before-edit");

      // Clear IPC calls to establish baseline before the edit
      await clearIpcCalls(page);

      // Edit the entity title via the UI to trigger real auto-save debounce
      await titleInput.fill("Elena Blackwood-Modified");

      // Screenshot: Entity form after editing (title changed, not yet saved)
      await takeStepScreenshot(page, PREFIX, "bug3-entity-form-after-edit");

      // Wait for debounced auto-save (1.5s debounce + 1s buffer = 2.5s)
      await page.waitForTimeout(2500);

      // Assert save_entity WAS called
      const saveCalls = await getIpcCallsByCommand(page, "save_entity");
      expect(saveCalls.length).toBeGreaterThanOrEqual(1);

      // Assert list_entities was NOT called (no full reload after save)
      const listCalls = await getIpcCallsByCommand(page, "list_entities");
      expect(listCalls.length).toBe(0);

      // Assert list_schemas was NOT called either
      const schemaCalls = await getIpcCallsByCommand(page, "list_schemas");
      expect(schemaCalls.length).toBe(0);

      // Verify the store's entitiesByType was updated in-place (not deleted)
      const storeState = await page.evaluate(async () => {
        const stores = await import("/src/lib/stores/index.ts");
        const list = stores.entityStore.entitiesByType["character"];
        return {
          listExists: !!list,
          listLength: list?.length ?? 0,
          firstTitle: list?.[0]?.title ?? null,
        };
      });

      // The entity list should still exist (not deleted by invalidateType)
      expect(storeState.listExists).toBe(true);
      expect(storeState.listLength).toBe(3);
      // The in-place update should have changed the title in the store
      expect(storeState.firstTitle).toBe("Elena Blackwood-Modified");

      // Screenshot: Binder state after save (store updated in-place)
      await takeStepScreenshot(
        page,
        PREFIX,
        "bug3-binder-after-save-store-verified",
      );
    });
  });

  // =========================================================================
  // Idle IPC audit: no spurious calls after settling
  // =========================================================================
  test.describe("idle IPC audit", () => {
    test("no IPC calls during 3-second idle period after app settles", async ({
      page,
    }) => {
      await openMockProject(page);

      // Wait for all initial loading to settle
      await page.waitForTimeout(2000);

      // Screenshot: App in fully settled state before idle measurement
      await takeStepScreenshot(page, PREFIX, "idle-app-settled");

      // Clear all IPC calls
      await clearIpcCalls(page);

      // Wait 3 seconds of idle time
      await page.waitForTimeout(3000);

      // Screenshot: App after 3s idle — should look identical to settled state
      await takeStepScreenshot(page, PREFIX, "idle-app-after-3s");

      // Check that no IPC calls were made during idle
      const schemaCalls = await getIpcCallsByCommand(page, "list_schemas");
      const entityCalls = await getIpcCallsByCommand(page, "list_entities");
      const getEntityCalls = await getIpcCallsByCommand(page, "get_entity");
      const getSchemaCalls = await getIpcCallsByCommand(page, "get_schema");
      const getChapterCalls = await getIpcCallsByCommand(page, "get_chapter");
      const saveEntityCalls = await getIpcCallsByCommand(page, "save_entity");

      expect(schemaCalls.length).toBe(0);
      expect(entityCalls.length).toBe(0);
      expect(getEntityCalls.length).toBe(0);
      expect(getSchemaCalls.length).toBe(0);
      expect(getChapterCalls.length).toBe(0);
      expect(saveEntityCalls.length).toBe(0);
    });
  });

  // =========================================================================
  // Console error monitoring
  // =========================================================================
  test.describe("no console errors during normal operation", () => {
    test("no console errors or unhandled rejections during multi-step user journey", async ({
      page,
    }) => {
      const consoleErrors: string[] = [];
      const pageErrors: string[] = [];

      page.on("console", (msg) => {
        if (msg.type() === "error") {
          const text = msg.text();
          // Ignore known non-error console messages from the mock layer
          if (!text.includes("[tauri-mock]")) {
            consoleErrors.push(text);
          }
        }
      });

      // Capture unhandled promise rejections
      page.on("pageerror", (err) => {
        pageErrors.push(err.message);
      });

      await openMockProject(page);

      // Wait for initial load
      await page.waitForTimeout(2000);

      // Screenshot: App loaded without errors
      await takeStepScreenshot(
        page,
        PREFIX,
        "console-step1-initial-load",
      );

      // Step 2: Open an entity
      await page.getByTitle("Elena Blackwood").click();
      await expect(page.getByLabel("Entity title")).toBeVisible({
        timeout: 5000,
      });
      await takeStepScreenshot(page, PREFIX, "console-step2-entity-open");

      // Step 3: Switch to a different entity
      await page.getByTitle("Marcus Thorne").click();
      await expect(page.getByLabel("Entity title")).toBeVisible({
        timeout: 5000,
      });
      await expect(page.getByLabel("Entity title")).toHaveValue(
        "Marcus Thorne",
      );
      await takeStepScreenshot(
        page,
        PREFIX,
        "console-step3-entity-switched",
      );

      // Step 4: Collapse the Characters section
      const characterHeader = page
        .locator(".section-header")
        .filter({ hasText: "Characters" });
      await characterHeader.click();
      await expect(
        page.getByTitle("Elena Blackwood"),
      ).not.toBeVisible();
      await takeStepScreenshot(
        page,
        PREFIX,
        "console-step4-section-collapsed",
      );

      // Step 5: Expand the Characters section again
      await characterHeader.click();
      await expect(
        page.getByTitle("Elena Blackwood"),
      ).toBeVisible();
      await takeStepScreenshot(
        page,
        PREFIX,
        "console-step5-section-expanded",
      );

      // No console errors should have been logged throughout the journey
      expect(consoleErrors).toEqual([]);
      // No unhandled promise rejections
      expect(pageErrors).toEqual([]);
    });
  });

  // =========================================================================
  // NEW: Entity Create + Delete triggers reload (Bug 3 inverse)
  // =========================================================================
  test.describe("entity create and delete trigger list reload", () => {
    test("creating entity calls create_entity and list_entities; deleting calls delete_entity and list_entities", async ({
      page,
    }) => {
      // Use a mock that returns an updated list after creation
      const entitiesAfterCreate = [
        ...MOCK_ENTITIES_BY_TYPE.character,
        {
          title: "New Hero",
          slug: "new-hero",
          schemaType: "character",
          tags: [],
        },
      ];

      await openMockProject(page, {
        // After entity creation, return the new list
        list_entities: new Function(
          "args",
          `var schemaType = args && args.schemaType;
           if (schemaType === 'character') {
             return ${JSON.stringify(entitiesAfterCreate)};
           }
           var places = ${JSON.stringify(MOCK_ENTITIES_BY_TYPE.place)};
           return places;`,
        ),
      });

      // Wait for initial load to settle
      await page.waitForTimeout(1500);

      // Screenshot: Initial binder state
      await takeStepScreenshot(
        page,
        PREFIX,
        "create-delete-step1-initial-state",
      );

      // --- Create entity via binder UI ---
      const characterHeader = page
        .locator(".section-header")
        .filter({ hasText: "Characters" });
      await characterHeader.hover();

      const addButton = page.getByLabel("Add Characters");
      await expect(addButton).toBeVisible();

      // Clear IPC calls before the create action
      await clearIpcCalls(page);

      // Click the add button to start inline creation
      await addButton.click();

      // Type the new entity name in the inline input
      const inlineInput = page.locator(".inline-input").last();
      await expect(inlineInput).toBeVisible({ timeout: 3000 });
      await inlineInput.fill("New Hero");

      // Screenshot: Inline creation input visible
      await takeStepScreenshot(
        page,
        PREFIX,
        "create-delete-step2-inline-input",
      );

      // Press Enter to confirm creation
      await inlineInput.press("Enter");

      // Wait for the create + reload to complete
      await page.waitForTimeout(1000);

      // Verify create_entity was called
      const createCalls = await getIpcCallsByCommand(page, "create_entity");
      expect(createCalls.length).toBe(1);

      // Verify list_entities was called (reload expected after create)
      const listAfterCreate = await getIpcCallsByCommand(
        page,
        "list_entities",
      );
      expect(listAfterCreate.length).toBeGreaterThanOrEqual(1);

      // Screenshot: Binder showing new entity
      await takeStepScreenshot(
        page,
        PREFIX,
        "create-delete-step3-entity-created",
      );

      // The new entity should now appear in the binder
      await expect(page.getByTitle("New Hero")).toBeVisible();

      // --- Delete entity via context menu ---
      // Clear IPC calls before the delete action
      await clearIpcCalls(page);

      // Right-click the new entity for context menu
      const newEntityItem = page.getByTitle("New Hero");
      await newEntityItem.click({ button: "right" });

      const deleteMenuItem = page.getByText("Delete", { exact: true });
      await expect(deleteMenuItem).toBeVisible({ timeout: 3000 });
      await deleteMenuItem.click();

      // Confirm deletion in dialog
      const confirmBtn = page
        .getByRole("button", { name: /delete/i })
        .last();
      await confirmBtn.click();

      // Wait for delete + reload to complete
      await page.waitForTimeout(1000);

      // Verify delete_entity was called
      const deleteCalls = await getIpcCallsByCommand(page, "delete_entity");
      expect(deleteCalls.length).toBe(1);

      // Verify list_entities was called (reload expected after delete)
      const listAfterDelete = await getIpcCallsByCommand(
        page,
        "list_entities",
      );
      expect(listAfterDelete.length).toBeGreaterThanOrEqual(1);

      // Screenshot: Binder after entity deleted
      await takeStepScreenshot(
        page,
        PREFIX,
        "create-delete-step4-entity-deleted",
      );
    });
  });

  // =========================================================================
  // NEW: schemasLoadedPath correctly tracks project (path-based guard)
  // =========================================================================
  test.describe("schemasLoadedPath tracks project", () => {
    test("schemasLoadedPath matches project path after initial load; schemasLoaded is true", async ({
      page,
    }) => {
      await openMockProject(page);

      // Wait for initial load to settle
      await page.waitForTimeout(1500);

      // Screenshot: Project loaded state
      await takeStepScreenshot(
        page,
        PREFIX,
        "path-guard-step1-project-loaded",
      );

      // Verify entity sections are visible with correct data
      await expect(page.getByText("Characters")).toBeVisible();
      await expect(page.getByText("Places")).toBeVisible();
      await expect(page.getByTitle("Elena Blackwood")).toBeVisible();

      // Verify the store has correct path-based tracking
      const storeState = await page.evaluate(async () => {
        const stores = await import("/src/lib/stores/index.ts");
        return {
          schemasLoadedPath: stores.entityStore.schemasLoadedPath,
          schemasLoaded: stores.entityStore.schemasLoaded,
          isLoadingSchemas: stores.entityStore.isLoadingSchemas,
          isSaving: stores.entityStore.isSaving,
          schemaCount: stores.entityStore.schemaSummaries.length,
        };
      });

      // schemasLoadedPath should match the mock project path
      expect(storeState.schemasLoadedPath).toBe("/mock/project/path");
      // schemasLoaded is derived as (schemasLoadedPath !== null)
      expect(storeState.schemasLoaded).toBe(true);
      // No loading should be in progress
      expect(storeState.isLoadingSchemas).toBe(false);
      expect(storeState.isSaving).toBe(false);
      // Should have loaded the 2 mock schemas
      expect(storeState.schemaCount).toBe(2);

      // Screenshot: Confirmed store state matches expectations
      await takeStepScreenshot(
        page,
        PREFIX,
        "path-guard-step2-store-verified",
      );
    });

    test("reset() clears schemasLoadedPath so next load triggers fresh fetch", async ({
      page,
    }) => {
      await openMockProject(page);
      await page.waitForTimeout(1500);

      // Reset the store and verify path is cleared
      const storeAfterReset = await page.evaluate(async () => {
        const stores = await import("/src/lib/stores/index.ts");
        stores.entityStore.reset();
        return {
          schemasLoadedPath: stores.entityStore.schemasLoadedPath,
          schemasLoaded: stores.entityStore.schemasLoaded,
          schemaCount: stores.entityStore.schemaSummaries.length,
        };
      });

      expect(storeAfterReset.schemasLoadedPath).toBeNull();
      expect(storeAfterReset.schemasLoaded).toBe(false);
      expect(storeAfterReset.schemaCount).toBe(0);

      // Screenshot: Store after reset
      await takeStepScreenshot(
        page,
        PREFIX,
        "path-guard-step3-after-reset",
      );
    });
  });
});
