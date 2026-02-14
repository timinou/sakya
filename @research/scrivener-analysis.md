# Scrivener Analysis: Lessons for Sakya

## Overview

Scrivener (Literature & Latte, first released 2007) is the de facto standard writing
application for novelists, screenwriters, and academics. It has dominated the "serious
writer's tool" category for nearly two decades with minimal competition. Understanding
what Scrivener gets right, what it gets wrong, and where the gap lies is essential to
building Sakya as a credible alternative.

This document is organized around Scrivener's core UI, its integration model, its
project format, what writers love and hate about it, how alternatives compare, and
what all of this means for Sakya.

---

## Core UI Components

### The Binder (Hierarchical Tree)

The Binder is the left-hand panel, a hierarchical tree of documents, folders, and
media files that represents the entire project structure. It is the organizational
backbone of Scrivener.

Key characteristics:

- **Unlimited nesting depth**: Folders can contain folders, documents can contain
  documents. There is no enforced distinction between "folder" and "document" except
  the icon. A folder can have its own text content; a document can have children.
- **Drag-and-drop reordering**: Writers can restructure their manuscript by dragging
  items in the binder. This is one of Scrivener's most beloved features, enabling
  scene and chapter reordering without copy-paste.
- **Color-coded labels**: Each item can have a label (e.g., "First Draft," "Revised,"
  "Final") with an associated color. The binder can be tinted to show these colors,
  giving a visual overview of manuscript progress.
- **Status stamps**: Separate from labels, each item can have a status (e.g.,
  "To Do," "In Progress," "Done"). These appear as watermarks on the corkboard.
- **Icons**: Custom icons can be assigned to items for quick visual identification
  (character documents, setting documents, etc.).
- **Multiple root-level folders**: By default, Scrivener creates three root folders:
  Draft (the manuscript), Research (reference material), and Trash. Users can add
  more root-level folders for characters, settings, worldbuilding, etc.

The Binder is always visible (unless explicitly hidden) and serves as the primary
navigation mechanism. Clicking an item in the binder opens it in the editor.

### The Editor (Writing Surface)

The Editor occupies the center of the screen and is where actual writing happens.

Key characteristics:

- **Rich text editing**: Scrivener uses an RTF-based editor, not plain text. Writers
  can apply formatting (bold, italic, headings, etc.) directly. The compile step
  later converts this to the final output format.
- **Split-screen mode**: The editor can be split vertically or horizontally into two
  panes. Each pane can show a different document (or the same document at different
  positions). This enables side-by-side reference while writing.
- **Composition mode (full-screen)**: A distraction-free mode that blacks out
  everything except the writing surface. Customizable background color, text width,
  and opacity of the background.
- **Scriptwriting mode**: A specialized mode for screenplays with auto-formatting
  for scene headings, action, dialogue, parentheticals, etc.
- **Inline annotations and footnotes**: Writers can add inline notes (highlighted in
  a different color) and footnotes directly in the text. These can be stripped or
  converted during compile.
- **Snapshots**: Before making major edits, writers can take a "snapshot" of a
  document's current state. Snapshots can be compared side-by-side with the current
  version. This is Scrivener's version control.
- **Typewriter scrolling**: An option to keep the current line vertically centered
  on screen as the writer types, mimicking the behavior of a typewriter carriage.

### The Inspector (Metadata, Notes, Synopsis)

The Inspector is the right-hand panel, showing metadata and notes for the currently
selected document.

Key characteristics:

- **Synopsis**: A short summary of the document (what appears on the corkboard index
  card). Can be plain text or an image.
- **General metadata**: Label, status, creation date, modification date, word count,
  character count, target word count.
- **Custom metadata**: Users can define custom metadata fields (text, checkbox, date,
  list) that apply to all documents. These are visible in the outliner as columns.
- **Document notes**: A free-form notes area associated with the document. Separate
  from the document's actual content.
- **Project notes**: Notes that apply to the entire project, accessible from any
  document.
- **References**: Internal links to other documents in the binder, or external links
  to URLs and files.
