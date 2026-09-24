# Prompt for coding agent: Harpoon-style quick-jump + CodeSnap-style export

Paste this whole file to the agent. Two features, both drawn from a well-known
Neovim note-taking workflow (`harpoon2` + `codesnap.nvim`), adapted to Mindforge.
Inline markdown rendering (the third plugin in that reference, `render-markdown.nvim`)
already exists in Mindforge as `:live` mode — nothing to add there.

## Feature 1: pinned quick-jump between notes ("Harpoon")

**The idea:** instead of hunting through the sidebar, pin your current 3-6
active notes to numbered slots, then jump to any of them with one keystroke —
no menu, no search, instant. This is the single most-loved part of the Harpoon
workflow: it turns "where was that note" into a reflex.

### Data model (add to `core`)

```rust
pub struct PinnedNote {
    pub slot: u8,       // 1-based, display order
    pub note_id: i64,
}
```
New table: `pinned_notes(slot INTEGER PRIMARY KEY, note_id INTEGER NOT NULL)`.
Small and simple on purpose — this isn't a general bookmarking system, it's a
fixed-size quick-access list (cap at 6 slots, matching Harpoon's usual feel;
more than that and it stops being "instant recall" and becomes another list to
search).

### Commands

- `:pin` — pins the currently open note to the next empty slot (1-6). If all
  slots are full, replace the oldest pin (slot 1) and shift the rest down, or
  show a status message telling the user to unpin one first — either is fine,
  pick whichever is less surprising given how the rest of this app handles
  "list is full" situations elsewhere.
- `:unpin` — removes the current note from its slot if pinned.
- `Ctrl+1` through `Ctrl+6` — jump directly to the note in that slot (open it
  in the editor, same as clicking it in the sidebar). This is the actual payoff
  — should feel instant, no confirmation, no animation delay.
- `:pins` — opens a small floating list (same visual language as the rest of
  the app: painter-drawn, monospace, theme-colored) showing all 6 slots, empty
  ones shown as `[empty]`, each row showing slot number + note title. Arrow
  keys to move selection, Enter to jump, `x` to unpin the selected slot,
  `Ctrl+Up`/`Ctrl+Down` (or `J`/`K`) to reorder — this mirrors Harpoon's own
  floating quick-menu, which is core to why the workflow feels good: you can
  glance at your 6 pins and reorganize them without leaving the keyboard.

### Sketch

```rust
// core: persistence
impl Database {
    pub fn set_pin(&self, slot: u8, note_id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO pinned_notes (slot, note_id) VALUES (?1, ?2)
             ON CONFLICT(slot) DO UPDATE SET note_id = excluded.note_id",
            rusqlite::params![slot, note_id],
        )?;
        Ok(())
    }

    pub fn unset_pin(&self, slot: u8) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM pinned_notes WHERE slot = ?1", [slot])?;
        Ok(())
    }

    pub fn list_pins(&self) -> rusqlite::Result<Vec<PinnedNote>> {
        let mut stmt = self.conn.prepare("SELECT slot, note_id FROM pinned_notes ORDER BY slot")?;
        let rows = stmt.query_map([], |r| Ok(PinnedNote { slot: r.get(0)?, note_id: r.get(1)? }))?;
        rows.collect()
    }
}
```

```rust
// commands.rs — add alongside the existing arms
"pin" => {
    if let Some(note_id) = app.active_note_id {
        let pins = app.db.as_ref().and_then(|db| db.list_pins().ok()).unwrap_or_default();
        let next_slot = (1..=6u8).find(|s| !pins.iter().any(|p| p.slot == *s));
        match next_slot {
            Some(slot) => {
                if let Some(ref db) = app.db { let _ = db.set_pin(slot, note_id); }
                app.set_status(format!("Pinned to slot {slot}"), now);
            }
            None => app.set_status("All 6 pin slots full — :unpin one first, or use :pins to reorder", now),
        }
    } else {
        app.set_status("No active note to pin", now);
    }
}
"unpin" => {
    if let Some(note_id) = app.active_note_id {
        if let Some(ref db) = app.db {
            if let Ok(pins) = db.list_pins() {
                if let Some(p) = pins.iter().find(|p| p.note_id == note_id) {
                    let _ = db.unset_pin(p.slot);
                    app.set_status(format!("Unpinned from slot {}", p.slot), now);
                }
            }
        }
    }
}
"pins" => {
    app.pins_open = true;
    app.pins_selected = 0;
}
```

