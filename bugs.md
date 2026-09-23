# Task: Unify Markdown Editor + Preview into One Robust Renderer

## Context

A screenshot shows the same markdown document rendered side-by-side in an Editor (left) and a Preview (right). They visibly disagree:

| # | Symptom | Root cause |
|---|---|---|
| 1 | Code fence content is proportional font in Preview, monospace in Editor | Preview has its own renderer that doesn't apply `FontId::monospace` |
| 2 | Blank-line vertical gaps differ between Editor and Preview | `line_h` for empty lines is computed in two places with different rules |
| 3 | Table columns misalign; `Created` column wraps in Preview | Tables rendered as a `LayoutJob` per row — `LayoutJob` cannot align columns across rows |
| 4 | `##` prefix still visible on inactive heading line in Editor | Inactive `Heading` arm consumes the prefix index but still appends it |
| 5 | Caret position does not reset when moving off a blockquote line | `char_map.len() != job_glyph_count + 1` — blockquote indent pushes map entries without appending matching glyphs |
| 6 | Task list checkboxes `- [x]` / `- [ ]` never render as ☑ / ☐ | Inactive `TaskItem` arm appends 4 spaces and pushes 4 map entries all pointing to `char_start + 3` |

All six trace to two causes:

- **R1:** Editor and Preview are two independent renderers with no shared contract — they will always drift.
- **R2:** The `char_map` invariant `map.len() == glyphs + 1` is enforced by hand and is violated in several branches.

## Goal

Produce a **single renderer** that both the Editor and the Preview call. Guarantee the `char_map` invariant by construction. Support **all common markdown**. Split into clean files.

## Supported Markdown

