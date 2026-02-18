import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { entityStore } from "./entities.svelte";

const mockedInvoke = vi.mocked(invoke);

describe("EntityStore", () => {
  beforeEach(() => {
    entityStore.reset();
    mockedInvoke.mockReset();
  });

  describe("schemasLoaded after empty load (Bug 1)", () => {
    it("schemasLoaded is true when loadSchemas returns empty array", async () => {
      mockedInvoke.mockResolvedValueOnce([]);
      await entityStore.loadSchemas("/project");
      expect(entityStore.schemasLoaded).toBe(true);
    });

    it("schemasLoaded is false before any load", () => {
      expect(entityStore.schemasLoaded).toBe(false);
    });

    it("schemasLoadedPath tracks which project was loaded", async () => {
      mockedInvoke.mockResolvedValueOnce([]);
      await entityStore.loadSchemas("/project-a");
      expect(entityStore.schemasLoadedPath).toBe("/project-a");
    });

    it("schemasLoadedPath resets to null on reset()", async () => {
      mockedInvoke.mockResolvedValueOnce([]);
      await entityStore.loadSchemas("/project");
      entityStore.reset();
      expect(entityStore.schemasLoadedPath).toBeNull();
    });
  });

  describe("per-type loading flags (Bug 2/4)", () => {
    it("isLoadingSchemas is independent from isSaving", async () => {
      const schemaPromise = new Promise((resolve) => {
        mockedInvoke.mockImplementation(async (cmd: string) => {
          if (cmd === "list_schemas") {
            // Hold the schema load open
            return new Promise((r) => setTimeout(() => r([]), 50));
          }
          return null;
        });
      });

      // Start schema load (don't await)
      const loadPromise = entityStore.loadSchemas("/project");
      expect(entityStore.isLoadingSchemas).toBe(true);

      // isSaving should be independent
      expect(entityStore.isSaving).toBe(false);
      await loadPromise.catch(() => {});
    });

    it("isLoadingEntities tracks per-type state", async () => {
      mockedInvoke.mockImplementation(async (cmd: string, args?: unknown) => {
        if (cmd === "list_entities") {
          return [];
        }
        return null;
      });

      await entityStore.loadEntities("/project", "character");
      // After load completes, the per-type flag should be false
      expect(entityStore.isLoadingEntities["character"]).toBeFalsy();
    });

    it("loadEntities prevents concurrent loads for the same type", async () => {
      let resolveFirst: (value: unknown) => void;
      let callCount = 0;
      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_entities") {
          callCount++;
          if (callCount === 1) {
            return new Promise((resolve) => {
              resolveFirst = resolve;
            });
          }
          return [];
        }
        return null;
      });

      // Start first load
      const first = entityStore.loadEntities("/project", "character");
      // Try second load for same type while first is in-flight
      const second = entityStore.loadEntities("/project", "character");

      // Resolve the first
      resolveFirst!([]);
      await first;
      await second;

      // Should only have called invoke once for list_entities
      const listCalls = mockedInvoke.mock.calls.filter(
        (c) => c[0] === "list_entities",
      );
      expect(listCalls).toHaveLength(1);
    });

    it("isLoading is a derived that ORs all sub-flags", async () => {
      expect(entityStore.isLoading).toBe(false);

      // When any sub-flag is true, isLoading should be true
      let resolveSchemas: (value: unknown) => void;
      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_schemas") {
          return new Promise((resolve) => {
            resolveSchemas = resolve;
          });
        }
        return null;
      });

      const loadPromise = entityStore.loadSchemas("/project");
      expect(entityStore.isLoading).toBe(true);
      expect(entityStore.isLoadingSchemas).toBe(true);

      resolveSchemas!([]);
      await loadPromise;
      expect(entityStore.isLoading).toBe(false);
    });
  });

  describe("saveEntity in-place update (Bug 3)", () => {
    it("saveEntity does NOT delete entitiesByType[type]", async () => {
      // Pre-populate entities
      mockedInvoke.mockImplementation(async (cmd: string, args?: unknown) => {
        if (cmd === "list_entities") return [
          { title: "Elena", slug: "elena", schemaType: "character", tags: [] },
        ];
        if (cmd === "save_entity") return null;
        return null;
      });

      await entityStore.loadEntities("/project", "character");
      expect(entityStore.entitiesByType["character"]).toHaveLength(1);

      // Save entity
      await entityStore.saveEntity("/project", {
        title: "Elena Updated",
        slug: "elena",
        schemaSlug: "character",
        tags: ["updated"],
        spiderValues: {},
        fields: {},
        body: "",
      });

      // entitiesByType should still have the character entry (not deleted)
      expect(entityStore.entitiesByType["character"]).toBeDefined();
      // And it should be updated in-place
      expect(entityStore.entitiesByType["character"][0].title).toBe(
        "Elena Updated",
      );
    });

    it("saveEntity does not trigger list_entities IPC call", async () => {
      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_entities")
          return [
            {
              title: "Elena",
              slug: "elena",
              schemaType: "character",
              tags: [],
            },
          ];
        if (cmd === "save_entity") return null;
        return null;
      });

      await entityStore.loadEntities("/project", "character");
      mockedInvoke.mockClear();

      await entityStore.saveEntity("/project", {
        title: "Elena Updated",
        slug: "elena",
        schemaSlug: "character",
        tags: [],
        spiderValues: {},
        fields: {},
        body: "",
      });

      // Only save_entity should have been called, NOT list_entities
      const calls = mockedInvoke.mock.calls.map((c) => c[0]);
      expect(calls).toContain("save_entity");
      expect(calls).not.toContain("list_entities");
    });
  });

  describe("createEntity / deleteEntity reload entities (Bug 3 inverse)", () => {
    it("createEntity triggers reloadEntities (list_entities call)", async () => {
      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_entities") return [];
        if (cmd === "create_entity")
          return { title: "New", slug: "new", schemaSlug: "character", tags: [], spiderValues: {}, fields: {}, body: "" };
        return null;
      });

      await entityStore.createEntity("/project", "character", "New");

      const calls = mockedInvoke.mock.calls.map((c) => c[0]);
      expect(calls).toContain("create_entity");
      expect(calls).toContain("list_entities");
    });

    it("deleteEntity triggers reloadEntities (list_entities call)", async () => {
      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_entities")
          return [
            {
              title: "Elena",
              slug: "elena",
              schemaType: "character",
              tags: [],
            },
          ];
        if (cmd === "delete_entity") return null;
        return null;
      });

      await entityStore.loadEntities("/project", "character");
      mockedInvoke.mockClear();

      mockedInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "list_entities") return [];
        if (cmd === "delete_entity") return null;
        return null;
      });

      await entityStore.deleteEntity("/project", "character", "elena");

      const calls = mockedInvoke.mock.calls.map((c) => c[0]);
      expect(calls).toContain("delete_entity");
      expect(calls).toContain("list_entities");
    });
  });

  describe("reset clears all state", () => {
    it("reset clears all loading flags and data", async () => {
      mockedInvoke.mockResolvedValueOnce([{ name: "Character", entityType: "character", fieldCount: 1, axisCount: 0 }]);
      await entityStore.loadSchemas("/project");

      entityStore.reset();

      expect(entityStore.schemaSummaries).toEqual([]);
      expect(entityStore.schemaCache).toEqual({});
      expect(entityStore.entitiesByType).toEqual({});
      expect(entityStore.currentEntity).toBeNull();
      expect(entityStore.isLoading).toBe(false);
      expect(entityStore.isLoadingSchemas).toBe(false);
      expect(entityStore.isSaving).toBe(false);
      expect(entityStore.schemasLoadedPath).toBeNull();
      expect(entityStore.error).toBeNull();
    });
  });
});
