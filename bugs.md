## Prompt for your agent

**Title:** Redesign the Stats view — proper padding, no outer border, larger typography on wide screens

**Context:** MINDFORGE's Stats view (`app/src/view_stats.rs`, `render_stats`) currently renders inside the editor panel with no top or left padding, an unwanted border around the whole view, and typography that is too small on wide displays. The screenshot shows "WRITING STORY & ACTIVITY" flush against the panel's top-left corner with no breathing room, and the metric cards and journal rows feel cramped.

The Stats view should look like a page inside the panel: inset from the panel edges, spacious, and readable on a 1500px-wide window.

### Task 1 — Remove the outer border

- Remove any `rect`, `rect_stroke`, or `rect_filled` call that draws a border or background around the entire Stats view.
- The Stats view shares the editor panel's background — it does not draw its own frame.
- The only borders that remain are inside individual components: the metric cards, the chart container, and the journal row pills. Those stay.

### Task 2 — Add proper padding

The Stats view must be inset from the panel's edges.

- **Top padding:** at least 24px between the panel's top edge and the "WRITING STORY & ACTIVITY" heading.
- **Left padding:** at least 28px between the panel's left edge and the leftmost content.
- **Right padding:** at least 28px between the rightmost content and the panel's right edge.
- **Bottom padding:** at least 32px of scrollable space below the last journal row.
- The section content (cards, chart, journal rows) must not touch the panel edges at any window size.

### Task 3 — Scale typography with available width

Current sizes are fixed and too small on wide screens. Make the type scale with the panel's width, within reasonable clamps.

Add a scale factor based on `avail_w`:

```
let scale = (avail_w / 900.0).clamp(1.0, 1.35);
```

Then multiply the current font sizes by `scale`:

- Header title (currently ~19.0) → `19.0 * scale`
- Header subtitle (~11.5) → `11.5 * scale`
- Section headings ("WRITING RHYTHM", "ACTIVITY JOURNAL") (~11.5–12.5) → `* scale`
- Card label (~9.5) → `9.5 * scale`, minimum 10.0
- Card main value (~17.0) → `17.0 * scale`, minimum 18.0
- Card subtitle (~10.0) → `10.0 * scale`, minimum 10.5
- Journal row text (~11.0) → `11.0 * scale`, minimum 11.5

Card heights, chart heights, and row heights must scale with the same factor so the layout doesn't look sparse. Specifically:

- `card_h` (currently 72.0) → `72.0 * scale`
- `chart_box_h` (currently 136.0) → `136.0 * scale`
- Journal row height (currently 34.0) → `34.0 * scale`

### Task 4 — More breathing room between sections

Increase the vertical gaps between major sections:

- Between header and metric cards: `ui.add_space(8.0)` → `ui.add_space(16.0 * scale)`
- Between metric cards and the chart: `ui.add_space(20.0)` → `ui.add_space(28.0 * scale)`
- Between the chart and "ACTIVITY JOURNAL": `ui.add_space(20.0)` → `ui.add_space(28.0 * scale)`
- Between the journal heading and the first row: `ui.add_space(6.0)` → `ui.add_space(10.0 * scale)`
- Between journal rows: `ui.add_space(4.0)` → `ui.add_space(6.0 * scale)`
- Final bottom space: `ui.add_space(24.0)` → `ui.add_space(36.0 * scale)`

### Task 5 — Journal row padding

The journal rows currently have text right at the row's left edge with only 12px padding. Increase:

- Left text padding: `12.0` → `16.0 * scale`
- Right text padding: `14.0` → `18.0 * scale`
- Row corner radius stays at 4.0
- Row background and border stay the same

### Task 6 — Metric card padding

The metric cards currently start content at `rect.min.x + 12.0`. Increase:

- Left content padding: `12.0` → `16.0 * scale`
- Top label padding: `10.0` → `14.0 * scale`
- Main value offset: `26.0` → `32.0 * scale`
- Subtitle offset: `50.0` → `58.0 * scale`
- Accent stripe width stays at 3.0

The card's internal layout must not overflow — verify by checking that the subtitle at `rect.min.y + 58.0 * scale` still fits inside `card_h = 72.0 * scale`. If it doesn't, increase `card_h` until it does.

### Constraints

- Do not change the data being displayed, only its presentation.
- Do not change the color logic (which card is highlighted, which rows are accent-colored).
- Do not remove the ScrollArea — keep the view scrollable.
- Do not change the chart's bar-drawing logic, only its container height and the section spacing around it.
- Do not change the layout of the sidebar or the editor panel.
- Do not add new sections or remove existing ones.

### Acceptance criteria

- The Stats view has visible padding from the panel's top, left, and right edges. Nothing touches the panel border.
- No border or background frame is drawn around the entire Stats view.
- On a 1500px-wide window, text is noticeably larger than before. On a 900px-wide window, text remains readable at the minimum sizes.
- Metric cards, chart, and journal rows have more vertical spacing between them.
- Journal rows have more left/right padding around the date and the numbers.
- The view still scrolls when content exceeds the panel height.
- No clipping or overflow: card subtitles, journal row content, and chart labels all fit inside their containers.
- All existing tests still pass. `cargo check -p app` and `cargo test -p app` clean.

### Out of scope

- Do not change the theme.
- Do not add new metrics or statistics.
- Do not change the ordering of sections.
- Do not change the sidebar layout or the editor panel layout.
- Do not change the status bar.

Report back with `cargo check -p app` output and one screenshot of the Stats view at the same window size as the original, so I can compare padding and type scale directly.