```rust
// Ctrl+1..6 handling — wherever global key shortcuts (not the command bar,
// not vim keys — app-level shortcuts) are already read:
if modifiers.ctrl {
    let slot = match key {
        Key::Num1 => Some(1), Key::Num2 => Some(2), Key::Num3 => Some(3),
        Key::Num4 => Some(4), Key::Num5 => Some(5), Key::Num6 => Some(6),
        _ => None,
    };
    if let Some(slot) = slot {
        if let Some(ref db) = app.db {
            if let Ok(pins) = db.list_pins() {
                if let Some(p) = pins.iter().find(|p| p.slot == slot) {
                    app.open_note(p.note_id); // reuse whatever function the sidebar click already calls
                }
            }
        }
    }
}
```

**The floating `:pins` list** (`app/src/pins_view.rs` or similar) should follow
the same painter+`ui.interact`+`animate_bool` pattern as the theme tab and
accent picker built earlier in this project — a small centered card, not a
separate window. Not written out in full here since it's a straightforward
list-with-selection UI matching patterns already established elsewhere in this
codebase; the agent should build it consistently with those, not introduce a
new visual style for it.

## Feature 2: polished image export ("CodeSnap")

**The idea:** select some text (or export the whole active note), get back a
nicely styled PNG — rounded corners, padding, the app's own syntax/markdown
rendering, maybe a subtle drop shadow — suitable for sharing on social media or
in a chat, the way `codesnap.nvim`/Carbon/Ray.so do for code.

### Approach: render through the same pipeline the app already has, not a new one

Mindforge already renders styled text via `LayoutJob` (from the markdown work)
onto an `egui::Painter`. The export should reuse that exact rendering path —
build the same `LayoutJob`s for the selected text, then paint them into an
**offscreen** render target sized to fit the content, instead of the visible
window, and save that as a PNG. This guarantees the exported image always looks
exactly like what's on screen (same fonts, same theme colors, same markdown
styling) with zero duplicated rendering logic — the single most important
design decision here, since a separate hand-built "export renderer" would
inevitably drift out of sync with the real editor's look over time.

```rust
// app/src/snap.rs — sketch; adapt exact egui/eframe offscreen-render API to
// whatever version this project is pinned to (egui's offscreen rendering
// approach has changed across versions — confirm current API before writing
// this for real, similar caution as with egui_term earlier in this project).

use eframe::egui;
use image::{ImageBuffer, Rgba};

pub struct SnapOptions {
    pub padding: f32,        // outer padding around content, e.g. 48.0
    pub corner_radius: f32,  // e.g. 12.0
    pub max_width: f32,      // wrap width for the content, e.g. 720.0
    pub watermark: bool,     // small "Made with Mindforge" footer, off by default
}

impl Default for SnapOptions {
    fn default() -> Self {
        Self { padding: 48.0, corner_radius: 12.0, max_width: 720.0, watermark: false }
    }
}

/// Renders `text` (already-selected content, or the full active note) using
/// the app's normal markdown LayoutJob pipeline, onto an offscreen surface,
/// and returns PNG bytes ready to write to disk.
pub fn render_snap(
    text: &str,
    theme: &crate::theme::Theme,
    opts: &SnapOptions,
) -> anyhow::Result<Vec<u8>> {
    // 1. Reuse the existing markdown parse + layout functions (from the
    //    markdown module built earlier) to get per-line LayoutJobs, exactly
    //    as the preview pane does — do not reimplement text styling here.
    let doc = markdown::parse_document(text);

    // 2. Measure total content size by laying out each line against
    //    `opts.max_width`, summing line heights — same measurement egui
    //    already does when painting the preview pane, just captured instead
    //    of drawn to the visible window.

    // 3. Create an offscreen render target sized to (content width/height +
    //    2*padding). Paint: rounded-rect background at theme.bg, then each
    //    line's LayoutJob via a Painter targeting that offscreen surface.

    // 4. Read back the rendered pixels into an `image::ImageBuffer<Rgba<u8>, _>`
    //    and encode as PNG.

    todo!("confirm exact offscreen-render API against the pinned egui/eframe version before implementing")
}
```

