import { test, expect } from "@playwright/test";
import { openMockProject } from "./utils/tauri-mocks";

// Mock data for sync tests
const MOCK_DEVICES = [
  {
    device_id: "11111111-1111-1111-1111-111111111111",
    name: "My Laptop",
    is_current: true,
  },
  {
    device_id: "22222222-2222-2222-2222-222222222222",
    name: "Device 22222222",
    is_current: false,
  },
];

const MOCK_PAIRING_CODE = {
  qr_svg:
    '<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="200" height="200" fill="#fff"/><text x="50" y="100">QR</text></svg>',
  pairing_string: "sk-pair_v1.dGVzdHBhaXJpbmdkYXRh",
};

const syncOverrides = {
  // Sync commands
  sync_connect: null,
  sync_disconnect: null,
  sync_status: "Disconnected",
  sync_enable_project: null,
  sync_disable_project: null,
  sync_send_update: null,
  // Pairing commands
  generate_pairing_code: MOCK_PAIRING_CODE,
  complete_pairing: {
    device_id: "33333333-3333-3333-3333-333333333333",
    name: "Device 33333333",
    is_current: false,
  },
  list_paired_devices: MOCK_DEVICES,
  remove_device: null,
  // Compile command (needed for some UI flows)
  compile_manuscript: { format: "html", content: "<p>test</p>", wordCount: 1 },
};

test.describe("Sync Status Indicator", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
  });

  test("shows sync status indicator in status bar", async ({ page }) => {
    const indicator = page.locator(".sync-indicator");
    await expect(indicator).toBeVisible();
  });

  test("displays 'Offline' status by default", async ({ page }) => {
    const statusLabel = page.locator(".sync-indicator .status-label");
    await expect(statusLabel).toContainText("Offline");
  });

  test("shows correct dot color for offline state", async ({ page }) => {
    const indicator = page.locator(".sync-indicator");
    await expect(indicator).toHaveClass(/status-offline/);
  });

  test("popover opens on click", async ({ page }) => {
    await page.locator(".sync-indicator").click();
    const popover = page.locator(".sync-popover");
    await expect(popover).toBeVisible();
    await expect(popover.locator(".popover-title")).toContainText("Sync Status");
  });

  test("popover shows pending updates count", async ({ page }) => {
    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover")).toContainText("0 updates");
  });

  test("popover closes on close button click", async ({ page }) => {
    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover")).toBeVisible();
    await page.locator(".popover-close").click();
    await expect(page.locator(".sync-popover")).not.toBeVisible();
  });

  test("updates status to connected via store manipulation", async ({
    page,
  }) => {
    // Manipulate the store directly to test reactivity
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      (stores as any).syncStore.connectionStatus = "connected";
    });

    const statusLabel = page.locator(".sync-indicator .status-label");
    await expect(statusLabel).toContainText("Synced");
    await expect(page.locator(".sync-indicator")).toHaveClass(/status-synced/);
  });

  test("updates status to error via store manipulation", async ({ page }) => {
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      (stores as any).syncStore.connectionStatus = "error";
      (stores as any).syncStore.lastError = "Connection refused";
    });

    const statusLabel = page.locator(".sync-indicator .status-label");
    await expect(statusLabel).toContainText("Sync Error");
    await expect(page.locator(".sync-indicator")).toHaveClass(/status-error/);

    // Open popover to check error display
    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover")).toContainText(
      "Connection refused",
    );
  });

  test("syncing state shows pulse animation class", async ({ page }) => {
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      (stores as any).syncStore.connectionStatus = "connecting";
    });

    const statusLabel = page.locator(".sync-indicator .status-label");
    await expect(statusLabel).toContainText("Connecting...");
    await expect(page.locator(".sync-indicator")).toHaveClass(/status-syncing/);
  });

  test("reconnecting state shows syncing style", async ({ page }) => {
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      (stores as any).syncStore.connectionStatus = "reconnecting";
    });

    const statusLabel = page.locator(".sync-indicator .status-label");
    await expect(statusLabel).toContainText("Reconnecting...");
    await expect(page.locator(".sync-indicator")).toHaveClass(/status-syncing/);
  });

  test("popover shows account email when logged in", async ({ page }) => {
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      (stores as any).syncStore.login({
        email: "test@example.com",
        accountId: "acc-123",
      });
    });

    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover")).toContainText(
      "test@example.com",
    );
  });
});

test.describe("Sync Status Indicator - all five states", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
  });

  const states: Array<{
    status: string;
    expectedLabel: string;
    expectedClass: string;
  }> = [
    {
      status: "connected",
      expectedLabel: "Synced",
      expectedClass: "status-synced",
    },
    {
      status: "connecting",
      expectedLabel: "Connecting...",
      expectedClass: "status-syncing",
    },
    {
      status: "disconnected",
      expectedLabel: "Offline",
      expectedClass: "status-offline",
    },
    {
      status: "reconnecting",
      expectedLabel: "Reconnecting...",
      expectedClass: "status-syncing",
    },
    {
      status: "error",
      expectedLabel: "Sync Error",
      expectedClass: "status-error",
    },
  ];

  for (const { status, expectedLabel, expectedClass } of states) {
    test(`renders ${status} state correctly`, async ({ page }) => {
      await page.evaluate(async (s) => {
        const stores = await import("/src/lib/stores/index.ts");
        (stores as any).syncStore.connectionStatus = s;
      }, status);

      await expect(page.locator(".sync-indicator .status-label")).toContainText(
        expectedLabel,
      );
      await expect(page.locator(".sync-indicator")).toHaveClass(
        new RegExp(expectedClass),
      );
    });
  }
});
