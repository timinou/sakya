import type { Page } from "@playwright/test";

type MockHandler = (args?: Record<string, unknown>) => unknown;
type MockValue = unknown | MockHandler;

/**
 * Set up Tauri IPC mocks by injecting a fake __TAURI_INTERNALS__ object.
 * This intercepts invoke() calls in the browser context.
 *
 * IMPORTANT: Mock functions are serialized via .toString() and eval'd in the
 * browser. They MUST be self-contained — no references to external variables
 * or imported constants. Use plain values (not functions) where possible.
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
    // Use window-level array so clearIpcCalls can replace it and invoke sees the new one
    (window as any).__TAURI_IPC_CALLS__ = [];

    (window as any).__TAURI_INTERNALS__ = {
      transformCallback(callback: Function) {
        const id = window.crypto.randomUUID();
        (window as any)[`_${id}`] = callback;
        return id;
      },
      convertFileSrc(filePath: string) {
        return `asset://localhost/${filePath}`;
      },
      invoke(
        cmd: string,
        args?: Record<string, unknown>,
        _options?: unknown,
      ) {
        // Always reference window-level array (not a closure) so clearIpcCalls works
        ((window as any).__TAURI_IPC_CALLS__ as Array<{ cmd: string; args?: unknown }>).push({ cmd, args });
        const mock = mocks[cmd];
        if (!mock) {
          console.warn(
            `[tauri-mock] No mock registered for command: ${cmd}`,
          );
          return Promise.reject(
            new Error(`No mock registered for command: ${cmd}`),
          );
        }
        if (mock.type === "function") {
          // eslint-disable-next-line no-eval
          const fn = eval(`(${mock.body})`);
          try {
            return Promise.resolve(fn(args));
          } catch (e) {
            return Promise.reject(e);
          }
        }
        return Promise.resolve(mock.data);
      },
    };
  }, serializedMocks);
}

// ---------------------------------------------------------------------------
// Mock data factories
// ---------------------------------------------------------------------------

/** Schema summaries returned by list_schemas */
export const MOCK_SCHEMA_SUMMARIES = [
  {
    name: "Character",
    entityType: "character",
    fieldCount: 3,
    axisCount: 4,
  },
  {
    name: "Place",
    entityType: "place",
    fieldCount: 2,
    axisCount: 0,
  },
];

/** Full schema returned by get_schema */
export const MOCK_SCHEMAS: Record<string, unknown> = {
  character: {
    name: "Character",
    entityType: "character",
    icon: "users",
    color: "#7c4dbd",
    description: "A character in the story",
    fields: [
      {
        name: "role",
        label: "Role",
        fieldType: "short_text",
        placeholder: "Protagonist, Antagonist...",
        required: true,
      },
      {
        name: "age",
        label: "Age",
        fieldType: "number",
        min: 0,
        max: 999,
      },
      {
        name: "backstory",
        label: "Backstory",
        fieldType: "long_text",
        placeholder: "Character background...",
      },
    ],
    spiderAxes: [
      { name: "Strength", min: 0, max: 10, default: 5 },
      { name: "Intelligence", min: 0, max: 10, default: 5 },
      { name: "Charisma", min: 0, max: 10, default: 5 },
      { name: "Wisdom", min: 0, max: 10, default: 5 },
    ],
  },
  place: {
    name: "Place",
    entityType: "place",
    icon: "map-pin",
    color: "#2e8b57",
    description: "A location in the story",
    fields: [
      {
        name: "region",
        label: "Region",
        fieldType: "short_text",
        placeholder: "Northern Kingdom...",
      },
      {
        name: "climate",
        label: "Climate",
        fieldType: "select",
        options: ["tropical", "temperate", "arctic", "desert"],
      },
    ],
    spiderAxes: [],
  },
};