```toml
# app/Cargo.toml — if not already present
image = "0.25"
```

### Command

- `:snap` — if there's an active text selection, export just that; otherwise
  export the whole active note. Opens a native save dialog (same `rfd`
  pattern already used by `:export`/`:import`) defaulting to
  `{note-title}-snap.png`.
- `:snap --watermark` — includes the small "Made with Mindforge" footer;
  off by default so exported images aren't unexpectedly branded.

```rust
// commands.rs
"snap" => {
    let text = app.ed.selected_text().unwrap_or_else(|| app.ed.text());
    let watermark = args.trim() == "--watermark";
    let opts = snap::SnapOptions { watermark, ..Default::default() };
    match snap::render_snap(&text, &app.theme, &opts) {
        Ok(png_bytes) => {
            let default_name = format!("{}-snap.png", app.active_note_title.replace(' ', "-"));
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("PNG Image", &["png"])
                .save_file()
            {
                match std::fs::write(&path, png_bytes) {
                    Ok(_) => app.set_status(format!("Saved snap: {}", path.display()), now),
                    Err(e) => app.set_status(format!("Snap save failed: {e}"), now),
                }
            }
        }
        Err(e) => app.set_status(format!("Snap render failed: {e}"), now),
    }
}
```

**This one has a genuine open question flagged as `todo!()`, not glossed
over:** the exact offscreen-rendering API differs across egui/eframe versions
and setups (render-to-texture vs. a headless `egui::Context` run, vs. using
`wgpu`/`glow` directly to capture a framebuffer). Confirm which approach fits
this project's actual rendering backend before writing the real
implementation — this is the one place in this feature where guessing wrong
would cost real rework, so it's called out explicitly rather than papered over
with invented API calls.

## Build order

1. Feature 1 data model + `:pin`/`:unpin`/`Ctrl+1..6` — get the core jump
   working end to end before building the floating list UI.
2. `:pins` floating list (view + reorder + unpin from the list).
3. Feature 2: confirm the offscreen-render approach against the real egui
   version (the flagged unknown above) before writing anything else.
4. `render_snap` using the confirmed approach, reusing the markdown layout
   pipeline.
5. `:snap` command + save dialog.

## Definition of done

- Pinning 6 notes and jumping between them with `Ctrl+1`..`Ctrl+6` feels
  instant — no visible delay, no confirmation dialog.
- `:pins` shows all 6 slots (empty ones marked), supports reorder and unpin.
- Pins persist across restart.
- `:snap` produces a PNG that visually matches what's on screen — same fonts,
  same theme colors, same markdown styling — because it reuses the real
  rendering pipeline rather than a separate one.
- Exporting a selection vs. the whole note both work correctly.

## Rules for the agent

- Feature 2's offscreen-rendering approach must be confirmed against the real
  egui/eframe version before implementation — don't guess at the API the way
  the `todo!()` above flags.
- Both features should visually match the app's existing UI language (painter,
  `ui.interact`, `animate_bool`) — don't introduce a new widget style for
  either.
- After each build step, give a 3-line summary of what changed and what to try.
