---
name: frontend-design
description: Create distinctive, production-grade frontend interfaces with high design quality. Use this skill when the user asks to build web components, pages, or applications. Generates creative, polished code that avoids generic AI aesthetics.
license: Complete terms in LICENSE.txt
---

This skill guides creation of distinctive, production-grade Elm UI v2 interfaces that avoid generic "AI slop" aesthetics. **Design thinking and aesthetics are primary**—made actionable through Elm UI v2's native capabilities.

The user provides frontend requirements: a component, page, application, or interface to build. They may include context about the purpose, audience, or technical constraints.

**Core Principles:**
1. Design aesthetics are the most important part—make them actionable and complete
2. Use ONLY native Elm UI v2 helpers—never raw CSS via `htmlAttribute` for visual effects
3. Emphasize using existing design system values (color systems, typography scales, spacing constants)—avoid magic values
4. Keep patterns general/reusable across Elm codebases

**CRITICAL:** Choose a clear conceptual direction and execute it with precision. Bold maximalism and refined minimalism both work—the key is intentionality, not intensity.

Implement working Elm UI v2 code that is:
- Production-grade and functional
- Visually striking and memorable
- Cohesive with a clear aesthetic point-of-view
- Meticulously refined in every detail

**Match implementation complexity to the aesthetic vision.** Maximalist designs need elaborate code with extensive animations and effects. Minimalist or refined designs need restraint, precision, and careful attention to spacing, typography, and subtle details. Elegance comes from executing the vision well.

Interpret creatively and make unexpected choices that feel genuinely designed for the context. No design should be the same. Vary between light and dark themes, different fonts, different aesthetics. NEVER converge on common choices (Space Grotesk, for example) across generations.

---

## Section 1: Design Thinking (Primary)

Before coding, commit to a BOLD aesthetic direction:

- **Purpose & Audience** - What problem does this solve? Who uses it?
- **Aesthetic Direction** - Pick an extreme and commit:
  - Brutally minimal
  - Maximalist chaos
  - Retro-futuristic
  - Organic/natural
  - Luxury/refined
  - Playful/toy-like
  - Editorial/magazine
  - Brutalist/raw
  - Art deco/geometric
  - Soft/pastel
  - Industrial/utilitarian
- **Differentiation** - What's the one unforgettable thing?
- **Constraints** - Technical/accessibility requirements

**Actionable output:** The chosen direction informs spacing density, typography scale ratio, color dominance ratios, animation timing curves.

---

## Section 2: Design System Values (No Magic Numbers)

**Principle:** Always use existing design system values. If a project has ColorSystem, Spacing, Typography modules—use them. If not, define semantic constants.

```elm
-- GOOD: Use semantic values from existing modules
Ui.spacing Spacing.md
Ui.background colors.surface
Font.size Typography.heading2

-- BAD: Magic numbers scattered in code
Ui.spacing 24
Ui.background (Ui.rgb 245 245 245)
Font.size 28
```

**If no design system exists, create semantic constants:**

```elm
-- Spacing scale (8px base, ratio 1.5 or 2)
spacing =
    { xs = 4, sm = 8, base = 16, md = 24, lg = 32, xl = 48, xxl = 64 }

-- Typography scale (ratio 1.25)
typography =
    { body = 16, h4 = 20, h3 = 25, h2 = 31, h1 = 39, hero = 49 }

-- Color palette (derive from single accent)
colors =
    { primary = Ui.rgb 79 70 229
    , background = Ui.rgb 250 250 252
    , surface = Ui.rgb 255 255 255
    , text = Ui.rgb 17 24 39
    , textMuted = Ui.rgba 17 24 39 0.6
    }
```

---

## Section 3: Aesthetics Made Actionable with Native Elm UI v2

### Typography

**Principle:** Choose distinctive fonts; pair display font with refined body font. Use your design system's typography scale.

```elm
import Ui.Font as Font

-- Use typography scale from design system
Ui.el
    [ Font.size typography.h1
    , Font.weight 700
    , Font.letterSpacing 1.5
    , Font.family [ Font.typeface "Your Display Font", Font.sansSerif ]
    ]
    (Ui.text "Hero Headline")

-- Font variants for refinement
Font.variants [ Font.smallCaps, Font.tabularNumbers ]

-- Text gradient (native)
Font.gradient
    (Gradient.linear (Ui.turns 0.25)
        [ Gradient.percent 0 colors.primary
        , Gradient.percent 100 colors.accent
        ])
```

### Color & Theme