- **Keywords**: Tags that can be assigned to documents and used for filtering.
- **Snapshots list**: A list of all snapshots taken for the current document.
- **Comments and footnotes list**: An aggregated view of all inline annotations and
  footnotes in the current document.

### The Corkboard (Visual Index Cards)

The Corkboard is an alternative view of the editor area. When a folder (or any item
with children) is selected, the editor can display its children as index cards on a
virtual corkboard.

Key characteristics:

- **Index cards**: Each child document appears as an index card showing its title and
  synopsis. Cards can be rearranged by dragging, which reorders the documents in
  the binder.
- **Color-coded pins**: Each card has a colored pin or tinted background matching its
  label color.
- **Status stamps**: Status watermarks appear on the cards.
- **Freeform mode**: Cards can be placed anywhere on the corkboard (not just in a
  grid), enabling mind-map-like spatial organization.
- **Stacking**: Cards can be stacked to represent groupings without creating folders.
- **Card size**: Adjustable card size to show more or fewer cards at once.
- **Commit arrangement**: In freeform mode, the spatial arrangement can be "committed"
  to reorder the documents in the binder.

The corkboard is one of Scrivener's most distinctive features and is directly inspired
by the physical index-card-on-corkboard technique used by screenwriters and novelists
for decades.

### The Outliner (Spreadsheet View)

The Outliner is another alternative view of the editor area. It shows children of the
selected item as rows in a spreadsheet-like table.

Key characteristics:

- **Columns**: Title, synopsis, label, status, word count, target word count, and all
  custom metadata fields. Columns can be shown, hidden, and reordered.
- **Expandable rows**: Rows can be expanded to show their children, enabling a full
  hierarchical view of the manuscript.
- **Sorting**: Columns can be sorted (though this is display-only; it does not reorder
  the actual documents).
- **Inline editing**: Synopsis, label, status, and custom metadata can be edited
  directly in the outliner cells.
- **Word count totals**: The outliner shows cumulative word counts for folders,
  enabling writers to track chapter and act lengths.

---

## Integration Model

Scrivener's most important architectural principle is that the binder, editor,
corkboard, and outliner are all views of the same underlying data. Any change in
one view is immediately reflected in all others.

- Reordering cards on the corkboard reorders items in the binder.
- Editing a synopsis in the inspector updates the index card on the corkboard.
- Changing a label color in the outliner updates the pin color on the corkboard
  and the tint in the binder.
- Renaming a document in the binder updates the title in the outliner and on the
  corkboard card.

This tight integration is what makes Scrivener feel like a unified environment rather
than a collection of separate tools. It is also one of the hardest things to replicate
in a new application. The key insight is that there is a single source of truth (the
binder hierarchy + document metadata), and every view is a projection of that truth.

### Compile: The Output Pipeline

Scrivener's "Compile" feature is its output system. It takes the contents of the Draft
folder and produces a single output document (PDF, DOCX, EPUB, etc.).

Key characteristics:

- **Format definitions**: Compile formats define how each structural level (folder,
  document group, document) is formatted in the output. Writers can assign different
  formatting to different levels (e.g., Part titles in large caps, chapter titles in
  bold, scene separators as "###").
- **Section layouts**: Each binder item can be assigned a "section type" (Chapter,
  Scene, Prologue, etc.), and each section type is mapped to a "section layout"
  that defines its formatting.
- **Transformations**: During compile, Scrivener can convert inline annotations to
  footnotes, strip comments, convert formatting, add front/back matter, etc.
- **Presets**: Scrivener ships with compile presets for common formats (Standard
  Manuscript, Paperback, EPUB, etc.), and users can create custom presets.

The compile system is extremely powerful but also extremely confusing. It is the
single most complained-about feature in Scrivener, and the primary source of the
"steep learning curve" reputation.

---

## Project Format

Scrivener uses a proprietary project format: the `.scriv` bundle.

### Structure of a .scriv Project

On macOS, a `.scriv` file is actually a directory (a "package" in macOS terminology).
On Windows, it uses a similar directory structure.

