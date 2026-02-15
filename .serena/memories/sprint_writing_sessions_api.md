# Sprint & Writing Sessions Implementation Guide

## Overview
The Sakya writing app implements a complete sprint (timed writing session) system with backend persistence, real-time UI feedback, and comprehensive statistics.

---

## 1. SPRINT STORE (`src/lib/stores/sprint.svelte.ts`)

### State Variables
```typescript
isActive: boolean              // Sprint is running
isPaused: boolean              // Sprint is paused (not stopped)
durationMinutes: number        // Target duration (e.g., 25, 45, 60)
remainingSeconds: number       // Time left (decrements every second)
sessionId: string | null       // Backend session ID (ISO 8601 timestamp)
startWordCount: number         // Word count when sprint started
sprintGoal: number | undefined // Optional word count goal
chapterSlug: string | null     // Chapter being written
projectPath: string            // Project path for IPC calls
```

### Derived State
```typescript
elapsed: number = $derived      // 0 to 1, progress through sprint
                                 // = 1 - (remainingSeconds / (durationMinutes * 60))
```

### Public Methods
```typescript
selectDuration(minutes: number)
  - Sets durationMinutes (only if not active)

start(durationMinutes, chapterSlug, projectPath, currentWordCount, sprintGoal?)
  - Invokes 'start_session' on backend
  - Sets sessionId from response
  - Starts 1000ms interval countdown
  - Returns early if already active

pause()
  - Sets isPaused = true
  - Stops interval (no countdown)

resume()
  - Sets isPaused = false
  - Restarts interval

stop(currentWordCount, projectPath)
  - Invokes 'end_session' on backend
  - Calculates wordsWritten = max(0, currentWordCount - startWordCount)
  - Resets all state
```

### Callbacks
```typescript
getWordCount: (() => number) | null
  - Set by integrating component (AppShell)
  - Called when timer auto-completes

onComplete: (() => void) | null
  - Fired when countdown reaches zero
  - Used to trigger auto-save
```

---

## 2. SESSIONS STORE (`src/lib/stores/sessions.svelte.ts`)

### State Variables
```typescript
sessions: WritingSession[]     // All sessions for project
stats: SessionStats | null     // Aggregated stats
isLoading: boolean             // Loading state
private loadedPath: string | null  // Track which project is loaded
```

### Derived State
```typescript
dailyWordCounts: Map<"YYYY-MM-DD", number>
  - Groups sessions by date
  - Sums words per day
  - Used by CalendarHeatmap
```

### Public Methods
```typescript
loadSessions(projectPath: string)
  - Promise.all(['get_sessions', 'get_session_stats'])
  - Sets sessions[], stats, and loadedPath
  - isLoading = true/false around call

refresh()
  - Re-invokes loadSessions with cached loadedPath
  - Called after a sprint ends
```

---

## 3. TAURI COMMANDS (`src-tauri/src/commands/sessions.rs`)

### IPC Commands

#### `start_session(project_path, chapter_slug, sprint_goal?)`
- **Returns**: `String` (ISO 8601 timestamp, serves as session ID)
- **Creates**: `.sakya/sessions.yaml` if missing
- **Stores**:
  - `id`: ISO 8601 timestamp
  - `start`: same ISO timestamp
  - `chapter_slug`: string
  - `sprint_goal`: Option<u32>
  - All other fields initialized to None/0

#### `end_session(project_path, session_id, words_written)`
- **Returns**: `Result<(), AppError>`
- **Updates**: Session in `.sakya/sessions.yaml`
  - `end`: ISO 8601 timestamp
  - `duration_minutes`: calculated as (end - start) / 60
  - `words_written`: provided value

#### `get_sessions(project_path, from?, to?)`
- **Parameters**:
  - `from`: Optional ISO 8601 date string "YYYY-MM-DD"
  - `to`: Optional ISO 8601 date string "YYYY-MM-DD"
- **Returns**: `Vec<WritingSession>` (filtered)
- **Filter**: Inclusive on both boundaries
- **Sessions counted by**: `start` date (if session spans midnight, counted on start date)

#### `get_session_stats(project_path)`
- **Returns**: `SessionStats`
- **Calculations**:
  - `total_sessions`: count of all sessions
  - `total_words`: sum of words_written (u64, no overflow)
  - `total_minutes`: sum of duration_minutes
  - `current_streak`: consecutive days from today backwards (1-day grace: if no session today, check yesterday)
  - `longest_streak`: longest consecutive days ever
  - `daily_average`: total_words / days_span (span = today - first_session_date)
  - `weekly_average`: daily_average * 7
  - `monthly_average`: daily_average * 30
  - `best_day_words`: max word count on any single day (sums sessions per day)
  - `best_day_date`: ISO 8601 date of best day