**Principle:** Dominant color with sharp accents. Use a single color system source.

```elm
-- Apply from color system consistently
Ui.el
    [ Ui.background colors.surface
    , Font.color colors.text
    ]
    content

-- Create colors from semantic base (native)
Ui.rgb 79 70 229      -- Integer RGB
Ui.rgba 0 0 0 0.15    -- With alpha
```

### Spatial Composition

**Principle:** Unexpected layouts. Asymmetry. Overlap. Use design system spacing.

```elm
-- Asymmetric layout with portions
Ui.row [ Ui.spacing spacing.lg ]
    [ Ui.el [ Ui.width (Ui.portion 2) ] narrowColumn
    , Ui.el [ Ui.width (Ui.portion 5) ] wideColumn
    ]

-- Overlap with native inFront/behindContent
Ui.el
    [ Ui.inFront overlappingElement
    , Ui.padding spacing.xl  -- From design system
    ]
    mainContent

-- Diagonal flow via native rotation
Ui.el
    [ Ui.rotate (Ui.turns 0.02)  -- Subtle tilt
    , Ui.move (Ui.right 20)      -- Native translate
    ]
    content
```

### Motion (Native Ui.Anim)

**Principle:** Focus on high-impact moments. Use native animation system.

```elm
import Ui.Anim as Anim

-- Hover state transition (native)
Anim.hovered (Anim.ms 200)
    [ Anim.backgroundColor colors.hover
    , Anim.scale 1.02
    ]

-- Entrance animation
Anim.intro (Anim.ms 400)
    [ Anim.opacity 0
    , Anim.y 20
    ]

-- Spring physics for organic feel
Anim.transition (Anim.ms 300)
    |> Anim.withTransition (Anim.spring { wobble = 0.5, quickness = 1.0 })

-- Custom easing
Anim.bezier 0.4 0 0.2 1  -- ease-out

-- Premade animations
Anim.spinning (Anim.ms 1000)   -- Loading spinners
Anim.pulsing (Anim.ms 2000)    -- Attention pulse
Anim.bouncing (Anim.ms 500) 10 -- Bounce with 10px height

-- Keyframe sequences
Anim.keyframes
    [ Anim.set [ Anim.opacity 0, Anim.scale 0.8 ]
    , Anim.step (Anim.ms 200) [ Anim.opacity 1, Anim.scale 1.05 ]
    , Anim.step (Anim.ms 100) [ Anim.scale 1.0 ]
    ]

-- Staggered reveals (animation-delay via parent triggers)
Anim.hoveredWith
    [ Anim.wait (Anim.ms 50)
    , Anim.step (Anim.ms 200) [ Anim.opacity 1 ]
    ]
```

### Shadows & Depth (Native Ui.Shadow)

**Principle:** Create atmosphere and depth with layered shadows.

```elm
import Ui.Shadow as Shadow

-- Multiple layered shadows for natural depth
Shadow.shadows
    [ { x = 0, y = 4, size = 0, blur = 24, color = Ui.rgba 0 0 0 0.15 }
    , { x = 0, y = 1, size = 0, blur = 3, color = Ui.rgba 0 0 0 0.1 }
    ]

-- Inset shadow for pressed states
Shadow.inner { x = 0, y = 2, size = 0, blur = 4, color = Ui.rgba 0 0 0 0.1 }

-- Text shadow for depth
Shadow.font { offset = ( 2, 2 ), blur = 4, color = Ui.rgba 0 0 0 0.2 }
```

### Gradients & Backgrounds (Native Ui.Gradient)

**Principle:** Create atmosphere. Use native gradient system.

```elm
import Ui.Gradient as Gradient

-- Linear gradient
Ui.backgroundGradient
    [ Gradient.linear (Ui.turns 0.25)
        [ Gradient.percent 0 (Ui.rgb 30 30 40)
        , Gradient.percent 100 (Ui.rgb 60 50 70)
        ]
    ]

-- Radial gradient
Ui.backgroundGradient
    [ Gradient.radial Gradient.center
        [ Gradient.percent 0 colors.primary
        , Gradient.percent 100 colors.background
        ]
    ]

-- Conic gradient (for pie charts, decorative)
Ui.backgroundGradient
    [ Gradient.conic Gradient.center (Ui.turns 0)
        [ ( Ui.turns 0, colors.primary )
        , ( Ui.turns 0.5, colors.accent )
        , ( Ui.turns 1, colors.primary )
        ]
    ]

-- Gradient borders (native)
Ui.borderGradient
    { gradient = Gradient.linear (Ui.turns 0.25) [...]
    , background = colors.surface
    }

-- Layered elements for texture
Ui.el
    [ Ui.behindContent backgroundTexture
    , Ui.inFront decorativeOverlay
    ]
    mainContent
```

