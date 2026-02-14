import { test, expect } from "@playwright/test";
import { setupDefaultTauriMocks } from "./utils/tauri-mocks";
import { takeStepScreenshot } from "./utils/screenshots";

test.beforeEach(async ({ page }) => {
  await setupDefaultTauriMocks(page);
  await page.goto("/");
});

test.describe("Visual regression", () => {
  test("home page matches baseline", async ({ page }) => {
    await expect(page.getByAltText("Vite Logo")).toBeVisible();
    await expect(page.getByAltText("Tauri Logo")).toBeVisible();
    await expect(page.getByAltText("SvelteKit Logo")).toBeVisible();

    await expect(page).toHaveScreenshot("home-page.png");
    await takeStepScreenshot(page, "home", "initial-load");
  });

  test("greet form after submission", async ({ page }) => {
    await takeStepScreenshot(page, "greet", "before-submit");

    const input = page.getByPlaceholder("Enter a name...");
    const button = page.getByRole("button", { name: /greet/i });

    await input.fill("Visual Test");
    await button.click();
    await expect(
      page.getByText("Hello, Visual Test! You've been greeted from Rust!"),
    ).toBeVisible();

    await expect(page).toHaveScreenshot("greet-result.png");
    await takeStepScreenshot(page, "greet", "after-submit");
  });

  test("logo row layout", async ({ page }) => {
    const logoRow = page.locator(".row").first();
    await expect(logoRow).toBeVisible();
    await expect(page.getByAltText("Tauri Logo")).toBeVisible();

    await expect(logoRow).toHaveScreenshot("logo-row.png");
  });

  test("form elements layout", async ({ page }) => {
    const form = page.locator("form");
    await expect(form).toBeVisible();

    await expect(form).toHaveScreenshot("form-row.png");
  });
});
