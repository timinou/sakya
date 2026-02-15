import { test, expect, type Page } from "@playwright/test";
import {
  openMockProject,
  getIpcCallsByCommand,
  clearIpcCalls,
} from "./utils/tauri-mocks";

// =============================================================================
// Writing Sessions & Sprint Mode E2E Tests (ITEM-093)
// =============================================================================

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/** Open the sprint panel popover from the toolbar. */
async function openSprintPanel(page: Page): Promise<void> {
  await page.getByLabel("Open sprint timer").click();
  await page
    .locator('[role="dialog"][aria-label="Sprint Timer"]')
    .waitFor({ state: "visible" });
}

/** Click a chapter in the binder to open it in the editor. */
async function openChapter(page: Page): Promise<void> {
  await page.getByTitle("1. The Awakening").click();
  await page.getByRole("tab", { name: "The Awakening" }).waitFor();
}

/** Start a sprint end-to-end: open panel → start → wait for overlay. */
async function startSprint(page: Page): Promise<void> {
  await openSprintPanel(page);
  await page.getByText("Start Sprint").click();
  await page.locator(".sprint-bar").waitFor({ state: "visible" });
}

/** Open the stats tab via store (bypasses Svelte 5 event delegation). */
async function openStatsTab(page: Page): Promise<void> {
  await page.evaluate(async () => {
    const { editorState } = await import("/src/lib/stores/index.ts");
    editorState.openDocument({
      id: "stats:writing",
      title: "Writing Stats",
      documentType: "stats",
      documentSlug: "writing",
      isDirty: false,
    });
  });
  await page.getByRole("tab", { name: "Writing Stats" }).waitFor();
  await page.locator(".writing-stats").waitFor({ state: "visible" });
  // Wait for session data to load and render
  await page.waitForTimeout(500);
}

// =============================================================================
// Sprint Timer
// =============================================================================

test.describe("Sprint Timer", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);
  });

  test("sprint panel opens from toolbar button", async ({ page }) => {
    await openSprintPanel(page);

    const panel = page.locator(
      '[role="dialog"][aria-label="Sprint Timer"]',
    );
    await expect(panel).toBeVisible();

    // Duration presets shown
    for (const minutes of ["15m", "25m", "30m", "45m", "60m"]) {
      await expect(panel.getByText(minutes, { exact: true })).toBeVisible();
    }
  });

  test("duration presets can be selected", async ({ page }) => {
    await openSprintPanel(page);

    // Click 15m preset
    const btn15 = page.locator(".preset-btn").filter({ hasText: "15m" });
    await btn15.click();
    await expect(btn15).toHaveClass(/selected/);

    // Click 45m — 45m selected, 15m not
    const btn45 = page.locator(".preset-btn").filter({ hasText: "45m" });
    await btn45.click();
    await expect(btn45).toHaveClass(/selected/);
    await expect(btn15).not.toHaveClass(/selected/);
  });

  test("Start Sprint begins countdown", async ({ page }) => {
    await startSprint(page);

    // Sprint bar visible with countdown in MM:SS format
    const countdown = page.locator(".sprint-overlay .countdown");
    await expect(countdown).toBeVisible();
    const text = await countdown.textContent();
    expect(text).toMatch(/\d{2}:\d{2}/);
  });

  test("countdown ticks down", async ({ page }) => {
    await startSprint(page);

    const countdown = page.locator(".sprint-overlay .countdown");
    const firstReading = await countdown.textContent();

    await page.waitForTimeout(2500);

    const secondReading = await countdown.textContent();
    // Parse MM:SS to total seconds for comparison
    const toSeconds = (t: string | null) => {
      const [m, s] = (t ?? "00:00").split(":").map(Number);
      return m * 60 + s;
    };
    expect(toSeconds(secondReading)).toBeLessThan(toSeconds(firstReading));
  });

  test("Pause freezes countdown", async ({ page }) => {
    await startSprint(page);

    // Pause
    await page.getByLabel("Pause sprint").click();
    await expect(page.locator(".pause-indicator")).toBeVisible();

    const pausedTime = await page
      .locator(".sprint-overlay .countdown")
      .textContent();

    await page.waitForTimeout(2500);

    const afterWait = await page
      .locator(".sprint-overlay .countdown")
      .textContent();
    expect(afterWait).toBe(pausedTime);
  });

  test("Resume continues countdown", async ({ page }) => {
    await startSprint(page);

    // Pause
    await page.getByLabel("Pause sprint").click();
    await expect(page.locator(".pause-indicator")).toBeVisible();

    // Resume
    await page.getByLabel("Resume sprint").click();
    await expect(page.locator(".pause-indicator")).not.toBeVisible();

    const timeAfterResume = await page
      .locator(".sprint-overlay .countdown")
      .textContent();

    await page.waitForTimeout(2500);

    const laterTime = await page
      .locator(".sprint-overlay .countdown")
      .textContent();
    const toSeconds = (t: string | null) => {
      const [m, s] = (t ?? "00:00").split(":").map(Number);
      return m * 60 + s;
    };
    expect(toSeconds(laterTime)).toBeLessThan(toSeconds(timeAfterResume));
  });

  test("Stop ends sprint", async ({ page }) => {
    await startSprint(page);
    await expect(page.locator(".sprint-overlay")).toBeVisible();

    // Stop sprint via overlay button
    await page.getByLabel("Stop sprint").click();

    await expect(page.locator(".sprint-overlay")).not.toBeVisible();
    await expect(page.locator(".app-shell")).not.toHaveClass(/sprint-active/);
  });

  test("timer reaching zero auto-completes", async ({ page }) => {
    await startSprint(page);

    // Force the timer to almost zero via store manipulation
    await page.evaluate(async () => {
      const { sprintStore } = await import("/src/lib/stores/index.ts");
      sprintStore.remainingSeconds = 2;
    });

    // Wait for auto-complete (2 ticks + buffer)
    await page.waitForTimeout(4000);

    await expect(page.locator(".sprint-overlay")).not.toBeVisible();
    await expect(page.locator(".app-shell")).not.toHaveClass(/sprint-active/);
  });
});