### Borders & Shapes (Native)

```elm
-- Border width and color
Ui.border 1
Ui.borderWith { top = 1, right = 0, bottom = 1, left = 0 }
Ui.borderColor colors.border

-- Rounded corners
Ui.rounded spacing.sm                    -- From design system
Ui.roundedWith { topLeft = 16, topRight = 16, bottomLeft = 0, bottomRight = 0 }
Ui.circle                                -- Perfect circle
```

---

## Section 4: Elm UI v2 Layout Fundamentals

### Length Types

- `Ui.px Int` - Fixed pixel size
- `Ui.fill` - Fill available space (equivalent to `portion 1`)
- `Ui.portion Int` - Proportional fill
- `Ui.shrink` - Only as big as content

### Core Layout

```elm
Ui.row [ Ui.spacing spacing.base ] children     -- Horizontal
Ui.column [ Ui.spacing spacing.base ] children  -- Vertical
Ui.row [ Ui.wrap, Ui.spacing spacing.sm ] tags  -- Wrapping
Ui.el [ Ui.padding spacing.md ] content         -- Single wrapper
```

### Behavior Rules

- **No margins** - Use `padding` (edge-to-content) and `spacing` (between children)
- `row` children flow left-to-right; `column` top-to-bottom
- `wrap` enables flex-wrap for natural line-breaking
- `shrink` prevents stretching; `fill` expands to fill
- `portion n` divides space proportionally among `fill`/`portion` siblings

### Responsive Design (Native Ui.Responsive)

```elm
import Ui.Responsive as Responsive

-- Define breakpoints
breakpoints =
    Responsive.breakpoints Mobile
        [ ( 640, Tablet )
        , ( 1024, Desktop )
        ]

-- Responsive visibility
Responsive.visible breakpoints [ Desktop ]  -- Only on desktop

-- Responsive layout switching
Responsive.rowWhen breakpoints [ Desktop ]
    [ Ui.spacing spacing.lg ]
    children  -- Row on desktop, column on mobile

-- Responsive font size
Responsive.fontSize breakpoints
    (\device ->
        case device of
            Mobile -> Responsive.value 24
            Tablet -> Responsive.value 32
            Desktop -> Responsive.fluid 32 48
    )
```

---

## Section 5: Building Custom Components with Attr.Attr

The Attr pattern enables composable, type-safe component configuration.

```elm
module Button exposing (Attribute, view, primary, secondary, onClick, disabled)

import Attr

-- 1. Opaque attribute type
type alias Attribute msg = Attr.Attr (Config msg)

-- 2. Internal configuration with sensible defaults
type alias Config msg =
    { variant : Variant
    , onClick : Maybe msg
    , disabled : Bool
    }

type Variant = Primary | Secondary | Ghost

defaultConfig : Config msg
defaultConfig =
    { variant = Primary, onClick = Nothing, disabled = False }

-- 3. Semantic attribute constructors
primary : Attribute msg
primary = Attr.attr (\c -> { c | variant = Primary })

secondary : Attribute msg
secondary = Attr.attr (\c -> { c | variant = Secondary })

onClick : msg -> Attribute msg
onClick msg = Attr.attr (\c -> { c | onClick = Just msg })

disabled : Attribute msg
disabled = Attr.attr (\c -> { c | disabled = True })

-- 4. View applies config
view : List (Attribute msg) -> String -> Element msg
view attrs label =
    let
        config = Attr.toAttrs defaultConfig attrs
    in
    -- Render using config.variant, config.onClick, etc.
    -- Use design system values: colors.primary, spacing.md, etc.
```

**Best Practices:**
- Semantic names (`primary`, `ghost`) over raw values
- `Maybe` for optional callbacks, `Nothing` default
- Keep config record flat
- Attribute order doesn't matter (compose via fold)
- Required data in function args, optional config in attrs
- **Use design system values internally, not magic numbers**


## Section 5bis: Use icons

Use the Phosphor icons, available within the `Components.PhosphorIcon` module, to
give your layouts a professional feel. Avoid emojis

---

## Section 5ter: Text Utilities (Ui.Prose)

The `Ui.Prose` module provides typography utilities for proper text handling beyond basic `Ui.text`.

### Available Utilities

