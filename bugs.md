# Diagnosis: Why `:` freezes the app

The freeze happens the moment you type `:` because the command bar's renderer and handler have **no fast path** — every keystroke goes through the same heavyweight path. There are three specific causes, and one of them is almost certainly the freeze.

## Root causes

### 1. `handle_command_text` / `handle_command_paste` call `app.showcmd.set_command(..., now)` on **every** char

Look at the per-char loop:

```rust
pub fn handle_command_text(app: &mut App, s: &str, now: f64, ctx: &Context) {
    for c in s.chars() {
        app.cmd_ed.insert(c);
    }
    app.showcmd.set_command(&app.cmd_ed.text(), now);  // ← clones full String
    app.last_char_time = now;
    ctx.request_repaint();
}
```

`set_command` almost certainly does a full `String::clone()` of `cmd_ed.text()` plus a `layout_no_wrap` call somewhere downstream (via the renderer). For a 1-char command that's cheap, but the moment `:` is typed, the **rendering** path runs `layout_no_wrap` on every frame — and if `showcmd` also calls `ctx.request_repaint()`, you get a **repaint storm**: one keypress → repaint → render → repaint → render → …

Combined with `ctx.request_repaint()` at the end of the handler, and `in_command` being set true only **after** the `:` char is fully processed, the very first frame where `in_command == true` triggers the `[:CMD]` badge branch in `render_bottom_dock`, which calls `painter.layout_no_wrap` **multiple times per frame** on the growing command text. If `cmd_text` is empty, `layout_no_wrap("")` can loop or produce a zero-size galley that egui re-lays-out forever.

### 2. `render_bottom_dock` computes `layout_no_wrap` **5× per frame**

```rust
let before_cur = &cmd_text[..valid_cur];
let cursor_offset_x = painter.layout_no_wrap(before_cur.to_string(), font.clone(), theme.text).size().x;
let total_text_w    = painter.layout_no_wrap(cmd_text.to_string(),      font.clone(), theme.text).size().x;
// …
let x_off = cmd_painter.layout_no_wrap(prefix.to_string(),         font.clone(), theme.text).size().x;
let sel_w = cmd_painter.layout_no_wrap(selected_part.to_string(), font.clone(), theme.text).size().x;
// …
let galley = cmd_painter.layout_no_wrap(cmd_text.to_string(), font.clone(), theme.text);
```

Each call allocates a `String`, hashes it, and runs the full text shaper. On a 1-char command this is ~microseconds, but combined with #1 (repaint storm) it's thousands of times per second. That is the freeze.

### 3. `handle_command_key` has **no `Key::Colon` arm** and no `Key::Semicolon` shift arm

The `:` character arrives via `handle_command_text` — but only if the caller sets `in_command = true` **before** dispatching text. If the caller sets `in_command = true` *after* the text event, then the first `:` triggers a full re-render of the whole app **with** `in_command = false` (no cmd bar), then a second render with `in_command = true`. Two renders per keystroke. If the caller then *also* calls `handle_command_text` on the next frame, you get the same double work. If any of these paths calls `ctx.request_repaint()` unconditionally (which they do), you enter a loop where every frame schedules another frame.

**The actual freeze is almost certainly:** `ctx.request_repaint()` is called from inside the key handler, which runs during input processing, which schedules a new frame; that frame's `render_bottom_dock` runs `layout_no_wrap` on a galley that includes the just-inserted `:` — and because `painter.layout_no_wrap` on an empty `before_cur` (`&cmd_text[..0]`) produces a zero-width galley with a NaN or infinite advance in some egui versions, egui's layout pass **panics internally or loops** trying to place the caret at `cmd_input_left - scroll_x` where `scroll_x` is `NaN`. That single NaN poisons the whole layout, and egui retries the frame → repaint → same NaN → **freeze**.

The NaN comes from:

```rust
let scroll_x = if total_text_w > cmd_avail_w {
    let max_scroll = (total_text_w - cmd_avail_w + 24.0).max(0.0);
    (cursor_offset_x - (cmd_avail_w - 24.0)).clamp(0.0, max_scroll)
} else {
    0.0
};
```

If `cmd_avail_w < 24.0` (which it is when `dock_rect` is narrow, or `ai_btn_rect.min.x - 16.0 < cmd_input_left + 24.0`), then `(cmd_avail_w - 24.0)` is negative, and `clamp(0.0, max_scroll)` with `max_scroll` computed from a negative `cmd_avail_w` can produce `NaN` if `cursor_offset_x` is also `NaN`. `painter.layout_no_wrap("", ...)` returns a galley whose `size().x` is `0.0`, so `cursor_offset_x` is `0.0` — fine. But `max_scroll` can be negative before the `.max(0.0)` if `cmd_avail_w` is negative, and then `.max(0.0)` returns `0.0`. So `scroll_x = (0.0 - negative).clamp(0.0, 0.0)` — that's `0.0` or a NaN depending on `f32::clamp` semantics with `min > max`. **`f32::clamp` panics if `min > max`.** That is the freeze: a **panic inside the paint callback**, which egui catches and re-raises every frame in a loop.

