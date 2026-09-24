# Prompt for coding agent: sleek pass — from "code IDE" to "knowledge hub"

Paste this whole file to the agent. This is a visual-design pass, not a features
pass — no new functionality, no behavior changes, only how existing things look.
The agent will need to locate the actual render code for each area named below
(sidebar, tab bar, preview pane, terminal/sessions panel, search/suggestion
highlighting) since those files weren't shared in this conversation — read them
first, then apply the patterns here.

## Why: what's currently making Mindforge read as "IDE" instead of "knowledge hub"

Mindforge's current look is a competent, VS-Code-style code editor: monospace
type everywhere, hard 1px borders around every pane, a terminal occupying
prominent screen real estate, boxy tab chrome. That's the right look for an
editing *tool*. It's the wrong look for a *knowledge hub* — something meant to
feel like reading and thinking, not building and compiling. Specifically, these
choices work against that:

- **Monospace type in prose panes.** Code editors use monospace because code has
  meaningful alignment. Prose doesn't — monospace in the preview pane makes
  reading notes feel like reading a diff.
- **A terminal is visually load-bearing.** Its presence, fixed height, and full
  border treatment give it the same visual weight as the content itself, even
  though for a "knowledge hub" use case it's a secondary, occasional tool.
- **Hard borders on every pane** turn the window into a grid of boxes. The eye
  reads "separate applications glued together" rather than "one considered
  surface."
- **Two competing accent hues** (violet in the chrome, magenta in preview
  headings) reads as two unfinished design passes rather than one intentional
  palette.
- **Background-fill highlighting on matched/suggested text** (see the dedicated
  section below) makes scanning content feel like reading redacted documents —
  every match/suggestion becomes a colored box competing with the text itself,
  rather than the text quietly standing out on its own.

None of this requires restructuring the app — it's a treatment pass. The fixes
below are ordered by visual impact per unit of effort; do them in this order.

## 1. Proportional type in the preview pane (highest impact)

The **editor** pane can and should stay monospace — that's correct for a
text/code-editing surface and matches the app's CLI-typing identity. The
**preview** pane (the rendered, read-only view) should switch to a proportional
font with more generous line-height and margins, so it reads as a document, not
a code diff. This single change does more for "feels like a knowledge hub" than
anything else in this list.

- Body text: a proportional font (bundle one — Inter, or whatever this app's
  design system already leans toward if it has an opinion elsewhere), not the
  monospace font currently shared with the editor.
- Line-height: increase from the editor's tighter code-appropriate spacing to
  something closer to 1.6-1.7x font size — prose needs more breathing room than
  code.
- Margins: wider side margins in the preview than the editor uses, so
  paragraph line-length stays comfortable (roughly 60-80 characters per line is
  the classic readability target) rather than stretching edge-to-edge.
- Code spans and code blocks *within* the preview should still render
  monospace — that's still correct, since that content genuinely is code.

## 2. One accent color, used consistently everywhere

Audit every place `theme.accent` (or a hardcoded color that should be it) is
used across: tab underline/active-tab indicator, sidebar selection state,
terminal prompt color, cursor, heading colors in the preview pane, link colors.
They should all resolve to the *same* hue family from the active theme — not
independently chosen colors that happen to be in the same general area of the
palette. If preview headings are currently using a separate hardcoded
magenta/pink rather than `theme.accent` or `theme.highlight`, that's the bug to
find and fix.

## 3. Remove background-highlight boxes behind matched/suggested text — text color only

**This is a specific, deliberate rule, not a general aesthetic preference:** for
anything that is *content being read* — search matches, command-bar
suggestions, autocomplete matches, syntax-highlighted spans, matched substrings
in a filtered list — the emphasis must come from **text color alone**, never a
background fill layered behind the glyphs. A colored box behind text turns
every match into a small redaction-looking rectangle competing with the words
themselves; a color change on the text itself is quieter and still perfectly
scannable.

This rule applies to *content*, not to UI chrome. A sidebar item's selected
state, a button's hover state, a card's active border — those are controls
being scanned, not text being read, and can keep a background treatment (see
section 5 for how the sidebar selection specifically should look).

**Find every instance of this pattern and fix it:**

```rust
// BEFORE — background box behind matched/highlighted text.
// Look for this shape anywhere search matches, autocomplete/command
// suggestions, or syntax spans are drawn: a `rect_filled` sized to the text,
// drawn immediately before/after the text itself.
painter.rect_filled(
    match_rect,
    2.0,
    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 60),
);
painter.text(pos, Align2::LEFT_TOP, &matched_text, font, theme.text);
```