---

## 4. SPRINT UI COMPONENTS

### SprintOverlay.svelte (`.sprint-overlay`)
**Rendered when**: `sprintStore.isActive === true`

#### Structure
```
.sprint-overlay (z-index: 500, pointer-events: none)
├─ .vignette.vignette-top      (gradient fade, 60px)
├─ .vignette.vignette-bottom   (gradient fade, 60px)
├─ .vignette.vignette-left     (gradient fade, 40px)
├─ .vignette.vignette-right    (gradient fade, 40px)
├─ .sprint-bar (z-index: 501, pointer-events: auto, centered at top)
│  ├─ .sprint-bar-left
│  │  └─ .sprint-label "Sprint"
│  ├─ .sprint-bar-center
│  │  ├─ .countdown "MM:SS"
│  │  └─ .pause-indicator "Paused" (if isPaused)
│  └─ .sprint-bar-right
│     ├─ button.bar-btn "Resume|Pause" (aria-label)
│     ├─ button.bar-btn.bar-btn-stop "Stop" (aria-label)
│     └─ button.bar-btn.bar-btn-save "Save" (aria-label)
├─ .elapsed-track (2px progress bar)
│  └─ .elapsed-fill (width: elapsed * 100%)
└─ .goal-bar (optional, if sprintGoal)
   ├─ .goal-track
   │  └─ .goal-fill (width: goalProgress * 100%, color changes if goal-met)
   └─ .goal-text "{wordsWritten} / {sprintGoal} words"
```

#### CSS Classes
```
.sprint-overlay         - Fixed full-screen, z-index 500
.sprint-overlay.paused  - Applied when isPaused === true
.sprint-bar             - min-width: 360px, centered, elevated
.countdown              - Monospace, semibold
.countdown.paused       - opacity: 0.5
.pause-indicator        - "Paused" label
.bar-btn                - Secondary style
.bar-btn-stop           - Error red color
.bar-btn-save           - Accent primary color
.elapsed-track          - Thin border-secondary background
.elapsed-fill           - Accent primary, smooth transition
.goal-bar               - Only visible if sprintGoal is set
.goal-fill.goal-met     - color: success green
```

#### Props
```typescript
chapterSlug?: string | null
projectPath?: string
currentWordCount?: number
onSprintEnd?: () => void
```

#### Custom Events
- Dispatches `sakya:save` on Save button (expects window listener)

#### Aria Labels
- "Resume sprint" / "Pause sprint" (button title + aria-label)
- "Stop sprint" (button title + aria-label)
- "Save document" (button title + aria-label)

---

### SprintTimer.svelte
**Rendering modes**: Inactive (duration selector) or Active (progress ring)

#### Duration Selection (when `!sprintStore.isActive`)
```
.sprint-timer
├─ .duration-selector
│  ├─ .selector-label "Sprint duration"
│  └─ .preset-buttons
│     ├─ button.preset-btn "15m" (class:selected if selected)
│     ├─ button.preset-btn "25m"
│     ├─ button.preset-btn "30m"
│     ├─ button.preset-btn "45m"
│     └─ button.preset-btn "60m"
├─ button.start-btn "Start Sprint" (disabled if !chapterSlug)
└─ .hint "Open a chapter to start a sprint" (if !chapterSlug)
```

#### Active Sprint (when `sprintStore.isActive`)
```
.sprint-timer
├─ .ring-container
│  ├─ svg.progress-ring (viewBox="0 0 200 200", 200x200 circle)
│  │  ├─ circle.ring-bg (r=90, stroke-width=6, gray)
│  │  └─ circle.ring-progress (r=90, animated stroke-dashoffset)
│  └─ .countdown
│     ├─ span.time "MM:SS" (class:paused if isPaused)
│     └─ span.pause-label "Paused" (if isPaused)
└─ .controls
   ├─ button.control-btn.pause-btn "Resume|Pause"
   └─ button.control-btn.stop-btn "Stop"
```

#### SVG Ring Parameters
```typescript
RING_RADIUS = 90
RING_CIRCUMFERENCE = 2 * Math.PI * 90 = ~565.5
ringOffset = RING_CIRCUMFERENCE * (1 - sprintStore.elapsed)
  - Animates from full (at start) to 0 (at end)
  - Transform: rotate(-90 100 100) to start at top
```

#### Props
```typescript
chapterSlug?: string | null
projectPath?: string
currentWordCount?: number
onSprintEnd?: () => void
```

---

## 5. WRITING STATS COMPONENT (`WritingStats.svelte`)