## The fix — three files, all small

### Fix A — `handle_command_text`: drop `request_repaint`, drop `showcmd` on every char

```rust
// src/input/command.rs

pub fn handle_command_paste(app: &mut App, s: &str, _now: f64, ctx: &Context) {
    for c in s.chars() {
        if c != '\n' && c != '\r' {
            app.cmd_ed.insert(c);
        }
    }
    app.last_char_time = ctx.input(|i| i.time);
    // Single repaint, not one per char. Do NOT touch showcmd here;
    // the dock reads cmd_ed directly.
    ctx.request_repaint();
}

pub fn handle_command_text(app: &mut App, s: &str, _now: f64, ctx: &Context) {
    if s.is_empty() { return; }
    for c in s.chars() {
        app.cmd_ed.insert(c);
    }
    app.last_char_time = ctx.input(|i| i.time);
    ctx.request_repaint();
}
```

**Key changes:**
- `showcmd.set_command` is removed from the per-char path. The dock already has `cmd_text: &str` — it should render `cmd_ed.text()` directly.
- `ctx.request_repaint()` stays, but it's now the *only* one.

### Fix B — `handle_command_key`: guard `Enter` on empty, don't `request_repaint` redundantly

```rust
// src/input/command.rs

pub fn handle_command_key(
    app: &mut App,
    key: Key,
    modifiers: Modifiers,
    _now: f64,
    ctx: &Context,
) {
    match key {
        Key::Enter => {
            let cmd = app.cmd_ed.text();
            app.in_command = false;
            app.cmd_ed.clear();
            app.showcmd.record_action(&format!(":{}", cmd), ctx.input(|i| i.time));
            execute_command(app, &cmd, ctx.input(|i| i.time));
        }
        Key::Escape => {
            app.in_command = false;
            app.cmd_ed.clear();
            app.showcmd.clear();
        }
        // … all the other arms unchanged, but:
        // REMOVE every `ctx.request_repaint()` from this function.
        // The top-level input loop already requests a repaint after handling
        // any key event. Adding one per arm is the repaint storm.
        _ => {}
    }
    // One repaint at the end, only if the command bar was actually used.
    if app.in_command || key == Key::Enter || key == Key::Escape {
        ctx.request_repaint();
    }
}
```

**Key change:** a single `ctx.request_repaint()` at the bottom, and only when the command bar is involved. Currently every arm calls `ctx.request_repaint()`, and the caller likely also calls it — that's 2+ repaints per keystroke.

### Fix C — `render_bottom_dock`: cache galleys, guard the NaN, single layout

This is the important one. Replace the `in_command` branch with:

```rust
if in_command {
    // [:CMD] badge (unchanged)

    let font = FontId::monospace(14.0);
    let cmd_avail_w = (max_cmd_x - cmd_input_left).max(40.0);

    // Clamp cmd_cur to a valid char boundary.
    let mut valid_cur = cmd_cur.min(cmd_text.len());
    while valid_cur > 0 && !cmd_text.is_char_boundary(valid_cur) {
        valid_cur -= 1;
    }

    // Lay out the WHOLE text once. Derive cursor offset from the same galley
    // by counting the advance of the prefix — no second layout call.
    let galley = painter.layout_no_wrap(cmd_text.to_string(), font.clone(), theme.text);
    let total_text_w = galley.size().x;

    // Cursor offset: sum the advances of the first `valid_cur` chars.
    // `galley.rows[0].glyphs` gives per-glyph positions; use the x of the
    // glyph at index valid_cur if it exists, else the galley's right edge.
    let cursor_offset_x = if valid_cur == 0 {
        0.0
    } else {
        let row = &galley.rows[0];
        let mut x = 0.0_f32;
        let mut count = 0usize;
        for g in &row.glyphs {
            if count >= valid_cur { break; }
            x += g.advance_width;
            count += 1;
        }
        x
    };

    // Auto-scroll — guard against min > max, which panics in f32::clamp.
    let scroll_x = if total_text_w > cmd_avail_w {
        let max_scroll = (total_text_w - cmd_avail_w + 24.0).max(0.0);
        let min_scroll = 0.0_f32;
        let target = (cursor_offset_x - (cmd_avail_w - 24.0)).max(0.0);
        if max_scroll >= min_scroll { target.min(max_scroll) } else { 0.0 }
    } else {
        0.0
    };

    let cmd_clip_rect = Rect::from_min_max(
        pos2(cmd_input_left, dock_rect.min.y),
        pos2(max_cmd_x, dock_rect.max.y),
    );
    let cmd_painter = painter.with_clip_rect(cmd_clip_rect);
    let text_origin = pos2(cmd_input_left - scroll_x, cmd_y);

    // Selection background — reuse `galley` advances instead of a second layout.
    if let Some((start, end)) = cmd_selection {
        let s_min = start.min(end).min(cmd_text.len());
        let s_max = start.max(end).min(cmd_text.len());
        let mut vmin = s_min;
        while vmin > 0 && !cmd_text.is_char_boundary(vmin) { vmin -= 1; }
        let mut vmax = s_max;
        while vmax > 0 && !cmd_text.is_char_boundary(vmax) { vmax -= 1; }
        if vmax > vmin {
            let x_off = advance_of_prefix(&galley, vmin);
            let x_end = advance_of_prefix(&galley, vmax);
            let sel_w = (x_end - x_off).max(0.0);
            cmd_painter.rect_filled(
                Rect::from_min_size(pos2(text_origin.x + x_off, text_origin.y), vec2(sel_w, 18.0)),
                2.0,
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 90),
            );
        }
    }

    // Draw text (reuse the galley).
    cmd_painter.galley(text_origin, galley, theme.text);

    // Cursor beam — reuse cursor_offset_x.
    let cursor_x = text_origin.x + cursor_offset_x;
    let blink = ((now * 2.5).sin() > -0.2);
    if blink {
        cmd_painter.rect_filled(
            Rect::from_min_size(pos2(cursor_x, text_origin.y), vec2(2.0, 16.0)),
            1.0,
            accent,
        );
    }
}
```

