import { test, expect, type Page } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function openSyncSettings(page: Page): Promise<void> {
  await page.locator(".sync-indicator").click();
  await page.locator(".settings-btn").click();
  await expect(
    page.locator('dialog[aria-label="Sync Settings"]'),
  ).toBeVisible();
}

async function loginViaStore(
  page: Page,
  email = "test@example.com",
): Promise<void> {
  await page.evaluate(async (e) => {
    const stores = await import("/src/lib/stores/index.ts");
    (stores as any).syncStore.login({ email: e, accountId: "acc-123" });
  }, email);
}

async function setConnected(page: Page): Promise<void> {
  await page.evaluate(async () => {
    const stores = await import("/src/lib/stores/index.ts");
    (stores as any).syncStore.connectionStatus = "connected";
  });
}

async function setServerUrl(
  page: Page,
  url = "wss://test.sync.sakya.app",
): Promise<void> {
  await page.evaluate(async (u) => {
    const stores = await import("/src/lib/stores/index.ts");
    (stores as any).syncStore.serverUrl = u;
  }, url);
}

// ===========================================================================
// Group 1: Sync Status Indicator (existing tests)
// ===========================================================================

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
    await expect(popover.locator(".popover-title")).toContainText(
      "Sync Status",
    );
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

// ===========================================================================
// Group 2: Settings Dialog Access
// ===========================================================================

test.describe("Settings Dialog Access", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
  });

  test('popover has "Sync Settings" button', async ({ page }) => {
    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover .settings-btn")).toBeVisible();
    await expect(page.locator(".sync-popover .settings-btn")).toContainText(
      "Sync Settings",
    );
  });

  test("button opens SyncSettingsDialog", async ({ page }) => {
    await page.locator(".sync-indicator").click();
    await page.locator(".settings-btn").click();
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog).toBeVisible();
    await expect(dialog.locator(".modal-title")).toContainText("Sync Settings");
    // Popover should be closed
    await expect(page.locator(".sync-popover")).not.toBeVisible();
  });

  test("closing dialog returns to app", async ({ page }) => {
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog).toBeVisible();
    await dialog.locator(".modal-close").click();
    await expect(dialog).not.toBeVisible();
  });
});

// ===========================================================================
// Group 3: AccountSettings — Login Flow
// ===========================================================================

test.describe("AccountSettings — Login Flow", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await openSyncSettings(page);
  });

  test("shows login form when logged out", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog.locator(".login-section")).toBeVisible();
    await expect(dialog.locator(".login-message")).toContainText(
      "Sign in to enable sync",
    );
    await expect(dialog.locator('input[type="email"]')).toBeVisible();
  });

  test("Send Magic Link disabled when empty", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const btn = dialog.getByRole("button", { name: /Send Magic Link/ });
    await expect(btn).toBeDisabled();
  });

  test("typing email enables button", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("user@test.com");
    const btn = dialog.getByRole("button", { name: /Send Magic Link/ });
    await expect(btn).toBeEnabled();
  });

  test("Send Magic Link shows verify screen", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("user@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    await expect(dialog.locator(".verify-section")).toBeVisible();
    await expect(dialog.locator(".verify-message")).toContainText(
      "Check your email",
    );
  });

  test('"Use a different email" returns to login', async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("user@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    await expect(dialog.locator(".verify-section")).toBeVisible();
    await dialog.locator(".btn-link").click();
    await expect(dialog.locator(".login-section")).toBeVisible();
  });

  test("empty verify code keeps Verify disabled", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("user@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    const verifyBtn = dialog.getByRole("button", { name: /^Verify$/ });
    await expect(verifyBtn).toBeDisabled();
  });

  test("entering code + Verify logs in", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("user@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    await dialog
      .locator('input[placeholder="Verification code"]')
      .fill("123456");
    await dialog.getByRole("button", { name: /^Verify$/ }).click();
    await expect(dialog.locator(".logged-in")).toBeVisible();
    await expect(dialog.locator(".account-email")).toContainText(
      "user@test.com",
    );
  });

  test("logged-in shows email + Log Out", async ({ page }) => {
    // Login via store, then reopen
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator(".modal-close").click();
    await loginViaStore(page);
    await openSyncSettings(page);
    const reopened = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(reopened.locator(".account-email")).toContainText(
      "test@example.com",
    );
    await expect(
      reopened.getByRole("button", { name: /Log Out/ }),
    ).toBeVisible();
  });
});

// ===========================================================================
// Group 4: AccountSettings — Logout
// ===========================================================================