/** Entity summaries returned by list_entities */
export const MOCK_ENTITIES_BY_TYPE: Record<string, unknown[]> = {
  character: [
    {
      title: "Elena Blackwood",
      slug: "elena-blackwood",
      schemaType: "character",
      tags: ["protagonist", "mage"],
    },
    {
      title: "Marcus Thorne",
      slug: "marcus-thorne",
      schemaType: "character",
      tags: ["antagonist"],
    },
    {
      title: "Sage Willowmere",
      slug: "sage-willowmere",
      schemaType: "character",
      tags: ["mentor"],
    },
  ],
  place: [
    {
      title: "Ironhaven",
      slug: "ironhaven",
      schemaType: "place",
      tags: ["city", "capital"],
    },
    {
      title: "The Whispering Woods",
      slug: "the-whispering-woods",
      schemaType: "place",
      tags: ["forest", "enchanted"],
    },
  ],
};

/** Full entity instance returned by get_entity */
export const MOCK_ENTITY_INSTANCES: Record<string, unknown> = {
  "elena-blackwood": {
    title: "Elena Blackwood",
    slug: "elena-blackwood",
    schemaSlug: "character",
    tags: ["protagonist", "mage"],
    spiderValues: {
      Strength: 4,
      Intelligence: 9,
      Charisma: 7,
      Wisdom: 6,
    },
    fields: {
      role: "Protagonist",
      age: 28,
      backstory: "Born in the northern reaches...",
    },
    body: "# Elena Blackwood\n\nThe last surviving heir of the Blackwood lineage.",
  },
  "marcus-thorne": {
    title: "Marcus Thorne",
    slug: "marcus-thorne",
    schemaSlug: "character",
    tags: ["antagonist"],
    spiderValues: {
      Strength: 8,
      Intelligence: 7,
      Charisma: 5,
      Wisdom: 3,
    },
    fields: {
      role: "Antagonist",
      age: 45,
      backstory: "A fallen knight turned warlord...",
    },
    body: "# Marcus Thorne\n\nOnce a noble knight, now consumed by ambition.",
  },
};

/** Manuscript config returned by get_manuscript_config */
export const MOCK_MANUSCRIPT_CONFIG = {
  chapters: ["the-awakening", "into-the-woods", "the-siege"],
};

/** Chapter content returned by get_chapter */
export const MOCK_CHAPTERS: Record<string, unknown> = {
  "the-awakening": {
    slug: "the-awakening",
    frontmatter: {
      slug: "the-awakening",
      title: "The Awakening",
      status: "revised",
      pov: "Elena",
      synopsis: "Elena discovers her latent magical abilities.",
      targetWords: 3000,
      order: 0,
    },
    body: "The morning light filtered through the cracked window pane, casting long shadows across Elena's cluttered desk. She had been awake for hours.",
  },
  "into-the-woods": {
    slug: "into-the-woods",
    frontmatter: {
      slug: "into-the-woods",
      title: "Into the Woods",
      status: "draft",
      pov: "Elena",
      synopsis:
        "Elena ventures into the Whispering Woods seeking answers.",
      targetWords: 4000,
      order: 1,
    },
    body: "The trees stood like sentinels at the forest's edge, their branches intertwined overhead to form a canopy that blocked out the sun.",
  },
  "the-siege": {
    slug: "the-siege",
    frontmatter: {
      slug: "the-siege",
      title: "The Siege",
      status: "draft",
      pov: "Marcus",
      synopsis:
        "Marcus leads his forces against the northern fortifications.",
      targetWords: 5000,
      order: 2,
    },
    body: "The war drums echoed across the valley as Marcus surveyed the fortress walls from his command tent.",
  },
};

/** Notes config returned by get_notes_config */
export const MOCK_NOTES_CONFIG = {
  notes: [
    {
      slug: "magic-system",
      title: "Magic System Rules",
      color: "#3b82f6",
      label: "worldbuilding",
      position: { x: 20, y: 30 },
    },
    {
      slug: "plot-outline",
      title: "Plot Outline",
      color: "#22c55e",
      label: "planning",
      position: { x: 60, y: 25 },
    },
    {
      slug: "character-arcs",
      title: "Character Arcs",
      color: "#8b5cf6",
      label: null,
      position: { x: 40, y: 65 },
    },
  ],
};

