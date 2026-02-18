import { test, expect } from "@playwright/test";
import {
  openMockProject,
  setupDefaultTauriMocks,
  getIpcCallsByCommand,
  clearIpcCalls,
  MOCK_SCHEMA_SUMMARIES,
  MOCK_ENTITIES_BY_TYPE,
  MOCK_ENTITY_INSTANCES,
} from "./utils/tauri-mocks";

test.describe("Infinite Loop Guards (BUG-005)", () => {
  // -----------------------------------------------------------------------
  // Bug 1: Empty schema project should not cause infinite list_schemas calls
  // -----------------------------------------------------------------------
  test.describe("empty schema project", () => {
    test("list_schemas called exactly once when it returns empty array", async ({
      page,
    }) => {
      // Override list_schemas to return empty
      await openMockProject(page, {
        list_schemas: [],
        list_entities: [],
      });

      // Wait for the app to settle
      await page.waitForTimeout(2000);

      const schemaCalls = await getIpcCallsByCommand(page, "list_schemas");
      // Should be called exactly once (on initial load), not looping
      expect(schemaCalls.length).toBe(1);
    });

    test("no entity sections shown for empty schema project", async ({
      page,
    }) => {
      await openMockProject(page, {
        list_schemas: [],
        list_entities: [],
      });

      await page.waitForTimeout(1000);

      // Should NOT show any entity sections
      await expect(page.getByText("Characters")).not.toBeVisible();
      await expect(page.getByText("Places")).not.toBeVisible();
    });
  });

  // -----------------------------------------------------------------------
  // Bug 2: Multi-type load — each list_entities called once per type
  // -----------------------------------------------------------------------
  test.describe("multi-type entity loading", () => {
    test("list_entities called exactly once per schema type", async ({
      page,
    }) => {
      // Use a 3-schema setup to test multi-type loading
      const threeSchemas = [
        ...MOCK_SCHEMA_SUMMARIES,
        { name: "Item", entityType: "item", fieldCount: 1, axisCount: 0 },
      ];

      await openMockProject(page, {
        list_schemas: threeSchemas,
        list_entities: new Function(
          "args",
          `var entities = ${JSON.stringify({ ...MOCK_ENTITIES_BY_TYPE, item: [] })};
           var schemaType = args && args.schemaType;
           return entities[schemaType] || [];`,
        ),
      });

      // Wait for all entity loading to complete
      await page.waitForTimeout(2000);

      const entityCalls = await getIpcCallsByCommand(page, "list_entities");

      // Group calls by schemaType
      const callsByType: Record<string, number> = {};
      for (const call of entityCalls) {
        const args = call.args as { schemaType?: string } | undefined;
        const type = args?.schemaType ?? "unknown";
        callsByType[type] = (callsByType[type] ?? 0) + 1;
      }

      // Each type should be called exactly once
      expect(callsByType["character"]).toBe(1);
      expect(callsByType["place"]).toBe(1);
      expect(callsByType["item"]).toBe(1);
    });
  });

  // -----------------------------------------------------------------------
  // Bug 3: Entity save should NOT trigger list_entities
  // -----------------------------------------------------------------------
  test.describe("entity save does not reload list", () => {
    test("saving entity does not trigger list_entities IPC call", async ({
      page,
    }) => {
      await openMockProject(page);

      // Wait for initial load to settle
      await page.waitForTimeout(1500);

      // Click on an entity to open it
      await page.getByText("Elena Blackwood").click();

      // Wait for entity to load
      await page.waitForTimeout(1000);

      // Clear IPC calls to establish baseline
      await clearIpcCalls(page);

      // Trigger a save by modifying the entity via store
      await page.evaluate(async () => {
        const stores = await import("/src/lib/stores/index.ts");
        const entity = stores.entityStore.currentEntity;
        if (entity) {
          await stores.entityStore.saveEntity("/mock/project/path", {
            ...entity,
            body: entity.body + " updated",
          });
        }
      });

      await page.waitForTimeout(500);

      // Check that list_entities was NOT called
      const listCalls = await getIpcCallsByCommand(page, "list_entities");
      expect(listCalls.length).toBe(0);

      // But save_entity SHOULD have been called
      const saveCalls = await getIpcCallsByCommand(page, "save_entity");
      expect(saveCalls.length).toBe(1);
    });
  });

  // -----------------------------------------------------------------------
  // Idle IPC audit: no spurious calls after settling
  // -----------------------------------------------------------------------
  test.describe("idle IPC audit", () => {
    test("no IPC calls during 3-second idle period after app settles", async ({
      page,
    }) => {
      await openMockProject(page);

      // Wait for all initial loading to settle
      await page.waitForTimeout(2000);

      // Clear all IPC calls
      await clearIpcCalls(page);

      // Wait 3 seconds of idle time
      await page.waitForTimeout(3000);

      // Check that no IPC calls were made during idle
      const allCalls = await getIpcCallsByCommand(page, "list_schemas");
      const entityCalls = await getIpcCallsByCommand(page, "list_entities");

      expect(allCalls.length).toBe(0);
      expect(entityCalls.length).toBe(0);
    });
  });

  // -----------------------------------------------------------------------
  // Console error monitoring
  // -----------------------------------------------------------------------
  test.describe("no console errors during normal operation", () => {
    test("no console errors during initial load and entity interaction", async ({
      page,
    }) => {
      const consoleErrors: string[] = [];
      page.on("console", (msg) => {
        if (msg.type() === "error") {
          // Ignore known non-error console messages
          const text = msg.text();
          if (!text.includes("[tauri-mock]")) {
            consoleErrors.push(text);
          }
        }
      });

      await openMockProject(page);

      // Wait for initial load
      await page.waitForTimeout(2000);

      // Click on an entity
      await page.getByText("Elena Blackwood").click();
      await page.waitForTimeout(1000);

      // No errors should have been logged
      expect(consoleErrors).toEqual([]);
    });
  });
});