### Block-level (per line, then multi-line passes)
- ATX headings `#` … `######`
- Setext headings (`Title` over `===` or `---`)
- Code fences ` ```lang `
- Horizontal rules `---`, `***`, `___` (≥3, spaces allowed)
- Blockquotes `>`, `>>`, `> > ` with depth
- Bullet items `- `, `* `, `+ `
- Numbered items `1. ` … `999. `
- Task items `- [ ] `, `- [x] `, `- [X] `, `* [ ] `, `* [x] `
- Table rows `| a | b |`, with separator `|:---|:---:|---:|`
- Blank lines
- Normal lines

### Inline
- Bold `**x**`, `__x__`
- Italic `*x*`, `_x_`
- Bold italic `***x***`, `___x___`
- Strikethrough `~~x~~`
- Inline code `` `x` ``, ``` ``x`` ```
- Link `[text](url)`, `[text](url "title")`
- Image `![alt](url)`
- Autolink `<https://…>`, `<user@host>`
- Footnote ref `[^id]`
- Escape `\*`, `\_`, `` \` ``, `\[`, `\]`, `\(`, `\)`, `\#`, `\\`
- Hard break (two trailing spaces, or trailing `\`)
- Inline HTML passthrough `<span>`, `<br/>` (muted)

## Architecture

```
src/view_editor/inline/
├── mod.rs
├── types.rs           // RenderMode, InlineLineKind, TableAlign, TableRowInfo, InlineSpan, InlineSpanKind
├── charmap.rs         // CharMapBuilder  — enforces invariant
├── classify.rs        // classify_line, classify_lines (setext + table-header promotion)
├── spans.rs           // parse_inline_spans
├── layout.rs          // build_line_layout  (public entry, per-line)
├── table.rs           // collect_table_block, render_table_block (egui::Grid)
├── elements/
│   ├── mod.rs         // BLANK_LINE_HEIGHT_FACTOR and shared constants
│   ├── heading.rs
│   ├── block_quote.rs
│   ├── list.rs
│   ├── table.rs       // metrics + colors + parse_aligns
│   ├── rule.rs
│   └── code.rs
└── render/
    ├── mod.rs
    ├── active.rs
    └── inactive.rs
```

### `RenderMode`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderMode {
    pub is_active: bool,
    pub hide_all_syntax: bool,
}

impl RenderMode {
    pub const EDITOR_ACTIVE:   Self = Self { is_active: true,  hide_all_syntax: false };
    pub const EDITOR_INACTIVE: Self = Self { is_active: false, hide_all_syntax: false };
    pub const PREVIEW:         Self = Self { is_active: false, hide_all_syntax: true  };
}
```

**Contract:** for the same line, `EDITOR_INACTIVE` and `PREVIEW` must produce output with identical font family, font size, and `line_h` for all *content* spans. The only difference is that `hide_all_syntax == true` omits syntax-marker glyphs entirely instead of drawing them as `Color32::TRANSPARENT`. Preview calls the same `build_line_layout` with `RenderMode::PREVIEW`. Delete any separate Preview parser. One renderer.

### `CharMapBuilder` invariant

```
map.len() == total_glyphs_appended_to_job + 1     // +1 for EOL sentinel
```

`CharMapBuilder` must be the *only* way glyphs and map entries are produced. Rule for the agent:

> **Never call `job.append` directly. Never call `char_map.push` directly.**
> Every append goes through `CharMapBuilder::append_span` or `append_run`.
> `finish()` must `debug_assert!` the invariant.

Hidden syntax markers (`##`, `1.`, `-`, `>`, `|`, `**`, `~~`, backticks, `- [x] `) on inactive / preview lines are **not mapped at all** — no phantom entries. Cursor navigation snaps past them to the nearest visible source index. This is what makes the invariant hold.

```rust
pub struct CharMapBuilder {
    map: Vec<usize>,
    glyphs: usize,
    char_start: usize,
}

impl CharMapBuilder {
    pub fn new(char_start: usize, capacity: usize) -> Self;
    /// Append `text` to `job`; map every glyph to `source_idx`.
    pub fn append_span(&mut self, job: &mut LayoutJob, text: &str, source_idx: usize, fmt: TextFormat);
    /// Append a 1:1 run of source chars in `range`; each glyph maps to `char_start + offset`.
    pub fn append_run(&mut self, job: &mut LayoutJob, chars: &[char], range: std::ops::Range<usize>, fmt: TextFormat);
    /// No-op for readability: explicitly skip a hidden source range.
    pub fn skip_range(&mut self, range: std::ops::Range<usize>);
    /// Push EOL sentinel; debug-assert invariant.
    pub fn finish(mut self, chars_len: usize) -> Vec<usize>;
}
```

### `build_line_layout` signature

```rust
pub fn build_line_layout(
    chars: &[char],
    char_start: usize,
    mode: RenderMode,
    base_font_size: f32,
    theme: &Theme,
    kind: &InlineLineKind,
    in_code_block: bool,
) -> (LayoutJob, Vec<usize>, f32);
```

Same return tuple. `is_active` is gone — replace with `mode`.

## Required Behavior by Line Kind

### Headings (ATX)
- **Active:** render `#…` prefix in `theme.accent` (syntax font, 1:1 map) then rest of line.
- **Inactive / Preview:** consume the `#…` prefix (`i = hash_count + 1`), **do not append it**, do not map it. Rest of line in `heading_color(level, theme)` with `heading_metrics`.
- **Test:** `inactive_heading_hides_hashes` — the joined section text must not contain `#`.

### Setext headings
- Add `InlineLineKind::SetextHeading(u8)` and `SetextUnderline(u8)`.
- `classify_lines` promotes line `i` when line `i+1` is all `=` or all `-` (and line `i` is non-empty and not itself a bullet/rule).
- Inactive render: heading style on the text line; underline line renders with `line_h = 2.0` and `Color32::TRANSPARENT`.

### Task items
- `classify_line` yields `TaskItem { checked, check_char_idx: 3 }` and consumes 6 chars.
- **Active:** render `- [x] ` prefix verbatim in `theme.accent`, 1:1 map. (User edits raw.)
- **Inactive / Preview:**
  - Consume `i = 6`, do **not** append `- [x] `.
  - Append `"☑ "` (checked) in `theme.accent` or `"☐ "` (unchecked) in `theme.muted`, using `append_span` with `source_idx = char_start + 3`.
  - Continue rendering the rest of the line.
- **Test:** inactive render of `- [x] done` must contain `☑` and must not contain `- [x]`.

### Tables — must use `egui::Grid`
- `InlineLineKind::TableRow(TableRowInfo { is_header, is_separator, aligns })`.
- `classify_lines` promotes the row directly above a separator row to `is_header = true` and copies alignment info from the separator into both the separator and the header.
- `parse_aligns` reads each separator cell `:?-+:?`:
  - `:---` → Left, `---:` → Right, `:---:` → Center, `---` → None.
- **Do not render a table row through `build_line_layout`.** Instead, the top-level editor widget:
  1. Scans for runs of consecutive `TableRow` lines via `collect_table_block(lines, kinds, i)`.
  2. If found, calls `render_table_block(ui, &block, base, theme, mode)` which uses `egui::Grid::new(...).striped(true).spacing(...)`.
  3. Skips those source lines in the per-line layout loop.
  4. Only if the caret is on a line inside the table, render that one line via `build_line_layout` (raw pipes, syntax color) so the user can edit.
- `render_table_block` computes per-column max text width and pins a `min_col_width` on each cell so columns align. Returns the pixel height consumed.
- **Test:** render a 3-row table with one long cell; assert all rows have the same number of cells and each column's rendered width is equal.

### Blockquotes
- **Active:** render `> ` prefix (and spaces) in `theme.accent`, 1:1 map.
- **Inactive / Preview:** consume prefix, do not append, do not map. Then append the depth indent **as actual glyphs** (spaces) via `append_span` mapped to `char_start` — this is the only correct way to reserve horizontal space, and it fixes the caret gap.
- Do **not** push map entries without appending matching glyphs. That was the bug.

### Bullet items
- **Active:** `- ` prefix in `theme.accent`, 1:1 map.
- **Inactive / Preview:** consume 2 chars; append `"•  "` via `append_span` with `source_idx = char_start`. Do **not** append the raw `- `.

### Numbered items
- **Active:** `1. ` prefix in `theme.accent`, 1:1 map.
- **Inactive / Preview:** consume `dot_idx + 2` chars; append `format!("{num}. ")` via `append_span` with `source_idx = char_start`. Do **not** append the raw digits+dot.

### Horizontal rules
- **Active:** render raw `---` / `***` / `___` in `theme.accent`, 1:1 map, `line_h = base * 1.4`.
- **Inactive / Preview:** append a single `" "` glyph in `Color32::TRANSPARENT`, map it to `char_start`, `line_h = 2.0`. Do not append the raw rule chars.

### Blank lines
- **Always:** append one `" "` glyph in `Color32::TRANSPARENT`, map to `char_start`. `line_h = base_font_size * BLANK_LINE_HEIGHT_FACTOR` where `BLANK_LINE_HEIGHT_FACTOR = 0.6`. Apply this in *both* editor and preview — that is what collapses the inconsistent gaps in the screenshot.

### Inline (all lines, both modes)
Parse with `spans.rs` → `Vec<InlineSpan>`, then render. Both `render/active.rs` and `render/inactive.rs` iterate the same span list; only `TextFormat` choices differ.

- **Bold:** active shows `**` in `syntax_color` (mapped 1:1), content in `theme.highlight` with bold font. Inactive/preview omits `**`, content in `theme.highlight` bold.
- **Italic:** `italics = true` on `TextFormat`. Active shows `*` markers in `syntax_color`. Inactive/preview omits markers.
- **Bold italic:** combine bold + italics.
- **Strike:** `strikethrough = Stroke::new(1.0, theme.muted)`, content in `theme.muted`. Active shows `~~` in `syntax_color`. Inactive/preview omits `~~`.
- **Inline code:** `FontId::monospace(font_size * 0.92)`, color `theme.accent`. Active shows backticks in `syntax_color`. Inactive/preview omits backticks.
- **Link:** text in `theme.accent` with `underline = Stroke::new(1.0, theme.accent)`. Active additionally shows `](url)` in `syntax_color` (1:1 map). Inactive/preview omits `](url)`.
- **Image:** prefix `"🖼 "` in `theme.muted`, alt text in `theme.muted` italic. Active shows `![alt](url)` markers in `syntax_color`.
- **Autolink:** `theme.accent` underlined. Active shows `<` and `>` in `syntax_color`.
- **Footnote ref:** `[^id]` in `theme.muted` with superscript sizing (`font_size * 0.85`).
- **Escape `\X`:** active shows `\` in `syntax_color`, `X` in `base_text_color`. Inactive/preview omits `\`, shows `X` only.
- **Hard break:** active shows `↩` in `syntax_color` mapped to `char_start + chars.len()`; inactive/preview renders nothing extra (the line break itself is handled by the caller).
- **Inline HTML:** passthrough in `theme.muted`.

**Unmatched markers** (`*`, `` ` ``, `~`, `[` with no closer) render as literal `base_text_color` glyphs, mapped 1:1.

