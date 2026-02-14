import { test, expect } from "@playwright/test";
import { setupDefaultTauriMocks } from "./utils/tauri-mocks";
import { takeStepScreenshot } from "./utils/screenshots";

test.beforeEach(async ({ page }) => {
  await setupDefaultTauriMocks(page);
  await page.goto("/");
});

test.describe("Visual regression", () => {
  test("launcher page appearance", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: /sakya/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /create project/i }),
    ).toBeVisible();

    await expect(page).toHaveScreenshot("launcher-page.png");
    await takeStepScreenshot(page, "launcher", "initial-load");
  });

  test("launcher page with create form open", async ({ page }) => {
    await page.getByRole("button", { name: /create project/i }).click();
    await expect(page.getByLabel(/project name/i)).toBeVisible();

    await takeStepScreenshot(page, "launcher", "create-form-open");
    await expect(page).toHaveScreenshot("launcher-create-form.png");
  });
});