```
MyNovel.scriv/
  Files/
    Data/
      <UUID>/
        content.rtf      # The document content
        notes.rtf         # Document notes
        synopsis.txt      # The synopsis text
      <UUID>/
        content.rtf
        ...
    search.indexes        # Search index data
    binder.scrivx         # XML file defining the binder structure
    version.txt           # Scrivener version that last saved
  Settings/
    compile.xml           # Compile settings
    favorites.xml         # Favorite keywords/labels
    ui.xml                # UI state (window position, etc.)
  Snapshots/
    <UUID>/
      <timestamp>.rtf     # Snapshot content
  QuickLook/
    Preview.html          # Quick Look preview
    Thumbnail.jpg         # Quick Look thumbnail
```

### Key Observations

- **Content is RTF**: Document content is stored as RTF (Rich Text Format) files.
  This is a decades-old format that is not human-friendly to read or edit outside
  Scrivener.
- **Structure is XML**: The binder structure and metadata are stored in XML files.
  While technically human-readable, they are not designed for external editing.
- **UUIDs for filenames**: Documents are identified by UUIDs, not human-readable
  names. You cannot browse your manuscript by reading filenames.
- **Not version-control friendly**: RTF files produce noisy diffs in Git. The UUID
  filenames make it impossible to tell which file corresponds to which document
  without parsing the XML.
- **Sync conflicts**: Because the format uses many small files with opaque names,
  cloud sync (Dropbox, iCloud) can produce conflicts that are nearly impossible
  to resolve manually.

---

## What Writers Love About Scrivener

### Everything in One Place

Writers consistently cite "having everything in one place" as Scrivener's primary
value. The ability to keep research documents, character sheets, setting notes, plot
outlines, and the manuscript itself all within a single project eliminates the need
to manage dozens of separate files and folders.

This is especially valuable for novelists working on complex stories with many
characters, subplots, and worldbuilding elements. Before Scrivener, writers typically
used a combination of Word documents, physical notebooks, spreadsheets, and sticky
notes. Scrivener unifies all of these.

### Visual Planning with the Corkboard

The corkboard resonates strongly with writers who think visually. Being able to see
the structure of a novel as a set of index cards, rearrange them, color-code them,
and annotate them is a powerful planning tool.

Many writers report that they use the corkboard exclusively during the planning and
revision phases, switching to the editor for drafting.

### Character Continuity Management

For complex stories, maintaining character continuity is a major challenge. Scrivener's
ability to keep character sheets alongside the manuscript, link to them from scenes,
and search across the entire project helps writers track details like:

- What color are a character's eyes?
- When did Character A last appear?
- What does Character B know at this point in the story?
- What is the timeline of events?

### Split-Screen Reference

The ability to view two documents side by side (e.g., a character sheet and the scene
being written, or two scenes that need to be consistent) is cited as transformative
by many writers.

### Flexible Hierarchy

Scrivener's unlimited nesting depth means writers can organize their work however they
see fit. Common organizational patterns include:

- Part > Chapter > Scene
- Act > Sequence > Scene
- Section > Subsection (for non-fiction)
- Single flat list of scenes (for short stories or experimental structures)

### Scalability

Scrivener handles large manuscripts well. Writers report working on 500+ page
manuscripts with hundreds of documents in the binder without significant performance
issues. The compile system can produce coherent output from even very complex project
structures.

---

## What Writers Hate About Scrivener

### Steep Learning Curve

Scrivener's learning curve is its most significant barrier. Multiple surveys and forum
analyses suggest an approximately 40% abandon rate: writers who purchase Scrivener,
attempt to learn it, and give up within the first month.

The primary pain points during onboarding:

- **Compile is incomprehensible**: New users cannot figure out how to produce a
  properly formatted document. The compile system has dozens of options, and the
  relationship between section types, section layouts, and format definitions is
  not intuitive.
- **Too many features visible at once**: The default UI exposes the binder, editor,
  inspector, and toolbar simultaneously. New users feel overwhelmed.
- **Terminology is unfamiliar**: "Binder," "Compile," "Scrivenings" (a view mode),
  "Section Layout" --- these terms are unique to Scrivener and must be learned.
- **Documentation is reference-style**: Scrivener's manual is over 900 pages and is
  organized as a reference rather than a tutorial. Writers who learn by doing
  (rather than by reading) struggle.