## Files to Produce

Produce **complete, compiling** code — no `todo!()`, no `unimplemented!()`, no stubs. Assume `eframe`, `egui`, and the existing `Theme` API: `theme.text`, `theme.muted`, `theme.accent`, `theme.highlight`, `theme.border()`, `theme.is_light()`.

- `types.rs` — `RenderMode`, `InlineLineKind` (with `SetextHeading`, `SetextUnderline`, `Blank`), `TableAlign`, `TableRowInfo`, `InlineSpan`, `InlineSpanKind`.
- `charmap.rs` — `CharMapBuilder` as above.
- `classify.rs` — `classify_line`, `classify_lines` (setext + table-header promotion).
- `spans.rs` — `parse_inline_spans(chars, base_offset) -> Vec<InlineSpan>`.
- `layout.rs` — `build_line_layout` as above.
- `table.rs` — `TableBlock`, `collect_table_block`, `render_table_block` using `egui::Grid`.
- `elements/mod.rs` — `BLANK_LINE_HEIGHT_FACTOR = 0.6` and shared helpers.
- `elements/heading.rs` — `heading_metrics(level, base)`, `heading_color(level, theme)` for levels 1–6.
- `elements/block_quote.rs` — `quote_color(theme)`, `quote_indent(depth) -> String`.
- `elements/list.rs` — `bullet_glyph() -> "•  "`, `checkbox_glyph(checked) -> "☑ " / "☐ "`, `number_glyph(num) -> String`.
- `elements/table.rs` — `table_metrics(base, is_separator, is_active)`, `pipe_color(theme, is_active)`, `cell_color(theme, is_header)`, `parse_aligns(sep_chars) -> Vec<TableAlign>`.
- `elements/rule.rs` — `rule_metrics(base) -> (FontId, f32)`.
- `elements/code.rs` — `code_metrics(base, is_closing, is_active) -> (FontId, f32)`.
- `render/active.rs` — `render_active(...)`.
- `render/inactive.rs` — `render_inactive(...)` (handles both `EDITOR_INACTIVE` and `PREVIEW` via `mode.hide_all_syntax`).
- `mod.rs` — re-exports.

