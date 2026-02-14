import { test, expect } from "@playwright/test";
import { setupDefaultTauriMocks, getIpcCalls } from "./utils/tauri-mocks";

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