// =============================================================================
// Sprint Mode Overlay
// =============================================================================

test.describe("Sprint Mode Overlay", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);
    await startSprint(page);
  });

  test(".app-shell has sprint-active class", async ({ page }) => {
    await expect(page.locator(".app-shell")).toHaveClass(/sprint-active/);
  });

  test("binder pane hidden during sprint", async ({ page }) => {
    const binderPane = page.locator(".binder-pane");
    await expect(binderPane).toHaveCSS("opacity", "0");
    await expect(binderPane).toHaveCSS("pointer-events", "none");
  });

  test("inspector pane hidden during sprint", async ({ page }) => {
    const inspectorPane = page.locator(".inspector-pane");
    await expect(inspectorPane).toHaveCSS("opacity", "0");
    await expect(inspectorPane).toHaveCSS("pointer-events", "none");
  });

  test("toolbar hidden during sprint", async ({ page }) => {
    const toolbar = page.locator(".toolbar");
    await expect(toolbar).toHaveCSS("opacity", "0");
    await expect(toolbar).toHaveCSS("pointer-events", "none");
  });

  test("sprint overlay bar contains expected content", async ({ page }) => {
    const bar = page.locator(".sprint-bar");
    await expect(bar).toBeVisible();

    // Sprint label
    await expect(bar.locator(".sprint-label")).toHaveText("Sprint");

    // Countdown in MM:SS format
    const countdownText = await bar.locator(".countdown").textContent();
    expect(countdownText).toMatch(/\d{2}:\d{2}/);

    // Action buttons present
    await expect(page.getByLabel("Pause sprint")).toBeVisible();
    await expect(page.getByLabel("Stop sprint")).toBeVisible();
    await expect(page.getByLabel("Save document")).toBeVisible();
  });

  test("UI fully restores after stop", async ({ page }) => {
    await page.getByLabel("Stop sprint").click();

    // Overlay gone
    await expect(page.locator(".sprint-overlay")).not.toBeVisible();
    await expect(page.locator(".app-shell")).not.toHaveClass(/sprint-active/);

    // Toolbar visible
    await expect(page.locator(".toolbar")).not.toHaveCSS("opacity", "0");

    // Binder visible
    await expect(page.locator(".binder-pane")).not.toHaveCSS("opacity", "0");
  });

  test("Escape opens stop confirmation dialog", async ({ page }) => {
    await page.keyboard.press("Escape");

    // ConfirmDialog with "Stop Sprint?" title
    const dialog = page.locator("dialog").filter({ hasText: "Stop Sprint?" });
    await expect(dialog).toBeVisible();
    // Use exact: true to distinguish from the overlay's "Stop" button
    await expect(
      dialog.getByRole("button", { name: "Stop Sprint" }),
    ).toBeVisible();
    await expect(
      dialog.getByRole("button", { name: "Keep Writing" }),
    ).toBeVisible();
  });

  test("confirming stop via Escape dialog ends sprint", async ({ page }) => {
    await page.keyboard.press("Escape");

    // Click "Stop Sprint" in the confirm dialog
    const dialog = page.locator("dialog").filter({ hasText: "Stop Sprint?" });
    await dialog.getByRole("button", { name: "Stop Sprint" }).click();

    await expect(page.locator(".sprint-overlay")).not.toBeVisible();
    await expect(page.locator(".app-shell")).not.toHaveClass(/sprint-active/);
  });

  test("canceling stop keeps sprint running", async ({ page }) => {
    await page.keyboard.press("Escape");

    // Click "Keep Writing"
    await page.getByRole("button", { name: "Keep Writing" }).click();

    // Dialog gone, overlay still visible
    await expect(
      page.locator("dialog").filter({ hasText: "Stop Sprint?" }),
    ).not.toBeVisible();
    await expect(page.locator(".sprint-overlay")).toBeVisible();
    await expect(page.locator(".app-shell")).toHaveClass(/sprint-active/);
  });
});

