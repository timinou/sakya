import { vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

const mockedInvoke = vi.mocked(invoke);

/**
 * Mock a single Tauri invoke command with a static return value.
 */
export function mockInvoke(cmd: string, returnValue: unknown): void {
  mockedInvoke.mockImplementation(async (command: string) => {
    if (command === cmd) return returnValue;
    throw new Error(`Unexpected invoke call: ${command}`);
  });
}

/**
 * Mock a single Tauri invoke command with a handler function.
 */
export function mockInvokeWith(
  cmd: string,
  handler: (args?: Record<string, unknown>) => unknown,
): void {
  mockedInvoke.mockImplementation(
    async (command: string, args?: Record<string, unknown>) => {
      if (command === cmd) return handler(args);
      throw new Error(`Unexpected invoke call: ${command}`);
    },
  );
}

/**
 * Mock multiple Tauri invoke commands at once.
 */
export function mockInvokeMultiple(
  mocks: Record<string, unknown | ((args?: Record<string, unknown>) => unknown)>,
): void {
  mockedInvoke.mockImplementation(
    async (command: string, args?: Record<string, unknown>) => {
      if (command in mocks) {
        const mock = mocks[command];
        return typeof mock === "function" ? mock(args) : mock;
      }
      throw new Error(`Unexpected invoke call: ${command}`);
    },
  );
}

/**
 * Reset all invoke mocks.
 */
export function resetInvokeMocks(): void {
  mockedInvoke.mockReset();
}

/**
 * Get the mocked invoke for direct assertions.
 */
export { mockedInvoke };
