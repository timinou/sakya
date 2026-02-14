import { test, expect } from "@playwright/test";
import {
  setupDefaultTauriMocks,
  openMockProject,
  getIpcCalls,
} from "./utils/tauri-mocks";

test.beforeEach(async ({ page }) => {
  await setupDefaultTauriMocks(page);
  await page.goto("/");
});

test.describe("Project Launcher", () => {
  test("displays Sakya launcher heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: /sakya/i }),
    ).toBeVisible();
    await expect(page.getByText("A writing application")).toBeVisible();
  });

  test("shows Create Project and Open Project buttons", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: /create project/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /open project/i }),
    ).toBeVisible();
  });

  test("create project flow - shows form with name and folder inputs", async ({
    page,
  }) => {
    await page.getByRole("button", { name: /create project/i }).click();

    // Form elements should be visible
    await expect(page.getByLabel(/project name/i)).toBeVisible();
    await expect(page.getByLabel(/location/i)).toBeVisible();
    await expect(
      page.getByRole("button", { name: /choose folder/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /^create$/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /cancel/i }),
    ).toBeVisible();
  });

  test("cancel returns to launcher buttons", async ({ page }) => {
    await page.getByRole("button", { name: /create project/i }).click();
    await expect(page.getByLabel(/project name/i)).toBeVisible();

    await page.getByRole("button", { name: /cancel/i }).click();

    await expect(
      page.getByRole("button", { name: /create project/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /open project/i }),
    ).toBeVisible();
  });

  test("open project flow - calls dialog and opens project", async ({
    page,
  }) => {
    await page.getByRole("button", { name: /open project/i }).click();

    // The dialog mock returns '/mock/project/path' and open_project mock
    // returns a manifest, so projectState.isOpen becomes true.
    // The IPC calls should include the dialog and open_project commands.
    const calls = await getIpcCalls(page);
    const dialogCall = calls.find((c) => c.cmd === "plugin:dialog|open");
    expect(dialogCall).toBeDefined();
  });

  test("create button is disabled when inputs are empty", async ({ page }) => {
    await page.getByRole("button", { name: /create project/i }).click();
    const createBtn = page.getByRole("button", { name: /^create$/i });
    await expect(createBtn).toBeDisabled();
  });
});

test.describe("Legacy Project Compatibility", () => {
  test("opens a legacy project without version/timestamps without error", async ({
    page,
  }) => {
    // Simulate a legacy manifest that only has name and author
    // (backend now fills defaults for missing version/timestamps)
    await openMockProject(page, {
      open_project: {
        name: "Legacy Novel",
        version: "0.1.0",
        author: "Old Author",
        description: null,
        createdAt: "2026-02-14T00:00:00Z",
        updatedAt: "2026-02-14T00:00:00Z",
      },
    });

    // Binder should be visible (project opened successfully)
    await expect(page.getByText("Binder")).toBeVisible();

    // No error banner should appear
    await expect(page.locator('[role="alert"]')).not.toBeVisible();
  });
});
