# MINDFORGE addendum: line numbers + live markdown preview

Add these two features to the existing `app` crate. Keep typing latency the same as before — neither feature should add per-frame allocations or slow down the hot path in `update()`.

## 1. Line numbers (gutter)

- A narrow column on the left of the text area showing the line number for every row, right-aligned, in a dim gray (about `Color32::from_gray(90)`).
- Current line's number is brighter (use the theme's `text` color) so it's easy to spot where you are.
- Gutter width = widest number's digit count * char width + 16px padding. Recompute only when the line count changes (not every frame).
- Shift `origin.x` for the text and caret drawing by the gutter width. The caret column math in `editor.rs` does not change — only where it's drawn on screen.
- Toggle with `:linenumbers` (or `:ln`), on by default. Save the setting like caret/theme.

```rust
fn draw_gutter(painter: &egui::Painter, origin: Pos2, line_count: usize, cur_row: usize,
               font: FontId, lh: f32, gutter_w: f32) {
    for row in 0..line_count {
        let color = if row == cur_row { Color32::from_gray(220) } else { Color32::from_gray(90) };
        painter.text(
            pos2(origin.x + gutter_w - 8.0, origin.y + row as f32 * lh),
            Align2::RIGHT_TOP,
            format!("{}", row + 1),
            font.clone(),
            color,
        );
    }
}
```

## 2. Live markdown preview

Only meaningful in `:explain` (and any free-text note mode) — cards/journal fields are single-line and don't need it.

- Split the window into two panes with a thin 1px dim-gray divider: left = editor (with gutter), right = rendered preview. Default split 50/50; draggable divider optional (`:split 60` sets left pane to 60%).
- Toggle with `:preview` (or `:pv`). When off, editor uses the full width.
- **Do not re-parse markdown every frame.** Re-render only when the buffer actually changed this frame (you already know this from `typed` in `handle_input`). Cache the parsed output between frames.
- Use the `pulldown-cmark` crate to parse, then draw the result yourself with `egui::Painter` (headings bigger/brighter, `**bold**` bold, `*italic*` italic, `` `code` `` in the accent color on a slightly lighter black box, `- ` bullets indented with a small dot, `> ` quotes dimmed with a left bar). Keep it minimal — this is a glance-preview, not a full renderer.
- Simplest correct approach: walk `pulldown_cmark::Parser` events once per change, produce a small `Vec<PreviewLine>` (text + style), and paint that vec every frame (cheap — no parsing on the paint path).

```toml
# app/Cargo.toml
pulldown-cmark = "0.12"
```

```rust
struct PreviewLine { text: String, bold: bool, italic: bool, heading: Option<u8>, code: bool, quote: bool }

fn render_markdown(src: &str) -> Vec<PreviewLine> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel};
    let mut lines = vec![];
    let mut bold = false; let mut italic = false; let mut heading: Option<u8> = None;
    let mut code = false; let mut quote = false;
    let mut cur = String::new();

    let flush = |cur: &mut String, lines: &mut Vec<PreviewLine>, bold, italic, heading, code, quote| {
        if !cur.is_empty() {
            lines.push(PreviewLine { text: std::mem::take(cur), bold, italic, heading, code, quote });
        }
    };

    for ev in Parser::new(src) {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => heading = Some(level as u8),
            Event::End(TagEnd::Heading(_)) => { flush(&mut cur, &mut lines, bold, italic, heading, code, quote); heading = None; }
            Event::Start(Tag::Strong) => bold = true,
            Event::End(TagEnd::Strong) => bold = false,
            Event::Start(Tag::Emphasis) => italic = true,
            Event::End(TagEnd::Emphasis) => italic = false,
            Event::Start(Tag::BlockQuote(_)) => quote = true,
            Event::End(TagEnd::BlockQuote(_)) => quote = false,
            Event::Code(s) => lines.push(PreviewLine { text: s.to_string(), bold, italic, heading, code: true, quote }),
            Event::Text(s) => cur.push_str(&s),
            Event::SoftBreak | Event::HardBreak =>
                flush(&mut cur, &mut lines, bold, italic, heading, code, quote),
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Item) => {
                flush(&mut cur, &mut lines, bold, italic, heading, code, quote);
                lines.push(PreviewLine { text: String::new(), bold: false, italic: false, heading: None, code: false, quote: false });
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut lines, bold, italic, heading, code, quote);
    lines
}

fn paint_preview(painter: &egui::Painter, origin: Pos2, lines: &[PreviewLine], base_size: f32, accent: Color32, text_color: Color32) {
    let mut y = 0.0;
    for l in lines {
        let size = match l.heading { Some(1) => base_size * 1.6, Some(2) => base_size * 1.3, Some(_) => base_size * 1.1, None => base_size };
        let mut font_id = FontId::monospace(size);
        if l.bold { font_id = FontId::monospace(size); } // egui monospace has no bold variant by default;
        // simplest fix: bundle a bold weight of the same font and switch FontId::new(size, FontFamily::Name("mono-bold".into()))
        let color = if l.code { accent } else if l.quote { Color32::from_gray(140) } else { text_color };
        let x = if l.quote { 16.0 } else { 0.0 };
        painter.text(origin + vec2(x, y), Align2::LEFT_TOP, &l.text, font_id.clone(), color);
        y += size * 1.4;
    }
}
```

Note left in code on purpose: egui's default monospace has no built-in bold/italic weight — for real bold text, bundle a second font file (e.g. `JetBrainsMono-Bold.ttf`) as a named family and switch `FontId::new(size, FontFamily::Name("mono-bold".into()))` when `bold` is true. Simplify for now if that's too fiddly: skip true bold, just make headings brighter/bigger and bold text the accent color instead.

## Definition of done for this addendum
- `:ln` toggles the gutter; current line number is visibly brighter.
- `:pv` toggles the preview pane in `:explain`; preview updates within one frame of a keystroke, never noticeably lags typing.
- Markdown is only re-parsed when the buffer changed, not every frame.
- Both settings persist across restarts like the others.