### Feature Overwhelm

Most Scrivener users report using only 10-20% of the application's features. The
remaining 80-90% is invisible to them, yet it adds complexity to the UI and
documentation. Features like scriptwriting mode, MathType integration, and linguistic
focus are used by tiny minorities but add cognitive load for everyone.

### Syncing Issues

Scrivener's multi-file project format is fundamentally at odds with file-syncing
services. Dropbox, iCloud, and Google Drive were designed for simple files, not for
directory bundles containing dozens of interdependent files. Writers regularly report:

- **Corruption**: Sync conflicts that corrupt the project, requiring restoration
  from backups.
- **Missing files**: Individual RTF files failing to sync, resulting in blank
  documents.
- **Slow sync**: Large projects with many files taking a long time to sync.
- **iOS sync setup**: Scrivener's iOS app requires specific Dropbox folder placement,
  which confuses users.

Literature & Latte's official recommendation is to close Scrivener before syncing and
wait for sync to complete before opening it on another device. This manual process is
error-prone and frustrating.

### Import/Export Formatting Problems

Moving content into and out of Scrivener is consistently problematic:

- **Word import**: Importing a DOCX file often produces formatting artifacts. Styles
  do not map cleanly.
- **Compile output**: Getting compile output to match specific submission guidelines
  (e.g., standard manuscript format, specific publisher requirements) requires deep
  knowledge of the compile system.
- **Round-tripping**: There is no clean way to export to DOCX, edit in Word (e.g.,
  with an editor's comments), and re-import the changes. This is a significant
  pain point for writers working with traditional publishers.

### No Real-Time Collaboration

Scrivener is a single-user application. There is no mechanism for two writers to
work on the same project simultaneously. For writing partnerships, writing groups,
and writer-editor workflows, this is a significant limitation.

### No AI Assistance

As of 2025, Scrivener has no built-in AI features. There is no grammar checking
beyond basic spell-check, no style suggestions, no AI-assisted brainstorming, no
summarization, and no generative AI integration. Writers who want these features
must copy text out of Scrivener, use external tools, and paste it back.

### Dated UI and Slow Development

Scrivener's UI has not changed significantly since Scrivener 3 (2017 on Mac, 2021
on Windows). It uses native UI frameworks that look increasingly dated compared to
modern applications. Development is slow --- major releases are years apart, and the
Windows version has historically lagged far behind the Mac version.

---

## Alternatives Comparison

### Obsidian

**Approach**: A "second brain" note-taking app built on local Markdown files.

**Strengths**:
- Plain Markdown files stored locally (no vendor lock-in)
- Graph view showing connections between notes
- Extensive plugin ecosystem (1000+ community plugins)
- Wiki-style `[[links]]` for connecting notes
- Fast, performant, handles large vaults well
- Active development and community

**Weaknesses for fiction writing**:
- No manuscript compilation system
- No corkboard or visual outlining
- No built-in metadata/synopsis system for narrative documents
- Graph view is interesting but not directly useful for story structure
- Plugin fragmentation (too many ways to do the same thing)
- Not optimized for long-form linear writing

**Relevance to Sakya**: Obsidian proves that Markdown-based, local-first writing
tools can succeed. Its `[[wiki-link]]` syntax and plugin architecture are worth
studying. But it is a knowledge management tool, not a fiction writing tool.

### Campfire (Campfire Blaze / Campfire Pro)

**Approach**: A plotting and worldbuilding tool focused on story planning.

**Strengths**:
- Purpose-built for fiction writers
- Strong worldbuilding tools (maps, timelines, character profiles)
- Visual timeline for tracking events and character arcs
- Clean, modern UI
- Better onboarding than Scrivener

**Weaknesses**:
- The writing editor is basic (not a serious drafting environment)
- Limited export options
- Subscription pricing model
- Small user base, uncertain long-term viability
- Focuses on planning at the expense of writing

**Relevance to Sakya**: Campfire validates the demand for worldbuilding-integrated
writing tools. Its timeline and character profile features are worth studying. But
it fails as a complete writing environment because the editor is an afterthought.