test.describe("AccountSettings — Logout", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await openSyncSettings(page);
  });

  test("Log Out returns to login form", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.getByRole("button", { name: /Log Out/ }).click();
    await expect(dialog.locator(".login-section")).toBeVisible();
  });

  test("Log Out clears store account", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.getByRole("button", { name: /Log Out/ }).click();
    const account = await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      return (stores as any).syncStore.account;
    });
    expect(account).toBeNull();
  });

  test("popover no longer shows email after logout", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.getByRole("button", { name: /Log Out/ }).click();
    await dialog.locator(".modal-close").click();
    await page.locator(".sync-indicator").click();
    const popover = page.locator(".sync-popover");
    await expect(popover).toBeVisible();
    await expect(popover).not.toContainText("test@example.com");
  });
});

// ===========================================================================
// Group 5: DeviceManager — Device List
// ===========================================================================

test.describe("DeviceManager — Device List", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await openSyncSettings(page);
  });

  test("shows all paired devices", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const items = dialog.locator(".device-item");
    await expect(items).toHaveCount(2);
  });

  test('current device has "This device" badge', async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const laptopRow = dialog
      .locator(".device-item")
      .filter({ hasText: "My Laptop" });
    await expect(laptopRow.locator(".badge-current")).toContainText(
      "This device",
    );
  });

  test("non-current shows truncated ID", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const otherRow = dialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await expect(otherRow.locator(".device-id")).toContainText("22222222...");
  });

  test("current device has no Remove button", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const laptopRow = dialog
      .locator(".device-item")
      .filter({ hasText: "My Laptop" });
    await expect(laptopRow.locator(".btn-remove")).not.toBeVisible();
  });

  test("non-current shows Remove button", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const otherRow = dialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await expect(otherRow.locator(".btn-remove")).toBeVisible();
  });
});

// ===========================================================================
// Group 6: DeviceManager — Remove Device
// ===========================================================================

test.describe("DeviceManager — Remove Device", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await openSyncSettings(page);
  });

  test("Remove opens ConfirmDialog", async ({ page }) => {
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    const otherRow = settingsDialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await otherRow.locator(".btn-remove").click();
    const confirmDialog = page.locator('dialog[aria-label="Remove Device"]');
    await expect(confirmDialog).toBeVisible();
    await expect(confirmDialog.locator(".modal-title")).toContainText(
      "Remove Device",
    );
  });

  test("confirmation warns about key rotation", async ({ page }) => {
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    const otherRow = settingsDialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await otherRow.locator(".btn-remove").click();
    const confirmDialog = page.locator('dialog[aria-label="Remove Device"]');
    await expect(confirmDialog).toContainText("rotate all encryption keys");
  });

  test("confirming removes device and calls IPC", async ({ page }) => {
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    const otherRow = settingsDialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await clearIpcCalls(page);
    await otherRow.locator(".btn-remove").click();
    const confirmDialog = page.locator('dialog[aria-label="Remove Device"]');
    await confirmDialog.locator(".btn-confirm").click();
    // Device should disappear
    await expect(
      settingsDialog.locator(".device-item").filter({ hasText: "Device 22222222" }),
    ).not.toBeVisible();
    // IPC called
    const calls = await getIpcCallsByCommand(page, "remove_device");
    expect(calls.length).toBeGreaterThanOrEqual(1);
  });

  test("cancel keeps device in list", async ({ page }) => {
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    const otherRow = settingsDialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await otherRow.locator(".btn-remove").click();
    const confirmDialog = page.locator('dialog[aria-label="Remove Device"]');
    await confirmDialog.locator(".btn-cancel").click();
    // Device still listed
    await expect(
      settingsDialog
        .locator(".device-item")
        .filter({ hasText: "Device 22222222" }),
    ).toBeVisible();
  });
});

// ===========================================================================
// Group 7: DeviceManager — Empty State
// ===========================================================================

test.describe("DeviceManager — Empty State", () => {
  test("empty list shows placeholder", async ({ page }) => {
    await openMockProject(page, {
      ...syncOverrides,
      list_paired_devices: [],
    });
    await loginViaStore(page);
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog.locator(".device-empty")).toContainText(
      "No devices paired yet.",
    );
  });
});

// ===========================================================================
// Group 8: PairingDialog — Show Code
// ===========================================================================