```elm
import Ui.Prose

-- Non-breaking space: prevents line breaks between words
-- Useful for keeping icon + label together, or "Dr. Smith" on one line
Ui.Prose.noBreak  -- "\u{00A0}"

-- Soft hyphen: suggests break point with hyphen if needed
-- Useful for long compound words that may need to wrap
Ui.Prose.softHyphen  -- "\u{00AD}"

-- Typography dashes
Ui.Prose.enDash  -- "–" for ranges (1–10)
Ui.Prose.emDash  -- "—" for breaks in thought

-- Smart quotes
Ui.Prose.quote "text"  -- wraps in proper curly quotes "text"

-- Paragraph element for prose text
Ui.Prose.paragraph [ Font.size 16 ] [ ... ]

-- Column for multiple paragraphs
Ui.Prose.column [ Ui.spacing 16 ] [ para1, para2 ]
```

### When to Use

**Non-breaking space (`noBreak`):**
```elm
-- Keep "5 items" from breaking across lines
Ui.text ("5" ++ Ui.Prose.noBreak ++ "items")

-- Keep icon + label together in cramped layouts
Ui.row []
    [ icon
    , Ui.text (Ui.Prose.noBreak ++ "Label")  -- won't orphan icon
    ]
```

**Soft hyphen (`softHyphen`):**
```elm
-- Allow long words to break gracefully
Ui.text ("super" ++ Ui.Prose.softHyphen ++ "cali" ++ Ui.Prose.softHyphen ++ "fragilistic")
```

**Paragraphs for prose:**
```elm
-- Use paragraph for body text that should wrap naturally
Ui.Prose.paragraph [ Font.size 14, Font.lineHeight 1.6 ]
    [ Ui.text "This is a long paragraph of text that will wrap "
    , Ui.text "naturally at the container boundaries with proper "
    , Ui.el [ Font.bold ] (Ui.text "inline formatting")
    , Ui.text " preserved."
    ]
```

### Text Truncation Pattern

For labels that must fit in constrained space (buttons, tags), truncate with ellipsis:

```elm
truncateLabel : Int -> String -> String
truncateLabel maxLen str =
    if String.length str <= maxLen then
        str
    else
        String.left (maxLen - 1) str ++ "…"

-- Usage
Ui.text (truncateLabel 18 buttonLabel)
```

---

## Section 6: Gestalt Principles in Elm UI


Read the org-mode files in ~.~, especially [[./alignment.org]]

Gestalt principles describe how humans perceive visual relationships. Elm UI's layout system maps directly to these principles through `fill`, `shrink`, and alignment attributes.

### Proximity (Elements that belong together should be grouped)

**Problem:** Items spread apart when they should be grouped.

```elm
-- WRONG: Row fills width, pushing items apart
Ui.row [ Ui.width Ui.fill ]
    [ icon, label ]  -- icon on left, label on far right

-- RIGHT: Row shrinks to content, keeping items together
Ui.row [ Ui.width Ui.shrink ]
    [ icon, label ]  -- icon and label stay together

-- RIGHT: alignLeft prevents row from filling parent width
Ui.row [ Ui.alignLeft, Ui.spacing Spacing.xs ]
    [ icon, label ]  -- items grouped on left side
```

**When to use:**
- Icon + label pairs: `Ui.row [ Ui.alignLeft, Ui.spacing Spacing.xs ]`
- Stat pills: `Frame.other (Ui.width Ui.shrink)` on the container
- Badge groups: `Ui.row [ Ui.wrap, Ui.alignLeft ]`

### Figure/Ground (Container vs Content sizing)

Understanding when containers should **shrink** (hug content) vs **fill** (expand to space):

```elm
-- Container FILLS, content SHRINKS (common card pattern)
Frame.view [ Frame.width Ui.fill ]  -- card fills grid cell
    (Ui.column [ Ui.width Ui.fill ]  -- content column fills card
        [ Ui.el [ Ui.alignLeft ] headerContent    -- header hugs left
        , Ui.el [ Ui.width Ui.fill ] bodyContent  -- body fills
        , Ui.row [ Ui.alignLeft ] footerButtons   -- buttons hug left
        ]
    )

-- Container SHRINKS (inline element pattern)
Ui.el [ Ui.width Ui.shrink ]  -- badge shrinks to fit
    (Ui.row [ Ui.spacing Spacing.xs ]
        [ statusDot, statusLabel ]
    )
```

### Alignment in Parent Context

The `align*` attributes position an element within its parent's available space:

```elm
-- Parent is fill-width column
Ui.column [ Ui.width Ui.fill, Ui.spacing Spacing.sm ]
    [ -- Each row can align differently within the column
      Ui.row [ Ui.alignLeft ] [ icon, "Left-aligned content" ]
    , Ui.row [ Ui.alignRight ] [ "Right-aligned content", icon ]
    , Ui.row [ Ui.centerX ] [ "Centered content" ]
    ]

-- Status badge in top-right of card
Ui.el [ Ui.alignRight ] (viewStatusBadge status)

-- Cards aligned to top of grid row
Ui.row [ Ui.wrap, Ui.alignTop, Ui.spacing Spacing.md ]
    (List.map viewCard items)
```

### Common Layout Patterns

**Stats bar with compact pills:**
```elm
viewStatPill icon label value =
    Frame.view
        [ Frame.paddingXY Spacing.lg Spacing.sm
        , Frame.other (Ui.width Ui.shrink)  -- Pill shrinks to content
        ]
        (Ui.row [ Ui.spacing Spacing.sm, Ui.centerY ]
            [ icon
            , Ui.column [ Ui.spacing 2 ]
                [ Ui.el [ Font.bold, Ui.centerX ] (Ui.text value)
                , Ui.el [ Font.size 12, Ui.centerX ] (Ui.text label)
                ]
            ]
        )
```

**Card with properly grouped metadata:**
```elm
viewCardContent =
    Ui.column [ Ui.spacing Spacing.sm, Ui.width Ui.fill ]
        [ Ui.el [ Ui.alignRight ] statusBadge
        , Ui.el [ Font.bold ] (Ui.text title)
        , Ui.el [ Font.size 14 ] (Ui.text subtitle)
        -- Icon + text pairs: alignLeft keeps them grouped
        , Ui.row [ Ui.alignLeft, Ui.spacing Spacing.xs ]
            [ PhosphorIcon.checkCircle [ PhosphorIcon.sm ]
            , Ui.text verifiedName
            ]
        , Ui.row [ Ui.alignLeft, Ui.spacing Spacing.xs ]
            [ PhosphorIcon.icon "robot" [ PhosphorIcon.sm ]
            , Ui.text botName
            ]
        ]
```

### Consistency (Uniform Appearance)

When optional content causes layout shifts, use invisible placeholders:

```elm
-- WRONG: Ui.none causes cards to have different heights
case maybeVerifiedName of
    Just name ->
        viewVerifiedRow name

    Nothing ->
        Ui.none  -- No space reserved, card height varies

-- RIGHT: Invisible placeholder maintains consistent height
let
    ( rowOpacity, displayText ) =
        case maybeVerifiedName of
            Just name -> ( 1.0, name )
            Nothing -> ( 0.0, "placeholder" )  -- Same height, invisible
in
Ui.row
    [ Ui.spacing Spacing.xs
    , Ui.alignLeft
    , Ui.opacity rowOpacity  -- 0.0 = invisible but takes space
    ]
    [ icon, Ui.text displayText ]
```

**When to use:**
- Cards in a grid where some have optional rows
- Lists where items should align vertically
- Any repeated elements that need visual consistency

### Quick Reference

| Goal | Attribute | Effect |
|------|-----------|--------|
| Keep items grouped | `Ui.alignLeft` on row | Row doesn't stretch to fill |
| Shrink container | `Ui.width Ui.shrink` | Container hugs content |
| Fill available space | `Ui.width Ui.fill` | Expands to parent |
| Top-align in grid | `Ui.alignTop` on items | Cards don't stretch vertically |
| Right-align element | `Ui.alignRight` | Pushes to right edge of parent |
| Center element | `Ui.centerX` / `Ui.centerY` | Centers in available space |
| Invisible placeholder | `Ui.opacity 0` | Takes space but not visible |

### Critical Layout Mistakes to Avoid

**Mistake 1: Putting multiple visual rows in a single `Ui.row`**

When you need elements on separate lines, use `Ui.column`:

```elm
-- WRONG: Everything on one line, spread across full width
Ui.row [ Ui.width Ui.fill ]
    [ switchControl, addButton, icon, title, badge ]

-- RIGHT: Two rows stacked vertically
Ui.column [ Ui.spacing Spacing.sm ]
    [ -- Row 1: Switch + Add button
      Ui.row [ Ui.spacing Spacing.sm ]
        [ switchControl, addButton ]
    -- Row 2: Icon + Title + Badge
    , Ui.row [ Ui.spacing Spacing.sm ]
        [ icon, title, badge ]
    ]
```