### World Anvil

**Approach**: A web-based worldbuilding platform with 25+ article templates and deep
entity linking.

**Strengths**:
- Extensive template system (characters, locations, species, magic systems, etc.)
- Deep entity linking (mention a character and get a tooltip with their profile)
- Community features (publishing, sharing, RPG campaign management)
- Powerful relationship mapping
- Good for collaborative worldbuilding

**Weaknesses**:
- Web-only (no offline access)
- Subscription pricing with significant feature gating
- The writing editor is rudimentary
- Designed for worldbuilding encyclopedias, not narrative manuscripts
- Performance issues with very large worlds
- Overwhelming number of templates and options

**Relevance to Sakya**: World Anvil demonstrates the value of structured entity
schemas and deep linking for fiction writers. Its template system is the most
sophisticated in the market. But it is a worldbuilding wiki, not a writing app.

### Plottr

**Approach**: A visual plotting and outlining tool.

**Strengths**:
- Visual timeline with color-coded plotlines
- Character arc tracking
- Scene and chapter organization
- Clean, intuitive UI
- Good integration with Scrivener (export to .scriv)

**Weaknesses**:
- No writing editor at all (it is purely a planning tool)
- Limited worldbuilding features
- Small development team
- Export options are limited

**Relevance to Sakya**: Plottr validates the demand for visual plotting tools. Its
timeline visualization is the best in the market. But it must be used alongside a
separate writing tool, which is exactly the fragmentation problem Sakya should solve.

### Ulysses

**Approach**: A minimalist, distraction-free writing app for Mac and iOS.

**Strengths**:
- Fast learning curve (can start writing immediately)
- Clean, beautiful UI
- Markdown-based (with some extensions)
- iCloud sync that actually works
- Good export/publishing options (including direct WordPress publishing)
- Focused writing experience

**Weaknesses**:
- Mac/iOS only
- Subscription pricing
- Very limited organizational features (flat or shallow hierarchy)
- No corkboard, outliner, or visual planning tools
- No worldbuilding or metadata features
- Not suitable for complex novels with many characters and subplots
- Files stored in proprietary iCloud database, not accessible as plain files

**Relevance to Sakya**: Ulysses proves that a clean, Markdown-based writing
experience with low friction can attract serious writers. Its fast onboarding is
the opposite of Scrivener's steep learning curve. But it sacrifices too much
organizational power to be useful for complex fiction projects.

---

## Relevance to Sakya

### What to Adopt from Scrivener

1. **Three-pane layout**: Binder (left), Editor (center), Inspector (right). This
   layout is proven, familiar to Scrivener users, and ergonomically sound for
   writing workflows. Sakya should use this as the default layout.

2. **Binder tree with unlimited nesting**: The hierarchical tree is the correct
   organizational primitive for long-form writing. Writers need the ability to
   structure their work at arbitrary depth. Sakya should replicate the binder's
   flexibility.

3. **Corkboard view**: The index-card view is one of Scrivener's most distinctive
   and beloved features. Sakya should implement a corkboard that shows children of
   the selected binder item as visual cards with synopses and color coding.

4. **Outliner view**: The spreadsheet-like outliner is essential for writers who
   prefer tabular data views. Sakya should implement an outliner with customizable
   columns.

5. **Split-screen editing**: The ability to view two documents side by side is
   critical for reference-while-writing workflows. Sakya should support this.

6. **Composition/focus mode**: A distraction-free writing mode that hides all UI
   except the writing surface. This is table stakes for any serious writing app.

7. **Snapshots/versioning**: Writers need the ability to save and compare versions
   of their documents. Sakya can improve on Scrivener by leveraging Git-based
   versioning of Markdown files.

8. **Document targets**: Per-document word count targets with visual progress
   indicators help writers maintain momentum.

### Where to Improve on Scrivener

1. **File format**: Sakya should use human-readable Markdown files with YAML
   frontmatter instead of RTF + XML. This enables:
   - Version control with Git (meaningful diffs)
   - Editing with any text editor
   - No vendor lock-in
   - Easy scripting and automation
   - Clean sync with any file-syncing service