test.describe("PairingDialog — Show Code", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await setServerUrl(page);
    await openSyncSettings(page);
    // Click "Add Device" to open PairingDialog
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    await settingsDialog.getByRole("button", { name: /Add Device/ }).click();
  });

  test("Add Device opens PairingDialog", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await expect(pairingDialog).toBeVisible();
    await expect(pairingDialog.locator(".modal-title")).toContainText(
      "Pair Device",
    );
  });

  test("Show Code tab active by default", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    const showTab = pairingDialog.locator('[role="tab"]', {
      hasText: "Show Code",
    });
    await expect(showTab).toHaveAttribute("aria-selected", "true");
  });

  test("QR SVG rendered", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await expect(pairingDialog.locator(".qr-container svg")).toBeVisible();
  });

  test("pairing string displayed", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await expect(pairingDialog.locator(".pairing-string")).toContainText(
      "sk-pair_v1.",
    );
  });

  test("help text visible", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await expect(pairingDialog).toContainText("Scan the QR code or share");
  });
});

// ===========================================================================
// Group 9: PairingDialog — Enter Code
// ===========================================================================

test.describe("PairingDialog — Enter Code", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await setServerUrl(page);
    await openSyncSettings(page);
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    await settingsDialog.getByRole("button", { name: /Add Device/ }).click();
    await expect(
      page.locator('dialog[aria-label="Pair Device"]'),
    ).toBeVisible();
  });

  test("Enter Code tab switches panel", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await expect(
      pairingDialog.locator('input[placeholder="sk-pair_v1...."]'),
    ).toBeVisible();
  });

  test("placeholder correct", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    const input = pairingDialog.locator(".enter-code input");
    await expect(input).toHaveAttribute("placeholder", "sk-pair_v1....");
  });

  test("Pair Device disabled when empty", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    const btn = pairingDialog.getByRole("button", { name: /Pair Device/ });
    await expect(btn).toBeDisabled();
  });

  test("typing enables Pair Device", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await pairingDialog
      .locator('input[placeholder="sk-pair_v1...."]')
      .fill("sk-pair_v1.abc123");
    const btn = pairingDialog.getByRole("button", { name: /Pair Device/ });
    await expect(btn).toBeEnabled();
  });

  test("submit calls complete_pairing IPC and closes", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await clearIpcCalls(page);
    await pairingDialog
      .locator('input[placeholder="sk-pair_v1...."]')
      .fill("sk-pair_v1.abc123");
    await pairingDialog.getByRole("button", { name: /Pair Device/ }).click();
    // Dialog should close
    await expect(pairingDialog).not.toBeVisible();
    // IPC called
    const calls = await getIpcCallsByCommand(page, "complete_pairing");
    expect(calls.length).toBe(1);
    expect((calls[0].args as any).remotePairingCode).toBe(
      "sk-pair_v1.abc123",
    );
  });
});

// ===========================================================================
// Group 10: PairingDialog — Tab Switching & Close
// ===========================================================================

test.describe("PairingDialog — Tab Switching & Close", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await loginViaStore(page);
    await setServerUrl(page);
    await openSyncSettings(page);
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    await settingsDialog.getByRole("button", { name: /Add Device/ }).click();
    await expect(
      page.locator('dialog[aria-label="Pair Device"]'),
    ).toBeVisible();
  });

  test("tab switch preserves input", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    // Switch to Enter Code
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await pairingDialog
      .locator('input[placeholder="sk-pair_v1...."]')
      .fill("sk-pair_v1.preserve-me");
    // Switch to Show Code
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Show Code" })
      .click();
    // Switch back to Enter Code
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    // Input should be preserved
    await expect(
      pairingDialog.locator('input[placeholder="sk-pair_v1...."]'),
    ).toHaveValue("sk-pair_v1.preserve-me");
  });

  test("close + reopen resets", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    // Switch to Enter Code and type
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await pairingDialog
      .locator('input[placeholder="sk-pair_v1...."]')
      .fill("sk-pair_v1.some-code");
    // Close
    await pairingDialog.locator(".modal-close").click();
    await expect(pairingDialog).not.toBeVisible();
    // Reopen via Add Device
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    await settingsDialog.getByRole("button", { name: /Add Device/ }).click();
    const reopened = page.locator('dialog[aria-label="Pair Device"]');
    await expect(reopened).toBeVisible();
    // Show Code tab should be active
    const showTab = reopened.locator('[role="tab"]', {
      hasText: "Show Code",
    });
    await expect(showTab).toHaveAttribute("aria-selected", "true");
    // Switch to Enter Code — input should be empty
    await reopened
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await expect(
      reopened.locator('input[placeholder="sk-pair_v1...."]'),
    ).toHaveValue("");
  });

  test("close via X button", async ({ page }) => {
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog.locator(".modal-close").click();
    await expect(pairingDialog).not.toBeVisible();
  });
});