### Loading State
```
.writing-stats
└─ .stats-loading
   ├─ .loading-spinner (animation: spin 0.8s linear infinite)
   └─ "Loading statistics..."
```

### Rendered Stats (when `!sessionsStore.isLoading`)
```
.writing-stats
├─ .stats-section (Calendar Heatmap)
│  ├─ h2.section-title "Writing Activity"
│  └─ CalendarHeatmap {dailyWordCounts}
├─ .stats-section (Overview)
│  ├─ h2.section-title "Overview"
│  └─ .stats-grid
│     ├─ .stat-card.stat-card--streak (Flame icon, red)
│     │  ├─ .stat-icon
│     │  ├─ .stat-value {currentStreak}
│     │  ├─ .stat-label "Current Streak"
│     │  └─ .stat-unit "day(s)"
│     ├─ .stat-card (Award icon) - longestStreak
│     ├─ .stat-card (PenTool icon) - totalWords
│     ├─ .stat-card (Clock icon) - totalMinutes (formatted)
│     ├─ .stat-card (Calendar icon) - totalSessions
│     ├─ .stat-card (TrendingUp icon) - dailyAverage
│     ├─ .stat-card (Star icon) - weeklyAverage
│     └─ .stat-card.stat-card--best (Trophy icon, accent) - bestDayWords
└─ .stats-section (Sprint History)
   ├─ h2.section-title "Sprint History"
   └─ SprintHistory {sessions}
```

### Data Loading
```typescript
$effect(() => {
  const path = projectState.projectPath;
  if (!path) return;
  sessionsStore.loadSessions(path);
});
```

---

## 6. SPRINT HISTORY COMPONENT (`SprintHistory.svelte`)

### Empty State
```
.sprint-history
└─ p.empty-state "No sprints yet. Start your first writing sprint!"
```

### Session List (max 20 per page)
```
.sprint-history
├─ .sprint-list
│  └─ li.sprint-entry (repeated)
│     ├─ .sprint-header
│     │  ├─ .sprint-date "Feb 14, 10:30 AM"
│     │  └─ .sprint-chapter "chapter-slug" (right-aligned, truncated)
│     └─ .sprint-details
│        ├─ .sprint-duration "25 min" or "—"
│        ├─ .sprint-words "847 words"
│        └─ .sprint-goal (optional if sprintGoal is set)
│           ├─ svg.goal-icon (✓ if met, ✗ if missed)
│           └─ .goal-target "500 goal"
└─ button.show-more-btn "Show more" (if hasMore)
```

#### Goal Status Classes
```
.sprint-goal.goal-met    - color: success green
.sprint-goal.goal-missed - color: error red
```

---

## 7. TYPE DEFINITIONS

### WritingSession
```typescript
interface WritingSession {
  id: string;                   // ISO 8601 timestamp (also start time)
  start: string;                // ISO 8601
  end?: string;                 // ISO 8601, optional until session ends
  durationMinutes?: number;     // Calculated on end_session
  wordsWritten: number;         // 0 until end_session
  chapterSlug: string;          // Chapter being worked on
  sprintGoal?: number;          // Optional word count goal
}
```

### SessionStats
```typescript
interface SessionStats {
  totalSessions: number;
  totalWords: number;
  totalMinutes: number;
  currentStreak: number;        // Days from today backwards (1-day grace)
  longestStreak: number;        // Longest consecutive days ever
  dailyAverage: number;         // total_words / days_since_first_session
  weeklyAverage: number;        // dailyAverage * 7
  monthlyAverage: number;       // dailyAverage * 30
  bestDayWords: number;         // Max words on any single day
  bestDayDate?: string;         // ISO 8601 date of best day
}
```

---

## 8. E2E TEST PATTERNS (from `distraction-free.spec.ts`)

### Accessing Stores via page.evaluate()
```typescript
const state = await page.evaluate(async () => {
  const { sprintStore, uiState } = await import("/src/lib/stores/index.ts");
  return {
    sprint: sprintStore.isActive,
    focus: uiState.focusMode,
  };
});
```

### Mock IPC Calls
```typescript
// In openMockProject, override IPC responses:
await openMockProject(page, {
  "plugin:fs|read_text_file": bytes,  // Array<u8> for file content
  "get_sessions": [...]                // JSON response
});

// Check calls made:
const calls = await getIpcCallsByCommand(page, "start_session");
expect(calls.length).toBeGreaterThan(0);
```

### Waiting for Async Operations
```typescript
// After toggling sprint or modes, wait for debounced persist
await page.waitForTimeout(1500);  // ~1s debounce + buffer

// Or wait for specific element
await expect(page.locator(".sprint-overlay")).toBeVisible({ timeout: 3000 });
```

---

