import type { Page } from "@playwright/test";

type MockHandler = (args?: Record<string, unknown>) => unknown;
type MockValue = unknown | MockHandler;

/**
 * Set up Tauri IPC mocks by injecting a fake __TAURI_INTERNALS__ object.
 * This intercepts invoke() calls in the browser context.
 */
export async function setupTauriMocks(
  page: Page,
  mocks: Record<string, MockValue>,
): Promise<void> {
  // Serialize mocks: functions become strings, values become JSON
  const serializedMocks: Record<
    string,
    { type: "function"; body: string } | { type: "value"; data: unknown }
  > = {};
  for (const [cmd, mock] of Object.entries(mocks)) {
    if (typeof mock === "function") {
      serializedMocks[cmd] = { type: "function", body: mock.toString() };
    } else {
      serializedMocks[cmd] = { type: "value", data: mock };
    }
  }

  await page.addInitScript((mocks) => {
    const ipcCalls: Array<{ cmd: string; args?: unknown }> = [];
    (window as any).__TAURI_IPC_CALLS__ = ipcCalls;

    (window as any).__TAURI_INTERNALS__ = {
      transformCallback(callback: Function) {
        const id = window.crypto.randomUUID();
        (window as any)[`_${id}`] = callback;
        return id;
      },
      convertFileSrc(filePath: string) {
        return `asset://localhost/${filePath}`;
      },
      invoke(cmd: string, args?: Record<string, unknown>) {
        ipcCalls.push({ cmd, args });
        const mock = mocks[cmd];
        if (!mock) {
          return Promise.reject(
            new Error(`No mock registered for command: ${cmd}`),
          );
        }
        if (mock.type === "function") {
          // eslint-disable-next-line no-eval
          const fn = eval(`(${mock.body})`);
          return Promise.resolve(fn(args));
        }
        return Promise.resolve(mock.data);
      },
    };
  }, serializedMocks);
}

/**
 * Set up default Tauri mocks for the current app commands.
 */
export async function setupDefaultTauriMocks(
  page: Page,
  overrides: Record<string, MockValue> = {},
): Promise<void> {
  const defaults: Record<string, MockValue> = {
    greet: (args: Record<string, unknown> | undefined) =>
      `Hello, ${args?.name ?? ""}! You've been greeted from Rust!`,
    create_project: (args: Record<string, unknown> | undefined) => ({
      name: args?.name ?? "Test Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }),
    open_project: () => ({
      name: "Opened Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }),
    save_project_manifest: () => null,
    "plugin:dialog|open": () => "/mock/project/path",
  };
  await setupTauriMocks(page, { ...defaults, ...overrides });
}

/**
 * Get all IPC calls recorded during the test.
 */
export async function getIpcCalls(
  page: Page,
): Promise<Array<{ cmd: string; args?: unknown }>> {
  return await page.evaluate(
    () => (window as any).__TAURI_IPC_CALLS__ ?? [],
  );
}