```rust
// AFTER — accent color on the text itself carries the emphasis. No background
// layer at all. If a match still needs to be scannable at a glance without
// reading every word (e.g. search-result highlighting in a long document),
// use a thin underline instead of a filled box — still lightweight, doesn't
// compete with the text's own shape.
painter.text(pos, Align2::LEFT_TOP, &matched_text, font, theme.accent);

// Optional, only where a match genuinely needs to be spottable without
// reading (e.g. :noh search highlighting across a long scroll):
painter.line_segment(
    [pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)],
    Stroke::new(1.0, theme.accent),
);
```

**Specific places to check** (locate the actual render code for each — these
weren't shared in this conversation):
- Search-match highlighting (wherever `:noh`/search results are drawn).
- The command-bar suggestion list, if it currently fills a background behind
  the matched prefix of each suggested command name.
- Any inline markdown match/emphasis rendering that uses a filled rect rather
  than just a text color (bold/code spans should already be text-color-only per
  the earlier markdown work — confirm they are, since that work predates this
  design pass and may not have had this rule stated explicitly).
- Task-checkbox rows, if the checked state currently draws a background tint
  behind the row rather than just coloring the checkbox glyph itself.

## 4. Drop hard borders between panes — use background-value shift instead

Replace visible 1px borders between sidebar/editor/preview/terminal with a
subtle difference in background value (each region a few percent darker or
lighter than its neighbor) instead of a drawn line. The eye reads adjacent
regions as separate from value contrast alone — a border is redundant most of
the time and is what's making the window look like a grid of boxes.

**Keep a visible border only where it does real work**: the pane-resize drag
handles (the divider a user can actually grab and drag) should stay visibly
distinct, since that's the one place a hard line communicates something
functional ("this is draggable") rather than just separating regions.

## 5. Sidebar selection: left-edge accent bar, not a flat fill

Replace the current flat full-rectangle fill on the selected sidebar item with:
a thin (2-3px) accent-colored bar on the left edge of the row, plus a much
lighter background tint than currently used (or none at all, if the left bar
alone reads clearly against the sidebar's own background). This is the pattern
most reading-focused sidebars (Linear, Notion, VS Code's own explorer) converge
on — legible without visually dominating the row.

## 6. Tab bar: reduce competing signals

Currently the active tab is marked by an underline, a text-color change, *and*
an always-visible `×`, all at once. Simplify to:
- Active tab indicated by **either** the underline **or** a background tint —
  pick one, drop the other, so there's a single clear signal instead of three.
- Close icon (`×`) fades in only on hover of that specific tab, not shown at
  rest — reduces visual noise across a full row of tabs when most aren't being
  interacted with.

## 7. Sessions panel: collapse when trivial

The terminal sessions list currently reserves a fixed-width column regardless
of session count. When there's exactly one session, collapse it to a slim strip
(or hide it entirely, showing just the active session's label in the terminal
header) — only expand to a full list once there are 2+ sessions to actually
choose between. Right now it reserves real estate for a feature that isn't
being used in the common case.

## 8. Merge the top bar and tab bar visually

If there's currently a visible seam (a hard color/border break) between the
window's top bar and the tab row directly below it, either remove that seam
(same background value across both) or make it deliberately subtle, so the top
of the window reads as one continuous surface rather than two stacked bands.

## Build order

1. Section 1 (preview typography) — biggest single visual shift, do it first
   and look at the result before touching anything else.
2. Section 3 (remove background highlights) — second-highest impact, and
   likely touches several different files (search, suggestions, syntax
   rendering), so worth doing as its own focused pass.
3. Section 2 (unify accent color) — should be mostly a find-and-replace once
   the offending hardcoded colors are located.
4. Sections 4-8 (borders, sidebar, tabs, sessions, top-bar seam) — smaller,
   independent, can be done in any order or split across multiple sessions.

## Definition of done

- Preview pane reads visibly differently from the editor pane — proportional
  type, more breathing room — while the editor keeps its monospace/CLI identity.
- No content (search matches, suggestions, syntax spans) is drawn with a
  background fill behind it anywhere in the app — text color only.
- Every accent-colored UI element (tabs, sidebar, cursor, terminal prompt,
  preview headings/links) resolves to the same hue.
- Pane boundaries read from background-value contrast, not drawn borders,
  except at actual drag handles.
- Sidebar selection uses the left-edge-bar pattern, not a flat fill.
- Tab bar has one active-tab signal, not three; close icons appear on hover only.

## Rules for the agent

- This is a visual pass only — no behavior, data model, or feature changes.
- Where a pattern described here (e.g. the background-highlight rule) appears
  in code not shown in this conversation, find every instance across the
  codebase, not just the first one — a partial fix that leaves some highlights
  as boxes and others as text-color defeats the point of the rule.
- After each build step, give a 3-line summary of what changed and what to look at.
