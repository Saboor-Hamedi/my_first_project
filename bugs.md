# Prompt for coding agent: AI Agent chat panel — alignment, avatars, input polish

Paste this whole file to the agent. Covers the chat panel specifically. Also
carries forward two issues flagged in earlier rounds that are still open and
belong in the same pass since they touch the same panel.

## 1. Message alignment: user messages on the right, assistant on the left

**Current bug:** both "You" and "Assistant" messages render left-aligned, which
reads as a transcript, not a conversation. Standard chat convention (and the
clearer signal) is user messages right-aligned, assistant messages left-aligned
— the eye can tell who's speaking from position alone, before even reading the
role label.

```rust
// Locate the message-rendering loop in the AI Agent panel. Each message
// currently gets the same left-anchored layout regardless of role — split
// this into two layouts based on role.

for msg in &conversation.messages {
    let bubble_max_width = panel_rect.width() * 0.78; // leave room so it doesn't span edge-to-edge
    let bubble_width = measure_content_width(&msg.text, font).min(bubble_max_width);

    let bubble_rect = match msg.role {
        Role::User => {
            // Right-aligned: anchor to the right edge of the panel, avatar sits
            // to the right of the bubble.
            Rect::from_min_size(
                pos2(panel_rect.max.x - pad - bubble_width, cursor_y),
                vec2(bubble_width, bubble_height),
            )
        }
        Role::Assistant => {
            // Left-aligned: anchor to the left edge, avatar sits to the left.
            Rect::from_min_size(
                pos2(panel_rect.min.x + pad, cursor_y),
                vec2(bubble_width, bubble_height),
            )
        }
    };

    // draw bubble + avatar using bubble_rect and msg.role, see section 2 for the avatar
    cursor_y += bubble_height + msg_gap;
}
```

The exact measurement/layout helper names will differ from this sketch —
adapt to however the real chat loop currently computes bubble size and
position, the key change is just: **role determines which edge the bubble
anchors to**, everything else about how a bubble is drawn stays the same.

## 2. Role badges → small round avatars, no border

**Current:** "You" and "Assistant" render as rectangular badges with a filled
background and visible border — reads as a UI chip, not a person/agent
identity marker.

**Change to:** a small filled circle, ~25px diameter, no border, with either
a single-letter/short glyph inside (e.g. "Y" for You, "✦" or "AI" for
Assistant) or an actual avatar image if the app has one for the user. Standard
chat-app treatment (iMessage, Slack, Discord all converge on this for exactly
this reason — a circle reads as "identity," a rectangle reads as "label").

```rust
fn draw_avatar(painter: &egui::Painter, center: Pos2, role: Role, theme: &Theme) {
    let radius = 12.5; // ~25px diameter
    let (fill, glyph, glyph_color) = match role {
        Role::User => (theme.accent, "Y", Color32::BLACK), // or the user's initial if known
        Role::Assistant => (theme.surface(), "\u{2726}", theme.accent), // ✦, matches the AI Agent tab's own icon
    };
    painter.circle_filled(center, radius, fill);
    painter.text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(11.0),
        glyph_color,
    );
    // No stroke/border drawn — filled circle only.
}
```

Position the avatar just outside the bubble on the side that matches the
message's alignment (right of the bubble for User, left of the bubble for
Assistant) so the two together read as one unit, avatar-then-bubble in
reading order for assistant messages, bubble-then-avatar for user messages.

## 3. Textarea placeholder text is oversized

**Current:** "Ask anything about your notes... (Enter to send, Shift+Enter for
newline)" renders at what looks like the same size as body text elsewhere,
making it visually loud for what should be a quiet hint.

**Fix:** drop the placeholder's font size a couple points below the actual
input text size, and use `theme.muted` rather than `theme.text` for its color
(placeholders should read as absent content, not as content) — if it's
currently using the input field's own text color/size for the placeholder
rather than a dedicated smaller/dimmer style, that's the bug to find. Also
worth splitting the hint into two lines or shortening it if it's currently
wrapping awkwardly at the textarea's width — "Ask anything about your notes…"
as the placeholder, with the keyboard-shortcut hint moved to a small persistent
label under the textarea instead of packed into the placeholder text itself,
usually reads cleaner than one long placeholder string.

## 4. Carried forward from earlier rounds — still open, same panel

**Table word-wrap breaking mid-word.** Flagged twice now, not yet fixed:
document names and other table-cell text break mid-word ("Recipro"/"cal",
"Documen"/"t") instead of wrapping at word boundaries. Locate wherever table
cells wrap their content and confirm it's wrapping on whitespace, not just
truncating at a fixed character/pixel width without checking for a word
boundary first.

**Chat input box border is too heavy.** The textarea currently has a thick,
bright accent-colored border, which — combined with the "no background boxes,
color-only for content" direction from earlier design passes — reads as the
loudest single element in the panel. Replace with either a much thinner
(1px) border only on focus, or a subtle background-tint change on focus with
no border at all, consistent with how focus/active states are being simplified
elsewhere in the app.

## Definition of done

- User messages align right, assistant messages align left, avatar-then-bubble
  order correct for each side.
- Role indicator is a ~25px filled circle with no border, not a rectangular
  bordered badge.
- Placeholder text in the chat input is visibly smaller/dimmer than real
  message text, not the same size.
- Table cells wrap at word boundaries, never mid-word.
- Chat input's focus border is thin or replaced with a background-tint change,
  not the current heavy accent outline.

## Rules for the agent

- This is a chat-panel-scoped pass — don't touch the editor/preview panes.
- Reuse existing avatar/circle-drawing helpers if this codebase already has one
  (e.g. from the sidebar or settings panels) rather than writing a new one.
- After the fix, give a 3-line summary of what changed and what to check.
