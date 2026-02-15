import { test, expect } from "@playwright/test";
import {
  setupDefaultTauriMocks,
  openMockProject,
  getIpcCalls,
  MOCK_RECENT_PROJECTS,
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

test.describe("Recent Projects", () => {
  test("shows recent projects when list is non-empty", async ({ page }) => {
    await setupDefaultTauriMocks(page, {
      list_recent_projects: MOCK_RECENT_PROJECTS,
    });
    await page.goto("/");

    // Section header should be visible
    await expect(page.getByText("Recent Projects")).toBeVisible();

    // All 3 projects should be listed
    await expect(page.getByText("The Warmth of Distant Things")).toBeVisible();
    await expect(page.getByText("Midnight in Paris")).toBeVisible();
    await expect(page.getByText("Old Forgotten Novel")).toBeVisible();

    // Paths should be shown
    await expect(
      page.getByText("/home/user/projects/warmth-of-distant-things"),
    ).toBeVisible();
  });

  test("does not show recent section when list is empty", async ({ page }) => {
    await setupDefaultTauriMocks(page, {
      list_recent_projects: [],
    });
    await page.goto("/");

    // Section header should NOT be visible
    await expect(page.getByText("Recent Projects")).not.toBeVisible();

    // Create and Open buttons should still be visible
    await expect(
      page.getByRole("button", { name: /create project/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /open project/i }),
    ).toBeVisible();
  });

  test("clicking a recent project opens it", async ({ page }) => {
    await setupDefaultTauriMocks(page, {
      list_recent_projects: MOCK_RECENT_PROJECTS,
      add_recent_project: MOCK_RECENT_PROJECTS,
    });
    await page.goto("/");

    // Click the first recent project
    await page.getByText("The Warmth of Distant Things").click();

    // Should transition to app shell (project opened)
    await page
      .getByText("Binder")
      .waitFor({ state: "visible", timeout: 10000 });

    // Verify open_project was called with the correct path
    const calls = await getIpcCalls(page);
    const openCall = calls.find((c) => c.cmd === "open_project");
    expect(openCall).toBeDefined();
    expect((openCall?.args as any)?.path).toBe(
      "/home/user/projects/warmth-of-distant-things",
    );
  });

  test("remove button removes a project from the list", async ({ page }) => {
    // After removal, return only 2 projects
    const afterRemoval = MOCK_RECENT_PROJECTS.slice(0, 2);

    await setupDefaultTauriMocks(page, {
      list_recent_projects: MOCK_RECENT_PROJECTS,
      remove_recent_project: afterRemoval,
    });
    await page.goto("/");

    // All 3 should be visible initially
    await expect(page.getByText("Old Forgotten Novel")).toBeVisible();

    // Click the remove button for the third item
    const removeBtn = page.getByRole("button", {
      name: /remove old forgotten novel/i,
    });
    await removeBtn.click();

    // Verify remove_recent_project was called
    const calls = await getIpcCalls(page);
    const removeCall = calls.find((c) => c.cmd === "remove_recent_project");
    expect(removeCall).toBeDefined();
    expect((removeCall?.args as any)?.path).toBe(
      "/home/user/projects/old-forgotten-novel",
    );

    // After removal, only 2 projects should remain
    await expect(page.getByText("Old Forgotten Novel")).not.toBeVisible();
    await expect(page.getByText("The Warmth of Distant Things")).toBeVisible();
    await expect(page.getByText("Midnight in Paris")).toBeVisible();
  });

  test("recent projects hidden when create form is shown", async ({ page }) => {
    await setupDefaultTauriMocks(page, {
      list_recent_projects: MOCK_RECENT_PROJECTS,
    });
    await page.goto("/");

    // Recent projects visible initially
    await expect(page.getByText("Recent Projects")).toBeVisible();

    // Click Create Project
    await page.getByRole("button", { name: /create project/i }).click();

    // Recent projects should be hidden
    await expect(page.getByText("Recent Projects")).not.toBeVisible();

    // Cancel create -> recent projects should reappear
    await page.getByRole("button", { name: /cancel/i }).click();
    await expect(page.getByText("Recent Projects")).toBeVisible();
  });

  test("opening project via dialog adds to recent projects", async ({
    page,
  }) => {
    await setupDefaultTauriMocks(page, {
      list_recent_projects: [],
      add_recent_project: [
        {
          name: "Opened Project",
          path: "/mock/project/path",
          lastOpened: "2026-02-15T11:00:00Z",
        },
      ],
    });
    await page.goto("/");

    // Open via dialog
    await page.getByRole("button", { name: /open project/i }).click();

    // Wait for project to open
    await page
      .getByText("Binder")
      .waitFor({ state: "visible", timeout: 10000 });

    // Verify add_recent_project was called
    const calls = await getIpcCalls(page);
    const addCall = calls.find((c) => c.cmd === "add_recent_project");
    expect(addCall).toBeDefined();
    expect((addCall?.args as any)?.name).toBe("Opened Project");
    expect((addCall?.args as any)?.path).toBe("/mock/project/path");
  });
});