---
## Prompt for your agent

**Title:** Redesign the Stats view — proper padding, no outer border, larger typography on wide screens

**Context:** MINDFORGE's Stats view (`app/src/view_stats.rs`, `render_stats`) currently renders inside the editor panel with no top or left padding, an unwanted border around the whole view, and typography that is too small on wide displays. The screenshot shows "WRITING STORY & ACTIVITY" flush against the panel's top-left corner with no breathing room, and the metric cards and journal rows feel cramped.

The Stats view should look like a page inside the panel: inset from the panel edges, spacious, and readable on a 1500px-wide window.

### Task 1 — Remove the outer border

- Remove any `rect`, `rect_stroke`, or `rect_filled` call that draws a border or background around the entire Stats view.
- The Stats view shares the editor panel's background — it does not draw its own frame.
- The only borders that remain are inside individual components: the metric cards, the chart container, and the journal row pills. Those stay.

### Task 2 — Add proper padding

The Stats view must be inset from the panel's edges.

- **Top padding:** at least 24px between the panel's top edge and the "WRITING STORY & ACTIVITY" heading.
- **Left padding:** at least 28px between the panel's left edge and the leftmost content.
- **Right padding:** at least 28px between the rightmost content and the panel's right edge.
- **Bottom padding:** at least 32px of scrollable space below the last journal row.
- The section content (cards, chart, journal rows) must not touch the panel edges at any window size.

### Task 3 — Scale typography with available width

Current sizes are fixed and too small on wide screens. Make the type scale with the panel's width, within reasonable clamps.

Add a scale factor based on `avail_w`:

```
let scale = (avail_w / 900.0).clamp(1.0, 1.35);
```

Then multiply the current font sizes by `scale`:

- Header title (currently ~19.0) → `19.0 * scale`
- Header subtitle (~11.5) → `11.5 * scale`
- Section headings ("WRITING RHYTHM", "ACTIVITY JOURNAL") (~11.5–12.5) → `* scale`
- Card label (~9.5) → `9.5 * scale`, minimum 10.0
- Card main value (~17.0) → `17.0 * scale`, minimum 18.0
- Card subtitle (~10.0) → `10.0 * scale`, minimum 10.5
- Journal row text (~11.0) → `11.0 * scale`, minimum 11.5

Card heights, chart heights, and row heights must scale with the same factor so the layout doesn't look sparse. Specifically:

- `card_h` (currently 72.0) → `72.0 * scale`
- `chart_box_h` (currently 136.0) → `136.0 * scale`
- Journal row height (currently 34.0) → `34.0 * scale`

### Task 4 — More breathing room between sections

Increase the vertical gaps between major sections:

- Between header and metric cards: `ui.add_space(8.0)` → `ui.add_space(16.0 * scale)`
- Between metric cards and the chart: `ui.add_space(20.0)` → `ui.add_space(28.0 * scale)`
- Between the chart and "ACTIVITY JOURNAL": `ui.add_space(20.0)` → `ui.add_space(28.0 * scale)`
- Between the journal heading and the first row: `ui.add_space(6.0)` → `ui.add_space(10.0 * scale)`
- Between journal rows: `ui.add_space(4.0)` → `ui.add_space(6.0 * scale)`
- Final bottom space: `ui.add_space(24.0)` → `ui.add_space(36.0 * scale)`

### Task 5 — Journal row padding

The journal rows currently have text right at the row's left edge with only 12px padding. Increase:

- Left text padding: `12.0` → `16.0 * scale`
- Right text padding: `14.0` → `18.0 * scale`
- Row corner radius stays at 4.0
- Row background and border stay the same

### Task 6 — Metric card padding

The metric cards currently start content at `rect.min.x + 12.0`. Increase:

- Left content padding: `12.0` → `16.0 * scale`
- Top label padding: `10.0` → `14.0 * scale`
- Main value offset: `26.0` → `32.0 * scale`
- Subtitle offset: `50.0` → `58.0 * scale`
- Accent stripe width stays at 3.0

The card's internal layout must not overflow — verify by checking that the subtitle at `rect.min.y + 58.0 * scale` still fits inside `card_h = 72.0 * scale`. If it doesn't, increase `card_h` until it does.

### Constraints

- Do not change the data being displayed, only its presentation.
- Do not change the color logic (which card is highlighted, which rows are accent-colored).
- Do not remove the ScrollArea — keep the view scrollable.
- Do not change the chart's bar-drawing logic, only its container height and the section spacing around it.
- Do not change the layout of the sidebar or the editor panel.
- Do not add new sections or remove existing ones.

### Acceptance criteria

- The Stats view has visible padding from the panel's top, left, and right edges. Nothing touches the panel border.
- No border or background frame is drawn around the entire Stats view.
- On a 1500px-wide window, text is noticeably larger than before. On a 900px-wide window, text remains readable at the minimum sizes.
- Metric cards, chart, and journal rows have more vertical spacing between them.
- Journal rows have more left/right padding around the date and the numbers.
- The view still scrolls when content exceeds the panel height.
- No clipping or overflow: card subtitles, journal row content, and chart labels all fit inside their containers.
- All existing tests still pass. `cargo check -p app` and `cargo test -p app` clean.

### Out of scope

- Do not change the theme.
- Do not add new metrics or statistics.
- Do not change the ordering of sections.
- Do not change the sidebar layout or the editor panel layout.
- Do not change the status bar.

Report back with `cargo check -p app` output and one screenshot of the Stats view at the same window size as the original, so I can compare padding and type scale directly.
