# Writing Methodologies and Story Structure Frameworks

This document surveys the major writing methodologies and story structure
frameworks used by novelists, screenwriters, and narrative designers. Each
methodology implies specific tooling requirements for a writing application
like Sakya.


## The Snowflake Method

### Origin and Philosophy

Randy Ingermanson developed the Snowflake Method as an engineering-inspired
approach to novel writing. The name comes from the mathematical fractal: you
begin with a simple shape and iteratively add detail until you have a complex,
beautiful structure. The core insight is that no one sits down and writes a
novel from scratch in one pass --- instead, the novel emerges through a series
of deliberate expansions.

The method appeals particularly to writers who want structure but resist rigid
outlining. Ingermanson positions it as a middle path: more organized than
pure discovery writing, but more flexible than beat-by-beat outlining.

### The Ten Steps

**Step 1: One-Sentence Summary**
Write a single sentence that captures the entire novel. This should be fifteen
words or fewer, contain no character names (use descriptors like "a rogue
physicist"), and distill the core dramatic question. This sentence often
becomes the hook used in query letters.

Example: "A disgraced detective must solve the murder she's being framed for
before the real killer strikes again."

**Step 2: One-Paragraph Expansion**
Expand the sentence into a full paragraph of approximately five sentences.
The first sentence covers the story setup and the inciting incident. The next
three sentences cover the three major turning points or "disasters" of the
story. The final sentence tells how the story ends. This paragraph functions
as a proto-synopsis.

**Step 3: Character Summaries**
For each major character, write a one-page summary that includes:
- Name
- A one-sentence summary of the character's storyline
- The character's motivation (what they want abstractly)
- The character's goal (what they want concretely)
- The character's conflict (what prevents them from reaching the goal)
- The character's epiphany (what they learn, how they change)
- A one-paragraph summary of the character's storyline

This step often causes revisions to Steps 1 and 2, which is expected. The
method is explicitly iterative.

**Step 4: Multi-Paragraph Expansion**
Expand each sentence from the Step 2 paragraph into its own full paragraph.
Each paragraph should end with a disaster (a turning point that raises stakes
or changes direction) except the last, which tells how the story ends. The
result is a one-page synopsis.

**Step 5: Character Synopses**
Write a one-page synopsis from each major character's point of view. For
minor characters, write a half-page synopsis. Again, this often triggers
revisions to earlier steps.

**Step 6: Four-Page Synopsis**
Expand the one-page synopsis from Step 4 into a four-page synopsis. Each
paragraph from Step 4 becomes roughly a page. This is where subplots begin
to weave in and the narrative logic gets pressure-tested.

**Step 7: Character Charts**
Create detailed character charts: full birth dates, physical descriptions,
history, values, motivations, and --- critically --- how the character
changes over the course of the novel.

**Step 8: Scene List**
Using a spreadsheet or equivalent, list every scene in the novel. For each
scene, record:
- POV character
- What happens
- How many pages (estimated)
- Chapter assignment

A typical novel might have 50--100 scenes. This is the first time the writer
thinks about the book at the scene level.

**Step 9: Scene Descriptions**
For each scene, write a multi-paragraph description. Include the scene's
purpose, any key dialogue, the conflict, and how the scene ends. Many writers
skip this step if their scene list is detailed enough.

**Step 10: First Draft**
Write the novel. By this point, most of the creative decisions have been made
and the writing is more about execution --- voice, prose rhythm, imagery,
dialogue --- than about figuring out what happens next.

### Strengths and Limitations

Strengths:
- Progressive disclosure of complexity prevents overwhelm
- Each step produces a tangible artifact
- Explicitly iterative: earlier steps get revised as understanding deepens
- The scene list in Step 8 maps naturally to a scene-card interface

Limitations:
- Can feel mechanical to intuitive writers
- The method assumes a single-protagonist, plot-driven story
- Steps 6 and 9 feel redundant to many practitioners
- Does not address theme, symbol, or motif development

### Relevance to Sakya

The Snowflake Method practically demands a writing tool that supports
progressive layers of detail. Key features implied:
- A "project summary" field that holds the one-sentence and one-paragraph
  summaries, visible from a high-level dashboard
- Character sheets with structured fields (motivation, goal, conflict,
  epiphany) that can be filled in incrementally
- A scene list / scene card view that supports metadata per scene (POV,
  chapter, estimated length)
- The ability to attach longer descriptions to each scene card
- Revision tracking or versioning at the synopsis level --- writers will
  rewrite their summaries multiple times


## Save the Cat

### Origin and Philosophy

Blake Snyder originally developed Save the Cat for screenwriting, publishing
the first book in 2005. Jessica Brody later adapted it specifically for
novelists in "Save the Cat! Writes a Novel" (2018). The framework's name
comes from a storytelling principle: early in the story, have your hero do
something likable (like saving a cat) to earn audience sympathy.

Save the Cat provides one of the most prescriptive beat sheet structures in
popular use. Its specificity is both its greatest strength (writers always
know what should come next) and its greatest vulnerability (stories can feel
formulaic if the beats are followed too rigidly).

### The 15 Beats

**1. Opening Image (0%--1%)**
A snapshot of the protagonist's life before the story begins. Establishes
tone, mood, and the "before" state. The Opening Image and Final Image should
mirror each other to show transformation.

**2. Theme Stated (5%)**
Someone (usually not the protagonist) states the theme of the story, often
in a single line of dialogue. The protagonist doesn't understand it yet. This
is the lesson the protagonist needs to learn.

Example: In a story about learning to trust, a mentor says "You can't do
everything alone" and the protagonist brushes it off.

**3. Setup (1%--10%)**
Introduce the protagonist's world, their flaws, their relationships, and
what needs fixing. Establish the "six things that need fixing" --- aspects of
the protagonist's life that will be resolved by the end. Plant every
important character and relationship.

**4. Catalyst (10%)**
The inciting incident. Something happens that disrupts the protagonist's
status quo and forces them onto a new path. This is not a choice --- it
happens to the protagonist.

**5. Debate (10%--20%)**
The protagonist resists the call to action. They weigh options, express fear,
seek advice. This section builds tension and shows what's at stake. The
reader should feel the difficulty of the decision.

**6. Break into Two (20%)**
The protagonist makes a choice and crosses the threshold into Act Two. This
is a proactive decision, not something that happens to them. The world of
Act Two should feel distinctly different from Act One.

**7. B Story (22%)**
A secondary storyline begins, often a love interest or mentorship. The B Story
carries the theme and provides the protagonist with the tools (emotional,
intellectual, relational) they will need for the climax.

**8. Fun and Games (20%--50%)**
The "promise of the premise." This is why the audience showed up. If the
premise is "a chef must win a cooking competition," this section is full of
cooking scenes. The tone is often lighter. The protagonist experiences
initial success in their new world.

**9. Midpoint (50%)**
A major turning point at the exact center of the story. Either a false
victory (things seem great but aren't) or a false defeat (things seem
terrible but the protagonist is actually on the right track). The stakes
are raised. The ticking clock often starts here.

**10. Bad Guys Close In (50%--75%)**
External antagonists regroup and apply pressure. Internal doubts and flaws
resurface. The team (if there is one) starts to fracture. Things get
progressively worse.

**11. All Is Lost (75%)**
The lowest point. The protagonist experiences a major defeat. Often involves
a "whiff of death" --- a literal death, a metaphorical death, or a reminder
of mortality. Something important is lost.

**12. Dark Night of the Soul (75%--80%)**
The emotional aftermath of All Is Lost. The protagonist wallows, grieves,
or reflects. This is the moment before the breakthrough. The protagonist must
confront their deepest flaw.

**13. Break into Three (80%)**
The "aha moment." The protagonist synthesizes everything they've learned ---
from the A Story and the B Story --- and devises a new plan. They understand
the theme now.

**14. Finale (80%--99%)**
The protagonist executes the plan, confronts the antagonist, and resolves the
story. The Finale has its own five-point structure:
1. Gathering the team
2. Executing the plan
3. The High Tower Surprise (a twist)
4. Digging Deep Down (protagonist uses inner strength)
5. Execution of the new plan

**15. Final Image (99%--100%)**
A snapshot of the protagonist's life after the story. Mirrors the Opening
Image to demonstrate change. If the Opening Image showed isolation, the
Final Image might show community.

### Beat Sheet as Percentage

Snyder's system uses page counts (originally for 110-page screenplays) that
translate to percentages for novels of any length. This makes it adaptable
to novellas, short novels, and epic-length works. A 80,000-word novel would
place the Catalyst at roughly word 8,000 and the Midpoint at word 40,000.

### Ten Story Genres

Save the Cat categorizes all stories into ten genres, each with specific
conventions:
1. Monster in the House (horror, thriller)
2. Golden Fleece (quest, road trip, heist)
3. Out of the Bottle (wish fulfillment, body swap)
4. Dude with a Problem (ordinary person, extraordinary situation)
5. Rites of Passage (life transitions, coming of age)
6. Buddy Love (love stories, buddy comedies, pet stories)
7. Whydunit (detective, mystery, courtroom)
8. The Fool Triumphant (underdog, fish out of water)
9. Institutionalized (group dynamics, family, organization)
10. Superhero (special person in ordinary world)

Each genre has its own required "ingredients" --- for instance, Monster in
the House requires a monster, a house (a confined space), and a sin (the
reason the monster is unleashed).

### Relevance to Sakya

Save the Cat is the most template-friendly methodology. Features implied:
- A beat sheet template with all 15 beats, assignable to scenes or chapters
- Percentage-based positioning: given total word count, show where each beat
  should fall and where it actually falls
- Genre selection that populates genre-specific conventions
- A dual-timeline view showing A Story and B Story in parallel
- Scene cards with beat assignment metadata


## Three-Act Structure

### Origin and Philosophy

The Three-Act Structure is the oldest and most universal story framework,
traceable to Aristotle's "Poetics" (335 BCE). Aristotle observed that
effective stories have a beginning, middle, and end, with each part serving
a distinct function. Nearly every other framework in this document is a
refinement or elaboration of the Three-Act Structure.

### The Three Acts

**Act One: Setup (roughly 25% of the story)**

Act One establishes the protagonist, their world, the central conflict,
and the stakes. Key components:
- Exposition: who, what, where, when
- Inciting Incident: the event that disrupts the status quo
- First Plot Point / First Turning Point: the moment where the protagonist
  commits to the central conflict and the story shifts into Act Two

Act One answers the question: "What is this story about?"

**Act Two: Confrontation (roughly 50% of the story)**

Act Two is the longest and most challenging act. The protagonist pursues
their goal while facing escalating obstacles. Key components:
- Rising Action: complications increase in severity
- Midpoint: a major reversal or revelation that shifts the story's direction
- Pinch Points: moments where the antagonist's power is felt directly
- Second Plot Point / Second Turning Point: a crisis that propels the story
  into Act Three

Act Two answers the question: "What stands in the way?"

**Act Three: Resolution (roughly 25% of the story)**

Act Three brings the conflict to its climax and resolves the story. Key
components:
- Climax: the final confrontation between protagonist and antagonist
- Resolution: the aftermath, showing the new status quo
- Denouement: loose ends are tied up

Act Three answers the question: "How does it end?"

### Variations

Different practitioners divide the acts differently. Some use a four-act
structure (splitting Act Two at the midpoint). Others use a five-act
structure (Freytag's Pyramid: exposition, rising action, climax, falling
action, denouement). The underlying principles remain consistent.

### Relevance to Sakya

The Three-Act Structure is the backbone. Features implied:
- An act overlay that can be applied to any project, dividing the manuscript
  into color-coded acts
- Flexible act boundaries that the writer can drag/adjust
- Act-level summary fields and goals
- Integration with more detailed frameworks (Save the Cat's 15 beats map
  onto the Three-Act Structure)


## Story Grid

### Origin and Philosophy

Shawn Coyne developed the Story Grid after decades as a book editor. His
central insight is that stories can be analyzed with the rigor of an
engineering discipline. The Story Grid provides both a diagnostic tool
(for editing existing manuscripts) and a generative tool (for planning
new ones).

Coyne's approach is analytical: he treats stories as systems with
identifiable components, measurable values, and testable principles.

### Core Concepts

**The Five Commandments of Storytelling**

Every unit of story --- from a single beat to the entire novel --- must
contain five elements:
1. Inciting Incident (causal or coincidental)
2. Turning Point Progressive Complication (the most important complication,
   which forces a crisis)
3. Crisis (a dilemma: best bad choice or irreconcilable goods)
4. Climax (the protagonist's choice made manifest in action)
5. Resolution (the consequence of the climax)

These five commandments operate fractally: a scene has them, a sequence has
them, an act has them, and the global story has them.

**Value Shifts**

Every scene must turn on a value --- a human universal like
life/death, love/hate, justice/injustice, freedom/slavery. The value
must change polarity during the scene (positive to negative, or negative
to positive). A scene where the value doesn't shift is a scene that
doesn't work.

Coyne tracks value shifts on a spreadsheet, assigning a positive or
negative charge to each scene. When plotted as a graph, the pattern of
charges reveals the story's rhythm and energy.

**The Six Core Questions**

Before writing, a Story Grid practitioner answers:
1. What is the genre? (Content genre, not marketing genre)
2. What are the conventions of that genre?
3. What are the obligatory scenes of that genre?
4. What is the point of view?
5. What are the objects of desire? (Want and Need)
6. What is the controlling idea/theme?

**Genre in the Story Grid**

Coyne's genre system is more granular than most. He identifies content
genres (Action, Horror, Crime, Love, Performance, Society, Status,
Worldview, Morality) each with specific obligatory scenes and conventions.
For example, a Love story requires:
- Obligatory scenes: Lovers meet, first kiss/intimate connection,
  confession of love, proof of love, lovers break up, lovers reunite
- Conventions: Triangle, helpers/harmers, gender-balanced scenes,
  external need (subplot), moral weight

### Levels of Story

**Beat**: The smallest unit. A single exchange of action/reaction.

**Scene**: A collection of beats that turns on a single value. Must contain
all five commandments. A novel typically has 50--70 scenes.

**Sequence**: A collection of scenes building toward a larger turning point.
A typical act contains 3--5 sequences.

**Act**: A major division of the story.
- Beginning Hook (Act One): Incites the protagonist into the story
- Middle Build (Act Two): Complicates and escalates
- Ending Payoff (Act Three): Resolves the global story

**Global Story**: The entire arc from beginning to end.

### The Spreadsheet

The Story Grid Spreadsheet is the methodology's primary analytical tool.
For each scene, the writer records:
- Scene number
- Word count
- Story event (what happens)
- Value shift (positive/negative charge and which value)
- Turning point type
- POV character
- Period/time
- Duration
- Location

### Relevance to Sakya

The Story Grid is the most data-intensive methodology. Features implied:
- Scene-level metadata fields for value shift, turning point type, and
  the five commandments
- A value shift graph that plots positive/negative charges across scenes
- Genre-specific templates that pre-populate obligatory scenes and
  conventions as a checklist
- The ability to analyze a manuscript at multiple levels (beat, scene,
  sequence, act, global)
- A spreadsheet-like view alongside or integrated with the scene card view


## The Hero's Journey

### Origin and Philosophy

Joseph Campbell identified the "monomyth" in "The Hero with a Thousand
Faces" (1949) by analyzing myths from cultures worldwide. He found a
common pattern: a hero ventures from the ordinary world into a region of
supernatural wonder, wins a decisive victory, and returns transformed.

Christopher Vogler adapted Campbell's work for screenwriters in "The
Writer's Journey" (1992), simplifying the structure and making it more
practical for modern storytelling.

### The Twelve Stages

**Act One: Departure**

1. **Ordinary World**: The hero's normal life before the adventure. Establishes
   baseline, shows flaws, creates empathy. Luke Skywalker on Tatooine.
   Frodo in the Shire.

2. **Call to Adventure**: Something disrupts the ordinary world. A challenge,
   a message, a crisis. Princess Leia's hologram. Gandalf's arrival with
   news of the Ring.

3. **Refusal of the Call**: The hero hesitates. Fear, obligation, doubt.
   This makes the eventual acceptance more meaningful. Luke says he can't
   leave the farm. Frodo tries to give the Ring to Gandalf.

4. **Meeting the Mentor**: The hero encounters a figure who provides guidance,
   training, or a gift. Obi-Wan Kenobi. Gandalf (again, in mentor capacity).

5. **Crossing the First Threshold**: The hero commits to the adventure and
   leaves the ordinary world. Luke leaves Tatooine. The Fellowship departs
   Rivendell.

**Act Two: Initiation**

6. **Tests, Allies, Enemies**: The hero navigates the new world, forming
   alliances, facing trials, and identifying enemies. The Mos Eisley Cantina.
   Moria and Lothlorien.

7. **Approach to the Inmost Cave**: The hero approaches the central challenge.
   Tension builds. Plans are made. Approaching the Death Star. The journey
   to Mordor grows more perilous.

8. **Ordeal**: The hero faces their greatest challenge and experiences a
   symbolic death and rebirth. The trash compactor / rescue of Leia.
   Shelob's lair.

9. **Reward (Seizing the Sword)**: The hero gains something valuable ---
   knowledge, power, a literal object. The Death Star plans. The destruction
   of the Ring (in the extended sense of the reward being the quest fulfilled).

**Act Three: Return**

10. **The Road Back**: The hero begins the return journey. New complications
    arise. The chase from the Death Star. The Scouring of the Shire.

11. **Resurrection**: A final test where the hero must use everything they've
    learned. A second, greater death-and-rebirth. Luke trusts the Force.
    Frodo's mercy toward Gollum pays off.

12. **Return with the Elixir**: The hero returns to the ordinary world,
    transformed, bearing a gift (literal or metaphorical) for their
    community. Luke is a hero of the Rebellion. Frodo sails to the
    Undying Lands, and the Shire is renewed.

### Archetypes

Campbell and Vogler also identified recurring character archetypes:
- **Hero**: The protagonist who grows through the journey
- **Mentor**: The wise guide (may have a dark side)
- **Threshold Guardian**: Tests the hero's commitment
- **Herald**: Announces the call to adventure
- **Shapeshifter**: Loyalty is uncertain; creates suspicion
- **Shadow**: The antagonist or dark mirror of the hero
- **Trickster**: Provides comic relief and challenges the status quo
- **Allies**: Companions who support the hero

### Criticisms and Adaptations

The Hero's Journey has been criticized for:
- Centering a single (often male) protagonist
- Assuming a quest/adventure structure
- Flattening diverse mythological traditions into a single template
- Over-application (not every story is a hero's journey)

Modern adaptations address some of these concerns. Maureen Murdock's
"Heroine's Journey" inverts several stages. Kim Hudson's "Virgin's
Promise" offers an alternative focusing on self-actualization rather
than external quests.

### Relevance to Sakya

The Hero's Journey is widely taught and widely used. Features implied:
- A 12-stage template mappable to scenes or chapters
- Archetype assignments for characters (a character can be tagged as
  "Mentor" or "Shadow")
- Visual journey map showing progression through the stages
- Alternative journey templates (Heroine's Journey, Virgin's Promise)


## Dan Harmon's Story Circle

### Origin and Philosophy

Dan Harmon, creator of "Community" and co-creator of "Rick and Morty,"
developed the Story Circle as a simplified, practical version of the Hero's
Journey. Harmon stripped Campbell's framework down to eight steps and
emphasized its circular nature --- the character ends where they began,
but changed.

Harmon originally developed it for television episodes, where tight structure
is essential due to time constraints. It has since been adopted by novelists
and game writers.

### The Eight Steps

The Story Circle divides into two halves: the top half (comfort zone,
order, the known) and the bottom half (the unknown, chaos, adventure). The
character's journey crosses between these two zones.

**1. You (A character is in a zone of comfort)**
Establish who the character is and what their world looks like. This is
the status quo.

**2. Need (But they want something)**
Something is missing. The character has an unmet desire --- conscious
(what they want) or unconscious (what they need).

**3. Go (They enter an unfamiliar situation)**
The character crosses a threshold into new territory. This can be physical,
emotional, social, or psychological.

**4. Search (They adapt to it)**
The character tries things, fails, learns, and adapts. They develop
new skills and relationships. This is the "road of trials."

**5. Find (They get what they wanted)**
The character achieves their conscious goal. But this is typically at the
midpoint, and achieving the goal reveals new complications or the true
nature of what they actually need.

**6. Take (But pay a heavy price for it)**
The victory comes at a cost. Sacrifice, loss, betrayal, or painful
revelation. The character cannot return to who they were.

**7. Return (They return to their familiar situation)**
The character re-enters their original world, but they and/or the world
have changed.

**8. Change (Having changed)**
The character is fundamentally different from Step 1. They have grown,
learned, or been transformed. The circle is complete.

### The Circle's Geometry

Harmon emphasizes the visual circle:
- Steps 1 and 5 are directly opposite (comfort vs. achievement)
- Steps 3 and 7 are the threshold crossings (going and returning)
- The top half is order; the bottom half is chaos
- Movement is always clockwise

This geometric quality makes the Story Circle particularly well-suited to
visual tools.

### Application to Different Scales

Like the Story Grid's Five Commandments, the Story Circle operates
fractally:
- A single scene can follow all eight steps
- A chapter can follow all eight steps
- An entire novel follows all eight steps
- A series arc follows all eight steps

### Relevance to Sakya

The Story Circle's simplicity and visual nature make it ideal for UI:
- A literal circular diagram where scenes or chapters can be placed at
  each of the eight positions
- Drag-and-drop scene assignment to circle positions
- Nested circles for different scales (scene-level, chapter-level,
  act-level)
- Color-coded halves (order vs. chaos)


## Planning Approaches

### The Spectrum

Writers generally fall somewhere on a spectrum from pure plotter to pure
pantser, with most occupying a middle ground.

**Plotters (Outliners)**

Plotters plan extensively before writing. They create detailed outlines,
beat sheets, character profiles, and scene lists. The first draft is largely
an execution of the plan.

Advantages:
- Fewer structural problems in the first draft
- Easier to maintain consistency in complex narratives
- Reduces "writer's block" (you always know what to write next)
- Easier to estimate timelines and progress

Disadvantages:
- Can feel creatively constraining
- The plan may not survive contact with the actual writing
- Risk of over-planning and never starting the draft
- Characters can feel like puppets following a script

**Pantsers (Discovery Writers)**

Pantsers write by the seat of their pants, discovering the story as they
go. They may have a vague sense of direction but let the writing lead.

Advantages:
- Greater sense of creative freedom and surprise
- Characters feel more organic and alive
- The writer's own enthusiasm stays high (they're discovering, not executing)
- Can produce unexpected, original plot developments

Disadvantages:
- Higher risk of structural problems
- May require extensive revision
- Can lead to dead ends and abandoned manuscripts
- Difficulty maintaining consistency in long works

**Plantsers (Hybrid)**

Most working writers are plantsers to some degree. Common hybrid approaches:
- Outline the major beats but discovery-write the scenes
- Discovery-write the first act, then outline the rest
- Write detailed character profiles but let the plot emerge
- Use a loose scene list that can be rearranged as the story evolves

### Scene and Chapter Planning

**One Sentence Per Scene**

A practical planning technique: write one sentence describing each scene.
This creates a lightweight outline that's easy to rearrange and doesn't
constrain execution. Example:

> 1. Elena discovers the letter in her grandmother's attic.
> 2. She brings it to Professor Marsh, who recognizes the handwriting.
> 3. Marsh reveals he knew her grandmother during the war.
> 4. Elena visits the address on the letter and finds it abandoned.

**Cause and Effect Chains**

Every scene should be linked to the next by "therefore" or "but" rather
than "and then." This ensures causal connections:

- Elena finds the letter, THEREFORE she visits Professor Marsh.
- Marsh recognizes the handwriting, BUT he refuses to say more.
- Elena presses him, THEREFORE he reveals the wartime connection.
- She visits the address, BUT finds it abandoned.

This technique, articulated by Trey Parker and Matt Stone of South Park,
prevents episodic, disconnected narratives.

**Scene Purpose Checklist**

Before including a scene, experienced writers verify it serves at least
two of these purposes:
1. Advances the plot
2. Reveals character
3. Provides necessary information
4. Establishes mood or atmosphere
5. Raises stakes or tension

### Planning Tools and Artifacts

Common planning artifacts that a writing tool should support:

- **Synopsis**: One paragraph to several pages summarizing the story
- **Beat sheet**: A list of major story beats with brief descriptions
- **Scene list**: Every scene with one-line descriptions, sortable and
  rearrangeable
- **Chapter plan**: Scenes grouped into chapters with chapter-level notes
- **Timeline**: Chronological ordering of events (which may differ from
  narrative ordering)
- **Character arc tracking**: Where each character is emotionally at each
  major beat
- **Subplot tracking**: How secondary plotlines weave through the main
  narrative

### Relevance to Sakya

Supporting the full plotter-pantser spectrum is essential. Features implied:
- Templates for each planning style: detailed (Snowflake, Save the Cat),
  light (scene list only), and blank (pure discovery)
- A scene card view that works equally well for pre-planned scenes and
  scenes added during drafting
- Drag-and-drop reordering of scenes and chapters
- The ability to attach structure overlays (beat sheets, act markers) at
  any point in the process --- not just at the beginning
- A "cause and effect" view or annotation system that lets writers track
  causal chains between scenes
- Scene purpose tags (advances plot, reveals character, etc.)
- Multiple synopsis/summary fields at different levels of detail


## Comparative Analysis

### Framework Overlap

These methodologies overlap significantly. A mapping of key concepts:

| Concept | Three-Act | Hero's Journey | Save the Cat | Story Circle | Story Grid |
|---|---|---|---|---|---|
| Beginning | Act One | Ordinary World | Opening Image -- Debate | You, Need | Beginning Hook |
| Inciting event | Inciting Incident | Call to Adventure | Catalyst | Go | Inciting Incident |
| First threshold | First Plot Point | Crossing Threshold | Break into Two | Go | First Turning Point |
| Middle exploration | Act Two (first half) | Tests, Allies | Fun and Games | Search | Progressive Complications |
| Midpoint reversal | Midpoint | Ordeal | Midpoint | Find | Midpoint (value shift) |
| Escalation | Act Two (second half) | Road Back prep | Bad Guys Close In | Take | Crisis builds |
| Low point | Second Plot Point | Road Back | All Is Lost | Take | Crisis |
| Climax | Act Three | Resurrection | Finale | Return | Climax |
| Resolution | Denouement | Return with Elixir | Final Image | Change | Resolution |

### Choosing a Framework

Different frameworks suit different projects:

- **Literary fiction with internal arcs**: Story Grid (value shifts track
  psychological change well)
- **Genre fiction (thriller, romance, mystery)**: Save the Cat (genre-
  specific conventions are invaluable)
- **Epic fantasy / sci-fi**: Hero's Journey (maps naturally to quest
  narratives)
- **Television / serial fiction**: Story Circle (episode-level structure)
- **First novel / learning writer**: Snowflake Method (step-by-step
  process reduces overwhelm)
- **Character-driven literary fiction**: Three-Act Structure with
  character arc focus (minimal scaffolding, maximum flexibility)


## Comprehensive Relevance to Sakya

### Core Architectural Implications

Studying these methodologies reveals a consistent set of needs that
Sakya's architecture must address:

**1. Layered Structure**
Every methodology operates at multiple levels (beat, scene, sequence/chapter,
act, global). Sakya must support this fractal structure natively. Scenes
are the atomic unit, but they nest into chapters, which nest into acts
(or parts), which compose the full manuscript.

**2. Metadata-Rich Scene Cards**
Scenes are not just text containers. They carry metadata:
- POV character
- Location and time
- Story beat assignment (from any framework)
- Value shift (Story Grid)
- Purpose tags
- Status (planned, drafted, revised)
- Word count (actual and target)
- Causal links to other scenes

**3. Multiple Structure Overlays**
Writers may use multiple frameworks simultaneously (Three-Act Structure for
the macro view, Save the Cat beats for pacing, Story Grid value tracking for
scene-level analysis). Sakya should support layering these rather than
forcing a single choice.

**4. Template System**
Each methodology implies templates:
- Beat sheet templates (Save the Cat, Hero's Journey, Story Circle)
- Character sheet templates (Snowflake steps 3 and 7)
- Genre convention checklists (Story Grid, Save the Cat genres)
- Planning templates at different levels of detail

**5. Visual Tools**
Several methodologies are inherently visual:
- Story Circle is literally a circle
- Save the Cat uses percentage-based positioning
- Story Grid uses a value-shift graph
- The Hero's Journey maps to a circular or linear journey diagram

**6. Flexibility for Discovery Writers**
Not every writer plans. Sakya must be equally useful for:
- A plotter who creates a full Snowflake-method structure before writing
  a word
- A pantser who starts with a blank manuscript and adds structure after
  the fact
- A plantser who plans loosely and adjusts as they go

This means structure overlays must be optional and addable at any point,
not required upfront.

**7. Cross-Referencing**
Characters, locations, and plot threads weave through scenes. The tool
must support linking scenes to characters, locations, and themes, and
navigating those links in both directions (from a character, see all
their scenes; from a scene, see all characters present).

### Feature Priority Matrix

Based on frequency across methodologies:

| Feature | Methodologies that need it | Priority |
|---|---|---|
| Scene cards with metadata | All | Critical |
| Drag-and-drop reordering | All | Critical |
| Act/part divisions | All | Critical |
| Character profiles | Snowflake, Hero's Journey, STC | High |
| Beat sheet templates | STC, Hero's Journey, Story Circle | High |
| Multiple structure overlays | Story Grid + any other | High |
| Synopsis at multiple levels | Snowflake, Three-Act | Medium |
| Value shift tracking | Story Grid | Medium |
| Visual circle/journey diagram | Story Circle, Hero's Journey | Medium |
| Percentage-based beat positioning | STC | Medium |
| Genre convention checklists | STC, Story Grid | Lower |
| Cause-and-effect chain view | Planning approaches | Lower |

This analysis should directly inform Sakya's feature roadmap and entity
schema design.