2. **Learning curve**: Sakya should use progressive disclosure to reveal features
   as writers need them. The initial experience should be as simple as Ulysses
   (just start writing), with complexity available on demand. Specific strategies:
   - Start with just the editor visible
   - Introduce the binder when the writer creates a second document
   - Introduce the inspector when the writer adds metadata
   - Introduce the corkboard and outliner when the writer has 5+ documents
   - Never require the writer to learn "compile" --- export should be one click

3. **Extensibility with custom entity schemas**: Scrivener's metadata system is
   fixed. Sakya should allow writers to define custom entity types (characters,
   locations, magic systems, etc.) with custom fields, and link them throughout
   the manuscript. This brings World Anvil's power into the writing environment.

4. **Modern editor**: Sakya should use Lexical (a modern, extensible editor
   framework) instead of RTF. This enables:
   - WYSIWYG Markdown editing (write in Markdown, see formatted output)
   - Custom node types (wiki-links, entity references, inline metadata)
   - Plugin architecture for editor extensions
   - Better performance than RTF editors
   - Focus mode, typewriter scrolling, and other modern UX patterns

5. **Output/compile simplification**: Instead of Scrivener's complex compile
   system, Sakya should offer:
   - One-click export to common formats (DOCX, PDF, EPUB)
   - A small number of well-designed templates (Standard Manuscript, Paperback, etc.)
   - Sensible defaults that work for 90% of use cases
   - Advanced customization available but not required

6. **AI integration**: Sakya should integrate AI assistance for:
   - Grammar and style suggestions
   - Continuity checking (flag inconsistencies)
   - Brainstorming and "what-if" exploration
   - Summarization (auto-generate synopses)
   - Character voice consistency

7. **Sync and collaboration**: Sakya's Markdown-based format should enable:
   - Clean Git-based version control
   - Conflict-free sync via standard file-syncing services
   - Future real-time collaboration via CRDT-based editors

### The Positioning Thesis

Sakya should position itself as:

> "Scrivener's organizational power + Ulysses's simplicity + Obsidian's openness +
> World Anvil's entity management + modern AI assistance."

The core insight is that Scrivener proved what writers need (unified environment,
hierarchical organization, visual planning, metadata management) but implemented it
with a dated technology stack and poor onboarding. Sakya can deliver the same value
with modern technology, a gentler learning curve, and an open, human-readable file
format.

---

## Appendix: Scrivener Feature Inventory

For reference, here is a categorized inventory of Scrivener's features, with notes
on priority for Sakya (High / Medium / Low / Skip).

| Feature | Scrivener | Priority for Sakya |
|---|---|---|
| Binder (hierarchical tree) | Core | High |
| Rich text editor | Core | High (but as WYSIWYG Markdown) |
| Corkboard (index cards) | Core | High |
| Outliner (spreadsheet view) | Core | High |
| Inspector (metadata panel) | Core | High |
| Split-screen editing | Core | High |
| Composition mode | Core | High |
| Document targets (word count) | Core | High |
| Labels and status | Core | High |
| Snapshots (versioning) | Core | High (via Git) |
| Custom metadata fields | Core | High |
| Compile/export | Core | High (simplified) |
| Keywords/tags | Secondary | Medium |
| Collections (saved searches) | Secondary | Medium |
| Project search | Secondary | High |
| Inline annotations | Secondary | Medium |
| Footnotes | Secondary | Medium |
| Scriptwriting mode | Secondary | Low |
| Linguistic focus | Niche | Skip |
| MathType integration | Niche | Skip |
| Name generator | Niche | Low |
| Writing history (stats) | Secondary | Medium |
| Project bookmarks | Secondary | Medium |
| Quick reference panels | Secondary | Medium |

---

## Sources and Further Reading

- Scrivener official documentation (https://www.literatureandlatte.com/learn-and-support)
- "Writing with Scrivener" community forums
- NaNoWriMo forums (writing tool discussions)
- "Scrivener vs [X]" comparison articles (multiple sources)
- Literature & Latte blog posts on Scrivener 3 design decisions
- User reviews on Mac App Store, Reddit r/scrivener, and writing communities