**Mistake 2: Using `Ui.width Ui.fill` on rows that should shrink**

When a row contains elements that should stay grouped together, DON'T use `width fill`:

```elm
-- WRONG: Row fills parent, elements spread to edges
Ui.row [ Ui.width Ui.fill, Ui.spacing Spacing.md ]
    [ leftGroup, Ui.el [ Ui.width Ui.fill ] Ui.none, rightGroup ]

-- RIGHT: Row shrinks to content, elements stay together
Ui.row [ Ui.spacing Spacing.md ]  -- No width attribute = shrink by default
    [ element1, element2, element3 ]
```

**Mistake 3: Using spacer elements to push things apart**

If you find yourself adding `Ui.el [ Ui.width Ui.fill ] Ui.none` as a spacer, you're probably structuring the layout wrong:

```elm
-- WRONG: Spacer hack to push elements apart
Ui.row [ Ui.width Ui.fill ]
    [ leftContent
    , Ui.el [ Ui.width Ui.fill ] Ui.none  -- Spacer hack
    , rightContent
    ]

-- RIGHT: Use alignRight for the right content
Ui.row [ Ui.width Ui.fill ]
    [ leftContent
    , Ui.el [ Ui.alignRight ] rightContent
    ]

-- OR: Separate rows if they're conceptually different
Ui.column []
    [ Ui.row [] leftContent
    , Ui.row [ Ui.alignRight ] rightContent
    ]
```

**Key Principle:** Think about what elements belong together conceptually. Group them in the same row/column. Don't try to put unrelated elements in the same container just because they appear on the same visual line.

**Mistake 4: Forgetting `alignLeft` on nested containers**

When you have a column containing rows, BOTH the column AND each row need `alignLeft` to prevent spreading:

```elm
-- WRONG: Only outer column has alignLeft, rows still spread
Ui.column [ Ui.alignLeft, Ui.spacing Spacing.sm ]
    [ Ui.row [ Ui.spacing Spacing.sm ]  -- Row will fill parent width!
        [ element1, element2 ]
    , Ui.row [ Ui.spacing Spacing.sm ]  -- This row will spread too
        [ element3, element4 ]
    ]

-- RIGHT: Both column AND rows have alignLeft
Ui.column [ Ui.alignLeft, Ui.spacing Spacing.sm ]
    [ Ui.row [ Ui.alignLeft, Ui.spacing Spacing.sm ]  -- Row hugs content
        [ element1, element2 ]
    , Ui.row [ Ui.alignLeft, Ui.spacing Spacing.sm ]  -- This row too
        [ element3, element4 ]
    ]
```

**Key insight:** `alignLeft` doesn't propagate to children. Each container that should hug its content needs its own `alignLeft`. This is especially important for:
- Tab groups / switch controls
- Icon + label pairs inside columns
- Badge groups inside card headers
- Any nested row inside a column that shouldn't fill width

---

## Section 7: Anti-Patterns to Avoid

### Generic AI Aesthetics

- Inter, Roboto, Arial, system fonts (use distinctive typography)
- Purple gradients on white backgrounds (commit to a real palette)
- Predictable symmetric layouts (use asymmetry, overlap, tension)
- Even spacing everywhere (vary density intentionally)

### Elm UI v2 Specific

- Using `Ui.htmlAttribute` for visual effects (use native helpers)
- Magic numbers for spacing/colors/sizes (use design system values)
- Forgetting `Ui.heightMin 0` on scrollable containers (causes overflow bugs)
- Creating elements without considering container fill/shrink behavior
- Mixing CSS margins with Elm UI spacing system

### Correct vs Incorrect Patterns

```elm
-- WRONG: Magic numbers, raw CSS
Ui.el
    [ Ui.htmlAttribute (Html.Attributes.style "box-shadow" "0 4px 12px rgba(0,0,0,0.1)")
    , Ui.spacing 24
    , Ui.background (Ui.rgb 245 245 245)
    ]

-- RIGHT: Design system values, native helpers
Ui.el
    [ Shadow.shadows shadows.card
    , Ui.spacing spacing.md
    , Ui.background colors.surface
    ]
```

---

**Remember:** Claude is capable of extraordinary creative work. Don't hold back—show what can truly be created when thinking outside the box and committing fully to a distinctive vision. Design aesthetics come first; Elm UI v2's native capabilities make those aesthetics actionable. Commit fully to a distinctive vision and execute it with precision.