/** Note content returned by get_note */
export const MOCK_NOTE_CONTENT: Record<string, unknown> = {
  "magic-system": {
    slug: "magic-system",
    title: "Magic System Rules",
    body: "# Magic System\n\n1. All magic has a cost\n2. Elemental affinities are hereditary\n3. Overuse leads to corruption",
  },
  "plot-outline": {
    slug: "plot-outline",
    title: "Plot Outline",
    body: "# Plot Outline\n\n## Act 1: Discovery\n- Elena discovers powers\n- Mentor appears\n\n## Act 2: Journey\n- Travel through Whispering Woods\n- Confrontation with Marcus",
  },
  "character-arcs": {
    slug: "character-arcs",
    title: "Character Arcs",
    body: "# Character Arcs\n\n**Elena**: Reluctant hero -> Confident leader\n**Marcus**: Honorable knight -> Fallen villain",
  },
};

/** Search results returned by search_project */
export const MOCK_SEARCH_RESULTS = [
  {
    title: "The Awakening",
    slug: "the-awakening",
    fileType: "chapter",
    matchingLine: "Elena's cluttered desk",
    lineNumber: 1,
    contextBefore: "",
    contextAfter: "",
  },
  {
    title: "Elena Blackwood",
    slug: "elena-blackwood",
    fileType: "entity",
    entityType: "character",
    matchingLine: "The last surviving heir of the Blackwood lineage",
    lineNumber: 3,
    contextBefore: "",
    contextAfter: "",
  },
  {
    title: "Magic System Rules",
    slug: "magic-system",
    fileType: "note",
    matchingLine: "Elemental affinities are hereditary",
    lineNumber: 4,
    contextBefore: "",
    contextAfter: "",
  },
];

// ---------------------------------------------------------------------------
// Default mock setup — includes ALL commands the app can call
// ---------------------------------------------------------------------------

/**
 * Set up default Tauri mocks for the current app commands.
 * Includes all entity, manuscript, notes, search, and FS plugin commands
 * so the app can fully initialize after opening a project.
 *
 * IMPORTANT: All function mocks must be SELF-CONTAINED — they are serialized
 * via .toString() and eval'd in the browser context. They cannot reference
 * any variables from this module (like MOCK_SCHEMA_SUMMARIES). Use plain
 * values for constant returns, and inline data into lookup functions.
 */