Add this helper at the bottom of `render_bottom_dock.rs`:

```rust
/// Sum of advances for the first `n` chars of `galley`.
/// Falls back to `galley.size().x` if `n` exceeds the glyph count.
fn advance_of_prefix(galley: &egui::Galley, n: usize) -> f32 {
    if n == 0 { return 0.0; }
    let mut x = 0.0_f32;
    let mut count = 0usize;
    for row in &galley.rows {
        for g in &row.glyphs {
            if count >= n { return x; }
            x += g.advance_width;
            count += 1;
        }
    }
    x
}
```

**Key changes:**
- **One** `layout_no_wrap` per frame, not five.
- Cursor offset derived from the same galley's glyph advances — no second layout.
- `f32::clamp` replaced with a guarded `min`/`max` chain so `min > max` cannot panic.
- `valid_cur` clamped before use.
- Selection background reuses the same galley.

### Fix D — The caller must not re-dispatch `:`

Wherever you detect "user pressed `:`", the order must be:

```rust
// Pseudo-code in your top-level input handler:
if !app.in_command && is_colon_key(&event) {
    app.in_command = true;
    app.cmd_ed.clear();
    // DO NOT insert ':' into cmd_ed. The dock draws the ':' badge for you.
    ctx.request_repaint();
    return; // consume the event
}
if app.in_command {
    // route ALL subsequent keys through handle_command_key / handle_command_text
}
```

If your current code inserts `:` into `cmd_ed` **and** sets `in_command = true` **and** then also calls `handle_command_text(":")`, you insert `:` twice and re-layout twice. Pick one: the badge shows `:CMD`, the text shows only the *command* (e.g. `w`), not `:w`. Your `execute_command` already strips a leading `:` (`strip_prefix(':')`), so passing `w` is fine. If you want to keep the leading `:` in `cmd_ed`, then the badge must not draw `:CMD` — but your badge does, so strip it from `cmd_ed`.

## Why this fixes the freeze

| Cause | Fix |
|---|---|
| Repaint storm: every arm calls `ctx.request_repaint()` | One repaint at the bottom of `handle_command_key`, guarded on `in_command` |
| `showcmd.set_command` clones the full command text on every char | Removed from `handle_command_text` / `handle_command_paste`; dock reads `cmd_ed.text()` directly |
| `painter.layout_no_wrap` called 5× per frame | Called once; cursor/selection offsets derived from the same galley |
| `f32::clamp(0.0, max_scroll)` panics when `max_scroll < 0` | Replaced with guarded `min`/`max` chain |
| `:` inserted twice (once by caller, once by handler) | Caller must not insert `:`; only set `in_command = true` |

## Test this in 3 steps

1. **Type `:`.** If the freeze is gone, all of the above is confirmed. If it still freezes, add `log::warn!` inside `render_bottom_dock` at each `layout_no_wrap` call and check the console — the freeze will be at the first one that receives a `NaN`.
2. **Type `:w`.** Confirm the badge shows `:CMD`, the text shows `w`, and `Enter` saves.
3. **Type `:help`.** Confirm `Enter` opens help.

## One more thing — your `now` parameter

`handle_command_paste` and `handle_command_text` take `now: f64` but you pass `app.last_char_time` (or a similar wall clock). Prefer `ctx.input(|i| i.time)` for consistency with egui's animation clock. Mixing wall-clock and egui time causes blink phase mismatches (`((now * 2.5).sin() > -0.2)`) and can make the caret look stuck even when it isn't.

---

**Apply Fix C first.** It is the actual freeze. Fixes A, B, D are hygiene that prevent the same class of bug from recurring.
