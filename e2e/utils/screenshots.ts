import type { Page, Locator } from "@playwright/test";
import { expect } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";

const SCREENSHOTS_DIR = resolve(import.meta.dirname, "..", "screenshots");

/**
 * Ensure the screenshots directory exists.
 */
function ensureDir(path: string): void {
  mkdirSync(dirname(path), { recursive: true });
}

/**
 * Take a screenshot saved to e2e/screenshots/ for AI inspection.
 * Returns the absolute path to the saved file.
 */
export async function takeAIScreenshot(
  page: Page,
  options: {
    name: string;
    element?: Locator;
    fullPage?: boolean;
  },
): Promise<string> {
  const filePath = resolve(SCREENSHOTS_DIR, `${options.name}.png`);
  ensureDir(filePath);

  if (options.element) {
    await options.element.screenshot({ path: filePath });
  } else {
    await page.screenshot({ path: filePath, fullPage: options.fullPage });
  }

  return filePath;
}

/**
 * Convenience wrapper producing e2e/screenshots/{prefix}--{step}.png.
 * Returns the absolute path to the saved file.
 */
export async function takeStepScreenshot(
  page: Page,
  testPrefix: string,
  stepName: string,
): Promise<string> {
  return takeAIScreenshot(page, { name: `${testPrefix}--${stepName}` });
}

/**
 * Set the app theme to 'light' or 'dark' via the uiState store.
 * Waits for the data-theme attribute to update on the DOM.
 */
export async function setTheme(
  page: Page,
  theme: "light" | "dark",
): Promise<void> {
  await page.evaluate(async (t) => {
    const { uiState } = await import("/src/lib/stores/index.ts");
    uiState.setTheme(t);
  }, theme);
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    theme,
  );
}