## Required Tests (in `layout.rs`, `table.rs`, `classify.rs`)

1. `char_map_invariant_holds_for_every_kind` — for each `InlineLineKind`, `map.len() == glyph_count + 1`. Count glyphs by iterating `job.sections` and summing `buffer[s.byte_range].chars().count()`.
2. `caret_round_trip` — for every source index `k` in `0..=chars.len()`, `pos_from_cursor(k)` then `cursor_from_pos(pos) == k` when `k` is in the mapped domain; otherwise the returned index is the nearest mapped neighbor.
3. `inactive_heading_hides_hashes` — as above.
4. `inactive_task_renders_checkbox` — as above.
5. `inactive_quote_does_not_leak_prefix` — inactive render of `> hello` contains `hello`, not `>`.
6. `table_grid_columns_align` — see above.
7. `blank_line_height_matches_preview` — `build_line_layout(blank, …, EDITOR_INACTIVE, …).2 == build_line_layout(blank, …, PREVIEW, …).2`.

## Style

- Rust 2021, `#![deny(clippy::all)]`-clean.
- No `unwrap()` on `Option` / `Result` in library code — use `?` or `unwrap_or`.
- All public items documented with `///`.
- No `unsafe`.
- No direct `job.append` / `char_map.push` anywhere. Enforced by `CharMapBuilder`.

## Output Format

Return one code block per file with the path as a header:

```rust
// src/view_editor/inline/types.rs
...
```

Do not include the original `inline.rs`. Replace it entirely.

## Reference: original broken file

```rust
// (paste original inline.rs here)
```

## Final note

The single most important invariant:

```
char_map.len() == total_glyphs_appended_to_job + 1
```

Every bug in the screenshot traces to a violation of this invariant or to Editor/Preview divergence. Enforce it with `CharMapBuilder`. Unify Editor and Preview on `build_line_layout` with `RenderMode`. Route tables through `egui::Grid`. Hidden syntax markers are never mapped. Blank line height is `base * 0.6` everywhere.