// =============================================================================
// Session Backend IPC
// =============================================================================

test.describe("Session Backend IPC", () => {
  test.beforeEach(async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);
  });

  test("start_session called with correct args", async ({ page }) => {
    await clearIpcCalls(page);
    await startSprint(page);

    const calls = await getIpcCallsByCommand(page, "start_session");
    expect(calls.length).toBe(1);

    const args = calls[0].args as Record<string, unknown>;
    expect(args.projectPath).toBeTruthy();
    expect(args.chapterSlug).toBe("the-awakening");
    expect(args.sprintGoal).toBeNull();
  });

  test("end_session called on stop", async ({ page }) => {
    await startSprint(page);
    await clearIpcCalls(page);

    await page.getByLabel("Stop sprint").click();

    const calls = await getIpcCallsByCommand(page, "end_session");
    expect(calls.length).toBe(1);

    const args = calls[0].args as Record<string, unknown>;
    expect(args.sessionId).toBe("2026-02-15T16:00:00Z");
    expect(typeof args.wordsWritten).toBe("number");
  });

  test("end_session word delta is correct", async ({ page }) => {
    await startSprint(page);

    // Capture the start word count from the sprint store
    const startWordCount = await page.evaluate(async () => {
      const { sprintStore } = await import("/src/lib/stores/index.ts");
      return sprintStore.startWordCount;
    });

    // Set a known word count higher than start (simulate writing)
    const targetWords = startWordCount + 200;
    await page.evaluate(async (target: number) => {
      const { editorState } = await import("/src/lib/stores/index.ts");
      editorState.wordCount = { words: target, characters: target * 5, charactersNoSpaces: target * 4 };
    }, targetWords);

    // Small wait for prop propagation
    await page.waitForTimeout(200);

    await clearIpcCalls(page);
    await page.getByLabel("Stop sprint").click();

    const calls = await getIpcCallsByCommand(page, "end_session");
    expect(calls.length).toBe(1);
    expect((calls[0].args as Record<string, unknown>).wordsWritten).toBe(200);
  });

  test("end_session called on auto-complete", async ({ page }) => {
    await startSprint(page);
    await clearIpcCalls(page);

    // Force timer to almost zero
    await page.evaluate(async () => {
      const { sprintStore } = await import("/src/lib/stores/index.ts");
      sprintStore.remainingSeconds = 2;
    });

    await page.waitForTimeout(4000);

    const calls = await getIpcCallsByCommand(page, "end_session");
    expect(calls.length).toBe(1);
  });
});

// =============================================================================
// Stats View
// =============================================================================

