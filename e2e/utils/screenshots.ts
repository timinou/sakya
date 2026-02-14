import type { Page, Locator } from "@playwright/test";
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