// ===========================================================================
// Group 11: ProjectSyncSettings — Disabled State
// ===========================================================================

test.describe("ProjectSyncSettings — Disabled State", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await openSyncSettings(page);
  });

  test("toggle disabled when not connected", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const toggle = dialog.locator('[role="switch"]');
    await expect(toggle).toBeDisabled();
  });

  test("warning message shown", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog.locator(".warning-message")).toContainText(
      "Connect to the sync server",
    );
  });

  test('description shows "Enable sync"', async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog.locator(".toggle-description")).toContainText(
      "Enable sync to collaborate",
    );
  });
});

// ===========================================================================
// Group 12: ProjectSyncSettings — Enable/Disable
// ===========================================================================

test.describe("ProjectSyncSettings — Enable/Disable", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await setConnected(page);
    await openSyncSettings(page);
  });

  test("toggle enabled when connected", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const toggle = dialog.locator('[role="switch"]');
    await expect(toggle).toBeEnabled();
  });

  test("click toggle enables sync", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await clearIpcCalls(page);
    const toggle = dialog.locator('[role="switch"]');
    await toggle.click();
    // IPC called
    const calls = await getIpcCallsByCommand(page, "sync_enable_project");
    expect(calls.length).toBe(1);
    // Toggle should be active
    await expect(toggle).toHaveAttribute("aria-checked", "true");
  });

  test("sync details appear after enabling", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const toggle = dialog.locator('[role="switch"]');
    await toggle.click();
    await expect(dialog.locator(".sync-details")).toBeVisible();
    await expect(dialog.locator(".sync-details")).toContainText(
      "Not yet synced",
    );
    await expect(dialog.locator(".sync-details")).toContainText("0");
  });

  test("click again disables sync", async ({ page }) => {
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    const toggle = dialog.locator('[role="switch"]');
    // Enable first
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-checked", "true");
    await clearIpcCalls(page);
    // Disable
    await toggle.click();
    const calls = await getIpcCallsByCommand(page, "sync_disable_project");
    expect(calls.length).toBe(1);
    await expect(toggle).toHaveAttribute("aria-checked", "false");
  });
});

// ===========================================================================
// Group 13: ProjectSyncSettings — Details
// ===========================================================================

test.describe("ProjectSyncSettings — Details", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
    await setConnected(page);
  });

  test("last sync time displayed", async ({ page }) => {
    // Enable sync and set last sync time via store
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      const projectPath = (stores as any).projectState.projectPath;
      const syncStore = (stores as any).syncStore;
      syncStore.syncedProjects = new Map([
        [
          projectPath,
          {
            projectId: projectPath,
            enabled: true,
            lastSyncTime: "2026-02-18T10:30:00Z",
            pendingUpdates: 0,
          },
        ],
      ]);
    });
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    // Should show a formatted time (locale-dependent, so check for digits)
    await expect(dialog.locator(".sync-details")).toBeVisible();
    await expect(
      dialog.locator(".detail-value").first(),
    ).not.toContainText("Not yet synced");
  });

  test("pending update count displayed", async ({ page }) => {
    await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      const projectPath = (stores as any).projectState.projectPath;
      const syncStore = (stores as any).syncStore;
      syncStore.syncedProjects = new Map([
        [
          projectPath,
          {
            projectId: projectPath,
            enabled: true,
            lastSyncTime: null,
            pendingUpdates: 5,
          },
        ],
      ]);
    });
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await expect(dialog.locator(".sync-details")).toContainText("5");
  });
});

// ===========================================================================
// Group 14: Cross-Component Journeys
// ===========================================================================