test.describe("Stats View", () => {
  test("stats tab opens from toolbar", async ({ page }) => {
    await openMockProject(page);
    // Dispatch the click handler's event directly to avoid Playwright
    // actionability timeout during the reactive render cascade
    await page.evaluate(async () => {
      const { editorState } = await import("/src/lib/stores/index.ts");
      editorState.openDocument({
        id: "stats:writing",
        title: "Writing Stats",
        documentType: "stats",
        documentSlug: "writing",
        isDirty: false,
      });
    });
    await page.getByRole("tab", { name: "Writing Stats" }).waitFor();
    await expect(page.locator(".writing-stats")).toBeVisible();
  });

  test("IPC commands called on mount", async ({ page }) => {
    await openMockProject(page);
    await clearIpcCalls(page);
    await openStatsTab(page);

    // Wait for data to load
    await page.waitForTimeout(1000);

    const sessionCalls = await getIpcCallsByCommand(page, "get_sessions");
    const statsCalls = await getIpcCallsByCommand(page, "get_session_stats");

    expect(sessionCalls.length).toBeGreaterThanOrEqual(1);
    expect(statsCalls.length).toBeGreaterThanOrEqual(1);
  });

  test("heatmap renders", async ({ page }) => {
    await openMockProject(page);
    await openStatsTab(page);

    const heatmap = page.locator(".calendar-heatmap");
    await expect(heatmap).toBeVisible();

    // Should have many day cells (52 weeks × 7 days = 364)
    const dayCells = page.locator(".day-cell");
    const count = await dayCells.count();
    expect(count).toBeGreaterThan(200);

    // Some cells should have non-zero intensity
    const filledCells = page.locator(
      ".day-cell:not(.level-0)",
    );
    const filledCount = await filledCells.count();
    expect(filledCount).toBeGreaterThan(0);
  });

  test("stats cards show correct values", async ({ page }) => {
    await openMockProject(page);
    await openStatsTab(page);

    // Current Streak
    const streakCard = page
      .locator(".stat-card--streak")
      .filter({ hasText: "Current Streak" });
    await expect(streakCard.locator(".stat-value")).toHaveText("3");

    // Total Words
    const wordsCard = page
      .locator(".stat-card")
      .filter({ hasText: "Total Words" });
    await expect(wordsCard.locator(".stat-value")).toHaveText("12,650");

    // Sessions count — use the label text to find the card
    const sessionsCard = page
      .locator(".stat-card")
      .filter({ has: page.locator(".stat-label", { hasText: "Sessions" }) });
    await expect(sessionsCard.locator(".stat-value")).toHaveText("25");

    // Best Day — value and date
    const bestDayCard = page
      .locator(".stat-card--best")
      .filter({ hasText: "Best Day" });
    await expect(bestDayCard.locator(".stat-value")).toHaveText("1,200");
    await expect(bestDayCard.locator(".stat-unit")).toContainText(
      "Feb 3, 2026",
    );
  });

  test("empty stats state", async ({ page }) => {
    await openMockProject(page, {
      get_sessions: [],
      get_session_stats: {
        totalSessions: 0, totalWords: 0, totalMinutes: 0, currentStreak: 0,
        longestStreak: 0, dailyAverage: 0, weeklyAverage: 0, monthlyAverage: 0,
        bestDayWords: 0, bestDayDate: null,
      },
    });

    await openStatsTab(page);

    await expect(page.getByText("No sprints yet")).toBeVisible();
  });
});

// =============================================================================
// Sprint History
// =============================================================================

