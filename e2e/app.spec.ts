import { test, expect } from "@playwright/test";
import { setupDefaultTauriMocks, getIpcCalls } from "./utils/tauri-mocks";

test.beforeEach(async ({ page }) => {
  await setupDefaultTauriMocks(page);
  await page.goto("/");
});

test.describe("App loads correctly", () => {
  test("displays the welcome heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: /welcome to tauri/i }),
    ).toBeVisible();
  });

  test("displays all three logos", async ({ page }) => {
    await expect(page.getByAltText("Vite Logo")).toBeVisible();
    await expect(page.getByAltText("Tauri Logo")).toBeVisible();
    await expect(page.getByAltText("SvelteKit Logo")).toBeVisible();
  });

  test("logos link to correct sites", async ({ page }) => {
    await expect(page.locator('a[href="https://vite.dev"]')).toBeVisible();
    await expect(page.locator('a[href="https://tauri.app"]')).toBeVisible();
    await expect(page.locator('a[href="https://svelte.dev"]')).toBeVisible();
  });
});

test.describe("Greet form", () => {
  test("submits and displays greeting", async ({ page }) => {
    const input = page.getByPlaceholder("Enter a name...");
    const button = page.getByRole("button", { name: /greet/i });

    await input.fill("World");
    await button.click();

    await expect(
      page.getByText("Hello, World! You've been greeted from Rust!"),
    ).toBeVisible();
  });

  test("records IPC call with correct arguments", async ({ page }) => {
    const input = page.getByPlaceholder("Enter a name...");
    const button = page.getByRole("button", { name: /greet/i });

    await input.fill("Sakya");
    await button.click();

    // Wait for the greeting to appear, confirming the IPC call completed
    await expect(
      page.getByText("Hello, Sakya! You've been greeted from Rust!"),
    ).toBeVisible();

    const calls = await getIpcCalls(page);
    expect(calls).toContainEqual({
      cmd: "greet",
      args: { name: "Sakya" },
    });
  });

  test("handles empty name submission", async ({ page }) => {
    const button = page.getByRole("button", { name: /greet/i });
    await button.click();

    await expect(
      page.getByText("Hello, ! You've been greeted from Rust!"),
    ).toBeVisible();
  });
});