test.describe("Cross-Component Journeys", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
  });

  test("full login: email -> magic link -> verify -> logged in", async ({
    page,
  }) => {
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    // Fill email
    await dialog.locator('input[type="email"]').fill("journey@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    // Verify
    await dialog
      .locator('input[placeholder="Verification code"]')
      .fill("abc123");
    await dialog.getByRole("button", { name: /^Verify$/ }).click();
    // Logged in
    await expect(dialog.locator(".logged-in")).toBeVisible();
    await expect(dialog.locator(".account-email")).toContainText(
      "journey@test.com",
    );
  });

  test("login then view devices", async ({ page }) => {
    await loginViaStore(page);
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    // Devices should be loaded
    await expect(dialog.locator(".device-item")).toHaveCount(2);
    await expect(
      dialog.locator(".device-item").filter({ hasText: "My Laptop" }),
    ).toBeVisible();
    await expect(
      dialog
        .locator(".device-item")
        .filter({ hasText: "My Laptop" })
        .locator(".badge-current"),
    ).toBeVisible();
  });

  test("login, add device, pair, new device appears", async ({ page }) => {
    await loginViaStore(page);
    await setServerUrl(page);
    await openSyncSettings(page);
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    // Initially 2 devices
    await expect(settingsDialog.locator(".device-item")).toHaveCount(2);
    // Add device via pairing
    await settingsDialog
      .getByRole("button", { name: /Add Device/ })
      .click();
    const pairingDialog = page.locator('dialog[aria-label="Pair Device"]');
    await pairingDialog
      .locator('[role="tab"]', { hasText: "Enter Code" })
      .click();
    await pairingDialog
      .locator('input[placeholder="sk-pair_v1...."]')
      .fill("sk-pair_v1.newdevice");
    await pairingDialog.getByRole("button", { name: /Pair Device/ }).click();
    // Pairing dialog closes
    await expect(pairingDialog).not.toBeVisible();
    // New device appears (3 total)
    await expect(settingsDialog.locator(".device-item")).toHaveCount(3);
  });

  test("logout clears everything", async ({ page }) => {
    await loginViaStore(page);
    await setConnected(page);
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    // Verify logged in state
    await expect(dialog.locator(".logged-in")).toBeVisible();
    // Log out
    await dialog.getByRole("button", { name: /Log Out/ }).click();
    // Check store state
    const state = await page.evaluate(async () => {
      const stores = await import("/src/lib/stores/index.ts");
      return {
        account: (stores as any).syncStore.account,
        devices: (stores as any).syncStore.devices,
      };
    });
    expect(state.account).toBeNull();
    expect(state.devices).toHaveLength(0);
  });

  test("popover updates after settings changes", async ({ page }) => {
    // Login via settings dialog
    await openSyncSettings(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('input[type="email"]').fill("popover@test.com");
    await dialog.getByRole("button", { name: /Send Magic Link/ }).click();
    await dialog
      .locator('input[placeholder="Verification code"]')
      .fill("token");
    await dialog.getByRole("button", { name: /^Verify$/ }).click();
    await expect(dialog.locator(".logged-in")).toBeVisible();
    // Close dialog
    await dialog.locator(".modal-close").click();
    // Open popover — should show email
    await page.locator(".sync-indicator").click();
    await expect(page.locator(".sync-popover")).toContainText(
      "popover@test.com",
    );
  });
});

// ===========================================================================
// Group 15: IPC Verification
// ===========================================================================

test.describe("IPC Verification", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page, syncOverrides);
  });

  test("login calls no sync IPC commands", async ({ page }) => {
    await clearIpcCalls(page);
    await loginViaStore(page);
    const syncCmds = [
      "sync_connect",
      "sync_disconnect",
      "sync_enable_project",
      "sync_disable_project",
    ];
    for (const cmd of syncCmds) {
      const calls = await getIpcCallsByCommand(page, cmd);
      expect(calls).toHaveLength(0);
    }
  });

  test("enable_project sends correct projectId", async ({ page }) => {
    await setConnected(page);
    await openSyncSettings(page);
    await clearIpcCalls(page);
    const dialog = page.locator('dialog[aria-label="Sync Settings"]');
    await dialog.locator('[role="switch"]').click();
    const calls = await getIpcCallsByCommand(page, "sync_enable_project");
    expect(calls.length).toBe(1);
    // projectId should be the mock project path
    expect((calls[0].args as any).projectId).toBeTruthy();
  });

  test("remove_device sends correct deviceId", async ({ page }) => {
    await loginViaStore(page);
    await openSyncSettings(page);
    await clearIpcCalls(page);
    const settingsDialog = page.locator(
      'dialog[aria-label="Sync Settings"]',
    );
    const otherRow = settingsDialog
      .locator(".device-item")
      .filter({ hasText: "Device 22222222" });
    await otherRow.locator(".btn-remove").click();
    const confirmDialog = page.locator('dialog[aria-label="Remove Device"]');
    await confirmDialog.locator(".btn-confirm").click();
    const calls = await getIpcCallsByCommand(page, "remove_device");
    expect(calls.length).toBe(1);
    expect((calls[0].args as any).deviceId).toBe(
      "22222222-2222-2222-2222-222222222222",
    );
  });
});
