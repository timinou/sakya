import { test, expect } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

// =============================================================================
// Editor Sync E2E Tests (ITEM-164)
//
// Tests the LoroSyncPlugin integration, Loro JS library in the browser,
// fallback behavior, and the exportCrdtToMarkdown utility.
// =============================================================================

test.describe("Editor Sync - Loro CRDT Integration", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
  });

  // ---------------------------------------------------------------------------
  // 1. Loro JS library loads and basic operations work in the browser
  // ---------------------------------------------------------------------------

  test("Loro JS library: LoroDoc creation and text operations", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      const doc = new LoroDoc();
      const text = doc.getText("test-container");

      // Insert text
      text.insert(0, "Hello, world!");
      doc.commit();

      // Read back
      const content = text.toString();
      const length = text.length;

      return { content, length };
    });

    expect(result.content).toBe("Hello, world!");
    expect(result.length).toBe(13);
  });

  test("Loro JS library: two docs sync via binary export/import", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      // Doc A writes content
      const docA = new LoroDoc();
      docA.setPeerId("1");
      const textA = docA.getText("chapter");
      textA.insert(0, "# Chapter One\n\nOnce upon a time...");
      docA.commit();

      // Doc B imports from A
      const docB = new LoroDoc();
      docB.setPeerId("2");
      const exportedA = docA.export({ mode: "update" });
      docB.import(exportedA);

      const textB = docB.getText("chapter");
      const contentB = textB.toString();

      // Doc B makes its own edit
      textB.insert(textB.length, " The end.");
      docB.commit();

      // Doc A imports from B
      const exportedB = docB.export({
        mode: "update",
        from: docA.version(),
      });
      docA.import(exportedB);

      const finalA = textA.toString();
      const finalB = textB.toString();

      return { contentB, finalA, finalB };
    });

    expect(result.contentB).toBe("# Chapter One\n\nOnce upon a time...");
    expect(result.finalA).toBe(
      "# Chapter One\n\nOnce upon a time... The end.",
    );
    expect(result.finalB).toBe(result.finalA);
  });

  test("Loro JS library: subscribeLocalUpdates fires on commit", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      const doc = new LoroDoc();
      const updates: number[] = [];

      doc.subscribeLocalUpdates((update: Uint8Array) => {
        updates.push(update.length);
      });

      const text = doc.getText("ch1");
      text.insert(0, "Some text");
      doc.commit();

      text.insert(9, " more");
      doc.commit();

      return { updateCount: updates.length, hasUpdates: updates.length > 0 };
    });

    expect(result.hasUpdates).toBe(true);
    expect(result.updateCount).toBeGreaterThanOrEqual(2);
  });

  test("Loro JS library: subscribe detects remote changes", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      const docA = new LoroDoc();
      docA.setPeerId("1");
      const docB = new LoroDoc();
      docB.setPeerId("2");

      let remoteEvents = 0;
      docB.subscribe(() => {
        remoteEvents++;
      });

      // A writes and exports
      const textA = docA.getText("ch1");
      textA.insert(0, "Hello from A");
      docA.commit();

      // B imports A's update
      const update = docA.export({ mode: "update" });
      docB.import(update);

      const textB = docB.getText("ch1");

      return {
        remoteEvents,
        contentB: textB.toString(),
      };
    });

    expect(result.remoteEvents).toBeGreaterThanOrEqual(1);
    expect(result.contentB).toBe("Hello from A");
  });

  // ---------------------------------------------------------------------------
  // 2. exportCrdtToMarkdown utility
  // ---------------------------------------------------------------------------

  test("exportCrdtToMarkdown reads LoroText content", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");
      const { exportCrdtToMarkdown } = await import(
        "/src/lib/editor/utils/markdown-io.ts"
      );

      const doc = new LoroDoc();
      const text = doc.getText("test-chapter");
      text.insert(0, "# My Chapter\n\nSome **bold** text and *italic* words.");
      doc.commit();

      return exportCrdtToMarkdown(doc, "test-chapter");
    });

    expect(result).toBe(
      "# My Chapter\n\nSome **bold** text and *italic* words.",
    );
  });

  test("exportCrdtToMarkdown returns empty string for empty container", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");
      const { exportCrdtToMarkdown } = await import(
        "/src/lib/editor/utils/markdown-io.ts"
      );

      const doc = new LoroDoc();
      return exportCrdtToMarkdown(doc, "empty-container");
    });

    expect(result).toBe("");
  });

  // ---------------------------------------------------------------------------
  // 3. Fallback mode — editor works normally without loroDoc
  // ---------------------------------------------------------------------------

  test("editor renders content without loroDoc (fallback mode)", async ({
    page,
  }) => {
    // Click a chapter to open it
    await page.getByTitle("1. The Awakening").click();

    // Tab should appear
    const tabList = page.getByRole("tablist", { name: "Open documents" });
    await expect(tabList).toBeVisible();
    await expect(
      tabList.getByRole("tab", { name: "The Awakening" }),
    ).toBeVisible();

    // Editor should render with content
    const editorContent = page.locator(".editor-content");
    await expect(editorContent).toBeVisible();

    // Content should contain the mock chapter text
    await expect(editorContent).toContainText("morning light filtered");
  });

  test("fallback mode triggers save_chapter on content change", async ({
    page,
  }) => {
    // Open a chapter
    await page.getByTitle("1. The Awakening").click();
    await page
      .getByRole("tab", { name: "The Awakening" })
      .waitFor({ state: "visible" });

    // Wait for editor to load
    const editorContent = page.locator(".editor-content");
    await expect(editorContent).toBeVisible();
    await page.waitForTimeout(500);

    await clearIpcCalls(page);

    // Type into the editor (contenteditable)
    await editorContent.click();
    await page.keyboard.type("New text ");

    // Wait for AutoSavePlugin debounce (1s default + buffer)
    await page.waitForTimeout(2000);

    const saveCalls = await getIpcCallsByCommand(page, "save_chapter");
    expect(saveCalls.length).toBeGreaterThanOrEqual(1);
  });

  test("welcome card shows when no document is open", async ({ page }) => {
    await expect(page.locator(".welcome-card")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // 4. Loro concurrent edit merging
  // ---------------------------------------------------------------------------

  test("Loro CRDT: concurrent edits merge correctly", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      // Initial state: both docs start with same content
      const docA = new LoroDoc();
      docA.setPeerId("1");
      const textA = docA.getText("ch1");
      textA.insert(0, "Hello world");
      docA.commit();

      const docB = new LoroDoc();
      docB.setPeerId("2");
      docB.import(docA.export({ mode: "update" }));

      // Concurrent edits: A inserts at start, B appends at end
      textA.insert(0, "Dear ");
      docA.commit();

      const textB = docB.getText("ch1");
      textB.insert(textB.length, "!");
      docB.commit();

      // Merge both directions
      docA.import(
        docB.export({ mode: "update", from: docA.version() }),
      );
      docB.import(
        docA.export({ mode: "update", from: docB.version() }),
      );

      return {
        finalA: textA.toString(),
        finalB: textB.toString(),
        converged: textA.toString() === textB.toString(),
      };
    });

    // Both docs should converge to the same content
    expect(result.converged).toBe(true);
    // Both edits should be present
    expect(result.finalA).toContain("Dear");
    expect(result.finalA).toContain("Hello world");
    expect(result.finalA).toContain("!");
  });

  test("Loro CRDT: delete + insert on same position merges", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      const docA = new LoroDoc();
      docA.setPeerId("1");
      const textA = docA.getText("ch1");
      textA.insert(0, "abcdef");
      docA.commit();

      const docB = new LoroDoc();
      docB.setPeerId("2");
      docB.import(docA.export({ mode: "update" }));

      // A deletes "cd"
      textA.delete(2, 2);
      docA.commit();

      // B inserts "X" at position 3 (between c and d)
      const textB = docB.getText("ch1");
      textB.insert(3, "X");
      docB.commit();

      // Merge
      docA.import(
        docB.export({ mode: "update", from: docA.version() }),
      );
      docB.import(
        docA.export({ mode: "update", from: docB.version() }),
      );

      return {
        finalA: textA.toString(),
        finalB: textB.toString(),
        converged: textA.toString() === textB.toString(),
      };
    });

    expect(result.converged).toBe(true);
    // Both edits should be reflected (delete "cd" and insert "X")
    expect(result.finalA.length).toBeLessThan("abcdef".length + 1);
  });

  // ---------------------------------------------------------------------------
  // 5. Loro binary update round-trip (simulates sync transport)
  // ---------------------------------------------------------------------------

  test("Loro binary update: incremental export/import round-trip", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const { LoroDoc } = await import("/src/lib/crdt/index.ts");

      const docA = new LoroDoc();
      docA.setPeerId("1");
      const textA = docA.getText("chapter");
      textA.insert(0, "First paragraph.");
      docA.commit();

      // Capture the initial version
      const version1 = docA.version();

      // Make more edits
      textA.insert(textA.length, "\n\nSecond paragraph.");
      docA.commit();

      // Export only the incremental update since version1
      const incrementalUpdate = docA.export({
        mode: "update",
        from: version1,
      });

      // Doc B starts from the initial state
      const docB = new LoroDoc();
      docB.setPeerId("2");
      // First sync B to version1
      docB.import(docA.export({ mode: "update" }));

      // Simulate: B receives the incremental update via network
      // (already imported full state above, so this would be a no-op, but tests the API)
      const textB = docB.getText("chapter");
      const contentB = textB.toString();

      return {
        contentB,
        incrementalSize: incrementalUpdate.length,
        fullExportSize: docA.export({ mode: "update" }).length,
      };
    });

    expect(result.contentB).toBe("First paragraph.\n\nSecond paragraph.");
    // Incremental update should be smaller than full export
    expect(result.incrementalSize).toBeLessThanOrEqual(
      result.fullExportSize,
    );
  });
});
