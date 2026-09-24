# Prompt for coding agent: fix the "AI Agent" tab button styling

Paste this whole file to the agent. Small, focused fix — applies the same rule
from the earlier sleek design pass ("accent color on text only, never a
background fill behind it") to one specific control that was missed: the
**AI Agent tab button** in the right-panel header (next to "Preview").

## The problem

The AI Agent tab currently renders with a filled background box behind the
`✦ AI Agent` label (and/or icon badge to its left) plus a visible border around
it — this makes it look like a separate highlighted card sitting inside the
header row, rather than a tab that matches the "Preview" tab next to it.

## The fix

Locate the render code for this tab (wherever the right-panel header draws
"Preview" / "AI Agent" as switchable tabs — likely the same function or a
sibling of whatever draws the main editor's tab bar). Change the AI Agent
tab's active/selected state to match how the rest of the app now signals
"this is selected" after the recent design pass:

- **Remove the filled background rect** behind the label and icon entirely.
- **Remove the border/stroke** around the tab.
- **Keep only a text-color change** — the label (and icon, if it's drawn as
  text/glyph rather than a raster image) goes to `theme.accent` when active,
  `theme.text`/`theme.muted` when inactive. No box, no border, just color.
- If this tab needs a way to show "currently selected" beyond text color alone
  (since it sits next to "Preview" and the two need to be visually
  distinguishable at a glance), use the same underline treatment already
  applied to the main tab bar in the earlier design pass — a thin
  accent-colored line under the active tab's label — rather than reintroducing
  a background fill. This keeps one consistent "active tab" signal across the
  whole app instead of this button having its own separate treatment.

```rust
// BEFORE — background box + border behind the tab label/icon
painter.rect(
    tab_rect,
    4.0,
    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 40),
    Stroke::new(1.0, theme.accent),
    egui::StrokeKind::Inside,
);
painter.text(text_pos, Align2::LEFT_CENTER, "✦ AI Agent", font, theme.accent);
```

```rust
// AFTER — text color carries the "active" signal, nothing drawn behind it.
// Optional thin underline only if a distinct active-tab marker is still
// needed alongside "Preview" — matches the main tab bar's existing pattern.
let label_color = if is_active { theme.accent } else { theme.muted };
painter.text(text_pos, Align2::LEFT_CENTER, "✦ AI Agent", font, label_color);

if is_active {
    painter.line_segment(
        [pos2(tab_rect.min.x, tab_rect.max.y), pos2(tab_rect.max.x, tab_rect.max.y)],
        Stroke::new(1.5, theme.accent),
    );
}
```

## Definition of done

- No filled background box behind the AI Agent tab's icon or label, active or
  inactive.
- No border/stroke drawn around the tab.
- Active state reads from text color (plus, if needed, the same thin-underline
  pattern the main tab bar already uses) — nothing else.
- Visually consistent with the "Preview" tab next to it — both should look
  like they belong to the same tab bar, not two different UI styles.

## Rules for the agent

- This is a one-control styling fix — don't touch the chat panel's own
  content styling (the table-wrapping bug and monospace-chat note are separate,
  already-flagged issues, not part of this task).
- If the icon (`✦`) is a raster/SVG asset rather than a text glyph, recolor it
  via tint rather than drawing a background behind it, so the same "text-only"
  rule still applies to it.
- After the fix, give a 3-line summary of what changed and what to check.