## 9. KEY SELECTORS FOR E2E TESTS

### Sprint Overlay
```
.sprint-overlay               - Main container
.sprint-bar                   - Top bar with controls
.countdown                    - Timer display
.pause-indicator              - "Paused" label
button.bar-btn                - Pause/Resume button
button.bar-btn-stop           - Stop button
button.bar-btn-save           - Save button
.elapsed-track                - Progress bar container
.elapsed-fill                 - Progress bar fill
.goal-bar                     - Goal section (optional)
.goal-fill                    - Goal progress
.goal-fill.goal-met           - Goal reached state
```

### Sprint Timer (Inspector Panel)
```
.sprint-timer                 - Container
.duration-selector            - Duration picker
button.preset-btn             - Duration buttons (15m, 25m, etc.)
button.preset-btn.selected    - Active duration
button.start-btn              - Start Sprint button
.hint                         - "Open a chapter" hint
.ring-container               - SVG ring container
svg.progress-ring             - SVG element
.countdown                    - Time display
button.control-btn.pause-btn  - Pause/Resume
button.control-btn.stop-btn   - Stop
```

### Writing Stats
```
.writing-stats                - Container
.stats-loading                - Loading state
.loading-spinner              - Spinner animation
.stats-section                - Sections (calendar, overview, history)
.section-title                - "Writing Activity", "Overview", "Sprint History"
.stats-grid                   - Card grid
.stat-card                    - Individual stat card
.stat-card--streak            - Streak card (red flame icon)
.stat-card--best              - Best day card (accent trophy)
.stat-value                   - Large number
.stat-label                   - Label text
.stat-unit                    - Unit text
```

### Sprint History
```
.sprint-history               - Container
.empty-state                  - "No sprints yet" message
.sprint-list                  - ul container
.sprint-entry                 - li item
.sprint-date                  - ISO date/time
.sprint-chapter               - Chapter slug
.sprint-duration              - "25 min" or "—"
.sprint-words                 - "847 words"
.sprint-goal                  - Goal section (optional)
.sprint-goal.goal-met         - Checkmark (success green)
.sprint-goal.goal-missed      - X mark (error red)
.goal-target                  - "500 goal" text
button.show-more-btn          - Load more sessions
```

---

## 10. ARIA LABELS & ACCESSIBILITY

### Sprint Overlay Buttons
```html
<button aria-label="Resume sprint">Resume</button>
<button aria-label="Pause sprint">Pause</button>
<button aria-label="Stop sprint">Stop</button>
<button aria-label="Save document">Save</button>
```

### Sprint History
```html
<!-- SVG icons have aria-hidden="true" -->
<svg aria-hidden="true">...</svg>
```

---

## 11. KNOWN PATTERNS & EDGE CASES

### Auto-complete Grace Period
- Countdown reaches 0 → `onTimerComplete()` fires
- Calls `end_session` with **current word count** (via `getWordCount` callback)
- Resets sprint state
- Calls `onComplete` callback for auto-save

### Goal Progress Calculation
```typescript
wordsWritten = max(0, currentWordCount - startWordCount)
goalProgress = sprintGoal ? min(1, wordsWritten / sprintGoal) : 0
goalMet = sprintGoal ? wordsWritten >= sprintGoal : false
```

### Session Streak Logic
- **Current Streak**: Count backwards from today (1-day grace: if no session today, check yesterday)
- **Longest Streak**: Maximum consecutive days ever recorded
- **Multiple sessions same day**: Count as single streak day

### Daily Word Count Aggregation
- Sessions grouped by **start date** (not end date)
- Multiple sessions on same day are summed
- Used for calendar heatmap and "Best Day" calculation

---

## 12. INTEGRATION POINTS

### AppShell Integration
```typescript
// Set word count callback so timer can get current value
sprintStore.getWordCount = () => editorState.currentWordCount;

// Set completion callback for auto-save
sprintStore.onComplete = () => autoSave();
```

### Project State
```typescript
// Load sessions when project path changes
$effect(() => {
  const path = projectState.projectPath;
  if (path) sessionsStore.loadSessions(path);
});
```

---

## 13. TESTING CHECKLIST FOR E2E

- [ ] Sprint start/stop lifecycle
- [ ] Pause/resume mechanics
- [ ] Goal progress tracking
- [ ] Word count delta calculation
- [ ] Session persistence (start_session, end_session IPC)
- [ ] Stats calculation (streaks, averages, best day)
- [ ] UI updates (elapsed bar, goal bar, countdown)
- [ ] Empty state messaging
- [ ] Pagination ("Show more")
- [ ] Date filtering for sessions
- [ ] Multiple sessions same day aggregation
- [ ] Session spanning midnight edge case