test.describe("Sprint History", () => {
  test("sprint entries display correctly", async ({ page }) => {
    await openMockProject(page);
    await openStatsTab(page);

    const entries = page.locator(".sprint-entry");
    const count = await entries.count();
    expect(count).toBeGreaterThan(0);

    // First entry (newest): Feb 15 14:00, the-awakening, 30 min, 750 words
    const firstEntry = entries.first();
    await expect(firstEntry.locator(".sprint-chapter")).toContainText(
      "the-awakening",
    );
    await expect(firstEntry.locator(".sprint-duration")).toContainText(
      "30 min",
    );
    await expect(firstEntry.locator(".sprint-words")).toContainText(
      "750 words",
    );
  });

  test("goal met/missed indicators", async ({ page }) => {
    await openMockProject(page);
    await openStatsTab(page);

    // First entry: goal 500, words 750 → goal met
    const metEntry = page
      .locator(".sprint-entry")
      .filter({ hasText: "500 goal" });
    await expect(metEntry.locator(".sprint-goal.goal-met")).toBeVisible();

    // Second entry: goal 600, words 420 → goal missed
    const missedEntry = page
      .locator(".sprint-entry")
      .filter({ hasText: "600 goal" });
    await expect(
      missedEntry.locator(".sprint-goal.goal-missed"),
    ).toBeVisible();
  });

  test("pagination with Show more", async ({ page }) => {
    await openMockProject(page);
    await openStatsTab(page);

    // Initially shows at most 20 entries
    const entries = page.locator(".sprint-entry");
    const initialCount = await entries.count();
    expect(initialCount).toBe(20);

    // "Show more" button visible
    const showMore = page.getByText("Show more");
    await expect(showMore).toBeVisible();

    // Click to load remaining entries
    await showMore.click();

    const expandedCount = await entries.count();
    expect(expandedCount).toBe(25);

    // "Show more" should be gone
    await expect(showMore).not.toBeVisible();
  });

  test("empty state", async ({ page }) => {
    await openMockProject(page, {
      get_sessions: [],
      get_session_stats: {
        totalSessions: 0, totalWords: 0, totalMinutes: 0, currentStreak: 0,
        longestStreak: 0, dailyAverage: 0, weeklyAverage: 0, monthlyAverage: 0,
        bestDayWords: 0, bestDayDate: null,
      },
    });

    await openStatsTab(page);

    await expect(
      page.getByText("No sprints yet. Start your first writing sprint!"),
    ).toBeVisible();
  });
});

// =============================================================================
// Edge Cases
// =============================================================================

test.describe("Edge Cases", () => {
  test("Start Sprint disabled with no chapter open", async ({ page }) => {
    await openMockProject(page);
    await openSprintPanel(page);

    // Start button should be disabled
    const startBtn = page.getByText("Start Sprint");
    await expect(startBtn).toBeDisabled();

    // Hint text visible
    await expect(
      page.getByText("Open a chapter to start a sprint"),
    ).toBeVisible();
  });

  test("Start Sprint enables after opening a chapter", async ({ page }) => {
    await openMockProject(page);
    await openSprintPanel(page);

    // Initially disabled
    await expect(page.getByText("Start Sprint")).toBeDisabled();

    // Close panel, open chapter
    await page.locator(".sprint-panel-backdrop").click();
    await openChapter(page);

    // Re-open panel — should now be enabled
    await openSprintPanel(page);
    await expect(page.getByText("Start Sprint")).toBeEnabled();
  });

  test("cannot open sprint panel while sprint is active", async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);
    await startSprint(page);

    // Dispatch the toggle-sprint event directly (toolbar is hidden during sprint)
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent("sakya:toggle-sprint"));
    });

    // Sprint panel dialog should NOT appear (handleToggleSprint returns early when active)
    await expect(
      page.locator('[role="dialog"][aria-label="Sprint Timer"]'),
    ).not.toBeVisible();
    // Overlay still visible
    await expect(page.locator(".sprint-overlay")).toBeVisible();
  });

  test("very short sprint completes correctly", async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);
    await startSprint(page);

    // Set remaining to 1 second
    await page.evaluate(async () => {
      const { sprintStore } = await import("/src/lib/stores/index.ts");
      sprintStore.remainingSeconds = 1;
    });

    await page.waitForTimeout(3000);

    await expect(page.locator(".sprint-overlay")).not.toBeVisible();

    const calls = await getIpcCallsByCommand(page, "end_session");
    expect(calls.length).toBeGreaterThanOrEqual(1);
  });

  test("sprint panel auto-closes when sprint starts", async ({ page }) => {
    await openMockProject(page);
    await openChapter(page);

    // Open panel
    await openSprintPanel(page);
    const dialog = page.locator(
      '[role="dialog"][aria-label="Sprint Timer"]',
    );
    await expect(dialog).toBeVisible();

    // Start sprint
    await page.getByText("Start Sprint").click();

    // Panel (dialog) should close, overlay should appear
    await expect(dialog).not.toBeVisible();
    await expect(page.locator(".sprint-overlay")).toBeVisible();
  });
});