export async function setupDefaultTauriMocks(
  page: Page,
  overrides: Record<string, MockValue> = {},
): Promise<void> {
  // Use plain values (not functions) for constant returns.
  // Use self-contained functions (with inlined data) for args-based lookups.
  const defaults: Record<string, MockValue> = {
    // --- Project commands ---
    greet: ((args: Record<string, unknown> | undefined) =>
      `Hello, ${args?.name ?? ""}! You've been greeted from Rust!`) as MockHandler,
    create_project: ((args: Record<string, unknown> | undefined) => ({
      name: args?.name ?? "Test Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    })) as MockHandler,
    open_project: {
      name: "Opened Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    },
    save_project_manifest: null,

    // --- Dialog plugin ---
    "plugin:dialog|open": "/mock/project/path",

    // --- Entity commands (values for simple returns, self-contained functions for lookups) ---
    list_schemas: MOCK_SCHEMA_SUMMARIES,

    // Self-contained function — data is inlined via JSON
    get_schema: new Function(
      "args",
      `var schemas = ${JSON.stringify(MOCK_SCHEMAS)};
       var schemaType = args && args.schemaType;
       return schemas[schemaType] || null;`,
    ) as MockHandler,

    list_entities: new Function(
      "args",
      `var entities = ${JSON.stringify(MOCK_ENTITIES_BY_TYPE)};
       var schemaType = args && args.schemaType;
       return entities[schemaType] || [];`,
    ) as MockHandler,

    get_entity: new Function(
      "args",
      `var instances = ${JSON.stringify(MOCK_ENTITY_INSTANCES)};
       var slug = args && args.slug;
       return instances[slug] || null;`,
    ) as MockHandler,

    create_entity: ((args: Record<string, unknown> | undefined) => ({
      title: args?.title ?? "New Entity",
      slug:
        (args?.title as string)?.toLowerCase().replace(/\s+/g, "-") ??
        "new-entity",
      schemaSlug: args?.schemaType ?? "character",
      tags: [],
      spiderValues: {},
      fields: {},
      body: "",
    })) as MockHandler,
    save_entity: null,
    delete_entity: null,
    rename_entity: null,
    save_schema: null,
    delete_schema: null,

    // --- Manuscript commands ---
    get_manuscript_config: MOCK_MANUSCRIPT_CONFIG,

    get_chapter: new Function(
      "args",
      `var chapters = ${JSON.stringify(MOCK_CHAPTERS)};
       var slug = args && args.slug;
       return chapters[slug] || null;`,
    ) as MockHandler,

    save_chapter: null,

    create_chapter: ((args: Record<string, unknown> | undefined) => ({
      slug:
        (args?.title as string)?.toLowerCase().replace(/\s+/g, "-") ??
        "new-chapter",
      frontmatter: {
        slug:
          (args?.title as string)?.toLowerCase().replace(/\s+/g, "-") ??
          "new-chapter",
        title: args?.title ?? "New Chapter",
        status: "draft",
        pov: null,
        synopsis: null,
        targetWords: null,
        order: 3,
      },
      body: "",
    })) as MockHandler,
    delete_chapter: null,
    reorder_chapters: null,

    // --- Notes commands ---
    get_notes_config: MOCK_NOTES_CONFIG,

    get_note: new Function(
      "args",
      `var notes = ${JSON.stringify(MOCK_NOTE_CONTENT)};
       var slug = args && args.slug;
       return notes[slug] || null;`,
    ) as MockHandler,

    save_note: null,

    create_note: ((args: Record<string, unknown> | undefined) => ({
      slug:
        (args?.title as string)?.toLowerCase().replace(/\s+/g, "-") ??
        "new-note",
      title: args?.title ?? "New Note",
      body: "",
    })) as MockHandler,
    delete_note: null,
    save_notes_config: null,

    // --- Search commands ---
    search_project: MOCK_SEARCH_RESULTS,
    resolve_wiki_link: null,
    find_backlinks: [],

    // --- FS plugin commands ---
    "plugin:fs|read_text_file": "{}",
    "plugin:fs|write_text_file": null,
    "plugin:fs|mkdir": null,
  };
  await setupTauriMocks(page, { ...defaults, ...overrides });
}

/**
 * Set up mocks for an already-opened project.
 * Calls setupDefaultTauriMocks, navigates to the app, then clicks Open Project
 * to transition from launcher -> app shell with all stores initialized.
 */
export async function openMockProject(
  page: Page,
  overrides: Record<string, MockValue> = {},
): Promise<void> {
  await setupDefaultTauriMocks(page, overrides);
  await page.goto("/");
  await page.getByRole("button", { name: /open project/i }).click();
  // Wait for the app shell to render (Binder header appears)
  await page
    .getByText("Binder")
    .waitFor({ state: "visible", timeout: 10000 });
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

/**
 * Get IPC calls filtered by command name.
 */
export async function getIpcCallsByCommand(
  page: Page,
  command: string,
): Promise<Array<{ cmd: string; args?: unknown }>> {
  const calls = await getIpcCalls(page);
  return calls.filter((c) => c.cmd === command);
}

/**
 * Clear recorded IPC calls (useful between actions in a test).
 */
export async function clearIpcCalls(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as any).__TAURI_IPC_CALLS__ = [];
  });
}
