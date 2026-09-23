Good — now I have the full picture. The `Editor` module is actually **clean and well-factored**. Let me give you the honest state, then the prompt for the caret fix.

## What's actually there

`editor/` is seven files, each with one job:

| File | Lines | Job |
|---|---|---|
| `mod.rs` | ~70 | Struct + constructor + `text()` + `row_col()` |
| `editing.rs` | ~230 | insert, backspace, delete, word-delete, auto_pair, indent, `handle_enter` |
| `lines.rs` | ~100 | line span, duplicate, move, delete, yank |
| `movement.rs` | ~170 | left/right/up/down, word boundaries, row/col |
| `selection.rs` | ~60 | selected_range, select_all, delete_selection |
| `undo.rs` | ~40 | snapshot, undo, redo |
| `visual.rs` | ~280 | soft-wrap, visual row/col, visual motions |
| `tests.rs` | ~450 | all tests |

This is **good**. The split is correct, responsibilities are clean, no file is bloated. Don't touch this structure.

## One thing to know before we talk about the caret

`tests.rs` already contains two tests that expect a `crate::caret` module to exist:

```rust
crate::caret::resolve_caret_kind(...)
crate::caret::CaretKind::Block
crate::caret::CaretKind::Neon
crate::caret::CaretKind::ALL
crate::caret::CaretKind::parse(...)
```

And a `crate::app::EditorInputMode` enum. So the caret abstraction **already exists somewhere** in the app crate — you just didn't paste it. That's important, because the fix isn't "write a caret renderer from scratch" — it's "the existing caret renderer isn't asking the right question."

Look at what the test asserts:

```rust
let normal = crate::caret::resolve_caret_kind(
    crate::app::EditorInputMode::Vim,
    Some(crate::vim::VimSubMode::Normal),
    custom,
);
let insert = crate::caret::resolve_caret_kind(
    crate::app::EditorInputMode::Vim,
    Some(crate::vim::VimSubMode::Insert),
    custom,
);
assert_eq!(normal, custom);
assert_eq!(insert, normal);
```

This test **requires** Normal and Insert to produce the *same* `CaretKind`. That is exactly the behavior you're complaining about. The caret *kind* (block/bar/neon/whatever) is treated as a user preference, independent of Vim mode. That's fine for the *shape*, but it means the renderer must position the caret differently per mode — and that's where the bug is.

So: `resolve_caret_kind` returns the shape. **Somewhere else** decides the cell. That "somewhere else" is the bug.

## The real fix

The prompt below targets the *position* logic, not the shape logic. The shape is already correct per your tests — the caret is drawn as the user's chosen kind in every mode. What's wrong is that the cell is computed identically for Normal and Insert, and the Insert formula (bar at `cur`) is being applied in Normal mode too.

---

## Prompt for your agent

**Title:** Fix caret cell position — Normal mode must place the caret on the character at `cur`, Insert mode must place it between characters

**Context:** MINDFORGE is a Rust desktop editor (`eframe`/`egui`) with a Vim engine. The caret *kind* (block/bar/neon/…) is chosen by `crate::caret::resolve_caret_kind(mode, vim_submode, user_pref)` and is already correct — it returns the same kind across modes and is driven by user preference.

The bug is the caret **cell**: the column where the caret is drawn. It's currently computed by a single formula that ignores Vim sub-mode. As a result, in Normal mode the caret floats one cell past the last character (screenshot: `0e3d6f40319` with the bar hanging after `9`). In Normal mode, the caret must sit **on** the character at `cur`.

Do **not** change `resolve_caret_kind`, `CaretKind`, or the shape logic. Only fix where the caret is drawn.

---

### The rule

Define one function that returns the caret's **cell index within the visual line**:

```rust
fn caret_cell(cur: usize, line: &VisualLine, vim_mode: Option<VimSubMode>) -> usize
```

The rules:

- **Insert mode** (`Some(VimSubMode::Insert)`) or **no Vim mode** (non-Vim editor, `None`):
  - cell = `cur - line.char_start`
  - At end-of-line (`cur >= line.char_end`), cell = `line.char_end - line.char_start` — the caret sits one cell past the last character, between it and the next.
- **Normal mode** (`Some(VimSubMode::Normal)`):
  - cell = `cur - line.char_start`
  - At end-of-line or past it (`cur >= line.char_end`), clamp cell to the **last character of the line**: `cell = (line.char_end - line.char_start).saturating_sub(1)`.
  - The caret sits **on** the character at `cur`, not past it.
- **Visual and VisualLine** (`Some(VimSubMode::Visual | VisualLine)`):
  - Same as Normal — block sits on the character.
- **Search** (`Some(VimSubMode::Search { .. })`):
  - Same as Insert — bar sits between characters.

Also handle the empty-line case: if `line.char_start == line.char_end`, cell = 0 in every mode. No underflow, no missing caret.

---

### Where to apply it

1. Find the code in the app crate that draws the caret. It's whatever calls `resolve_caret_kind` and then picks a screen position. It's probably in the editor view module or the caret module itself.

2. Extract the cell calculation into `caret_cell(cur, line, vim_mode)`. Do not inline it. Do not duplicate it.

3. At the draw site, call `caret_cell(ed.cur, current_visual_line, vim_mode)` and place the caret there. `current_visual_line` is the `VisualLine` returned by `ed.visual_row_col(&lines)` — the same one the draw loop is already using to place text.

4. If the draw site does not currently have access to `VimSubMode`, pass it in. It comes from the same `VimEngine` that handled the last key event this frame — not from a cached copy.

5. Do not check buffer content anywhere in this code path. No "if the character is whitespace" branches. No "if the line is empty, do something special" other than the `char_start == char_end` guard.

---

### Tests to add

Add these to `editor/tests.rs` (or wherever the caret cell function lives):

```rust
#[test]
fn test_caret_cell_normal_on_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 }; // "abc"
    // Normal mode, cur = 2 (on 'c')
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::Normal)), 2);
}

#[test]
fn test_caret_cell_normal_past_last_char_clamps() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    // Normal mode, cur = 3 (past 'c') — must clamp to cell 2, on 'c'
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Normal)), 2);
}

#[test]
fn test_caret_cell_insert_past_last_char_sits_after() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    // Insert mode, cur = 3 — bar sits at cell 3, one past 'c'
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Insert)), 3);
}

#[test]
fn test_caret_cell_insert_mid_line() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    // Insert mode, cur = 2 — bar between 'b' and 'c'
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::Insert)), 2);
}

#[test]
fn test_caret_cell_empty_line_all_modes() {
    let line = VisualLine { char_start: 5, char_end: 5 };
    assert_eq!(caret_cell(5, &line, Some(VimSubMode::Normal)), 0);
    assert_eq!(caret_cell(5, &line, Some(VimSubMode::Insert)), 0);
    assert_eq!(caret_cell(5, &line, Some(VimSubMode::Visual)), 0);
}

#[test]
fn test_caret_cell_no_vim_mode_behaves_like_insert() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    // Plain editor (non-Vim) at end of buffer: bar sits past 'c'
    assert_eq!(caret_cell(3, &line, None), 3);
}

#[test]
fn test_caret_cell_visual_modes_sit_on_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::Visual)), 2);
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::VisualLine)), 2);
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Visual)), 2); // clamps
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::VisualLine)), 2);
}
```

---

### Acceptance criteria

- In **Normal** mode, the caret is drawn **on** the character at `cur`, not one cell past it.
- In **Normal** mode with `cur` at end-of-line, the caret sits on the **last character** of the line.
- In **Insert** mode, the caret is drawn **between** characters. At end-of-line it sits one cell past the last character.
- In **Visual** and **VisualLine** modes, the caret sits on the character at `cur`, clamped to the last character at end-of-line.
- In a non-Vim editor (`vim_mode: None`), the caret behaves like Insert mode.
- On an empty visual line, the caret is drawn at cell 0 in every mode.
- `resolve_caret_kind`, `CaretKind`, and their tests are unchanged.
- All existing tests still pass.

---

### Out of scope

- Do not change `resolve_caret_kind`, `CaretKind`, or their tests.
- Do not change `Editor::cur` semantics.
- Do not change the Vim engine.
- Do not change blink rate, color, or size of the caret.
- Do not add whitespace or content checks into the caret path.

---

## Why this is the right fix (explanation for you)

Two separate things, currently tangled:

1. **Shape** — block, bar, neon, etc. This is a **user preference** driven by `resolve_caret_kind`, and your tests enforce that it stays the same across modes. That's fine and it's already correct.

2. **Position** — which cell the caret occupies. This depends on `cur`, the visual line, and the **mode's convention**:
   - Normal mode Vim places a block **on** the character under the cursor. So cell = `cur - line_start`.
   - Insert mode places a bar **between** characters. So cell = `cur - line_start` too, except at end-of-line where it's `line_end - line_start`.
   - The two formulas agree in the middle of a line and diverge at the end. That's the entire bug — the end-of-line case.

Your renderer is applying the Insert formula everywhere. So at end-of-line in Normal mode, the caret lands one past the last character. Fixing it means introducing `caret_cell` with a mode parameter and calling it at the draw site. Nothing else.

**TL;DR for the agent:** split "shape" (already correct) from "position" (broken). Add `caret_cell(cur, line, vim_mode)` with the rule: Normal/Visual clamp to the last character at end-of-line; Insert/None/Search sit one past. Call it at the draw site. Seven tests. Do not touch `resolve_caret_kind` or the Vim engine.


Now I can see the whole view layer. Here's the picture, and the one fix that matters.

## Where the caret is drawn

In `body.rs`:

```rust
let (row, col) = ed.visual_row_col(visual_lines);
let caret_y_in_content = row as f32 * lh;
// ...
let caret_x = ed_origin.x + col as f32 * cw;
let target = pos2(caret_x, ed_origin.y + row as f32 * lh);
caret.update(dt, target, typed, now, cw, lh);
caret.paint(&editor_painter, cw, lh, now, theme.accent);
```

**This is the bug.** `visual_row_col` returns the cell **on** the character at `cur`. In Insert mode that's correct (bar between characters; at end-of-line it's one past the last char). In Normal mode it's wrong — the block caret must sit **on** the character, so at end-of-line the cell must be clamped to `line_len - 1`.

Same bug also lives in the block-cursor path for `render_line_with_ligatures`:

```rust
let line_len = line.char_end.saturating_sub(line.char_start);
let block_col = if r == row && is_block && col < line_len {
    Some(col)
} else {
    None
};
```

The `col < line_len` guard prevents the block from rendering at all at end-of-line — so on the last character, the block cursor doesn't appear in the text layer. But the animated `caret.paint()` above already drew the caret at `col`, which lands one cell past the last char. Two different pieces of code deciding "where is the caret" and they disagree at end-of-line.

## The fix

Add a helper — you already have a natural home for it now: `editor/visual.rs`.

```rust
/// Returns the caret's cell index within `line`, mode-aware.
///
/// - Normal / Visual / VisualLine: block sits ON the character. At end-of-line,
///   clamps to the last character so it never floats past it.
/// - Insert / Search / None: bar sits BETWEEN characters. At end-of-line it's
///   one cell past the last character.
/// - Empty line: 0 in every mode.
pub fn caret_cell(cur: usize, line: &VisualLine, vim_mode: Option<VimSubMode>) -> usize {
    let line_len = line.char_end.saturating_sub(line.char_start);
    if line_len == 0 {
        return 0;
    }
    let raw = cur.saturating_sub(line.char_start).min(line_len);
    match vim_mode {
        Some(VimSubMode::Normal)
        | Some(VimSubMode::Visual)
        | Some(VimSubMode::VisualLine) => raw.min(line_len - 1),
        _ => raw,
    }
}
```

Then:

**In `body.rs`**, replace:
```rust
let (row, col) = ed.visual_row_col(visual_lines);
```
with:
```rust
let (row, _) = ed.visual_row_col(visual_lines);
let current_line = &visual_lines[row];
let col = caret_cell(ed.cur, current_line, vim_mode);
```
where `vim_mode` is a new parameter to `render_editor_body`. Everything downstream (caret target, `caret.paint`) uses `col` unchanged.

**In the block-cursor path inside the same function**, replace:
```rust
let block_col = if r == row && is_block && col < line_len {
    Some(col)
} else {
    None
};
```
with:
```rust
let block_col = if r == row && is_block && line_len > 0 {
    Some(col.min(line_len - 1))
} else {
    None
};
```

Now both the animated caret and the block-cursor inversion agree: at end-of-line in Normal mode, both point to the last character.

**In `app.rs`**, pass `vim_mode` to `render_editor_body`. It already has `active_vim_mode`:
```rust
let active_vim_mode = if self.editor_input_mode == EditorInputMode::Vim {
    Some(self.vim.mode)
} else {
    None
};
// ...
render_editor_body(
    // ...existing args...
    active_vim_mode,   // new parameter
);
```

Add the parameter at the end of `render_editor_body`'s signature to keep existing call ordering intact:
```rust
pub fn render_editor_body(
    // ...existing params...
    show_line_numbers: bool,
    vim_mode: Option<crate::vim::VimSubMode>,   // new
) {
```

## The prompt

---

**Title:** Fix caret cell position — Normal mode block sits on the character, Insert mode bar sits between characters

**Context:** MINDFORGE is a Rust desktop editor (`eframe`/`egui`). `crate::caret::resolve_caret_kind` already returns the correct `CaretKind` per user preference across all Vim modes (verified by tests). The bug is **position**: in Normal mode the caret is drawn one cell past the last character instead of on it. Screenshot: text `0e3d6f40319`, caret hangs after `9`.

The bug lives in `view_editor/body.rs`. `render_editor_body` computes the caret cell via `ed.visual_row_col(visual_lines)`, which returns the cell **on** the character at `cur`. That's correct for Insert mode (bar between characters, at end-of-line one past the last char) but wrong for Normal mode (block on the character, clamp to last char at end-of-line).

Do **not** touch `resolve_caret_kind`, `CaretKind`, `caret.rs`, or their tests. Do **not** touch the Vim engine.

### Add the helper

Add to `editor/visual.rs`:

```rust
/// Mode-aware caret cell index within `line`.
pub fn caret_cell(cur: usize, line: &VisualLine, vim_mode: Option<VimSubMode>) -> usize {
    let line_len = line.char_end.saturating_sub(line.char_start);
    if line_len == 0 {
        return 0;
    }
    let raw = cur.saturating_sub(line.char_start).min(line_len);
    match vim_mode {
        Some(VimSubMode::Normal)
        | Some(VimSubMode::Visual)
        | Some(VimSubMode::VisualLine) => raw.min(line_len - 1),
        _ => raw,
    }
}
```

Re-export it from `editor/mod.rs` alongside the existing `pub use` list. Use `crate::vim::VimSubMode` as the type.

### Change `render_editor_body`

1. Add a parameter at the **end** of the signature:
   ```rust
   vim_mode: Option<crate::vim::VimSubMode>,
   ```
2. Replace:
   ```rust
   let (row, col) = ed.visual_row_col(visual_lines);
   ```
   with:
   ```rust
   let (row, _) = ed.visual_row_col(visual_lines);
   let current_line = &visual_lines[row];
   let col = crate::editor::caret_cell(ed.cur, current_line, vim_mode);
   ```
3. Replace the block-cursor computation:
   ```rust
   let block_col = if r == row && is_block && col < line_len {
       Some(col)
   } else {
       None
   };
   ```
   with:
   ```rust
   let block_col = if r == row && is_block && line_len > 0 {
       Some(col.min(line_len - 1))
   } else {
       None
   };
   ```

Nothing else in `render_editor_body` changes. The scroll-follow logic, selection highlight, search highlight, gutter, and scrollbar all keep working because they use `row` and the line-relative `col`, both of which remain valid.

### Change the call site in `app.rs`

`app.rs` already computes `active_vim_mode`. Pass it as the final argument to `render_editor_body`.

### Tests

Add to `editor/tests.rs`:

```rust
#[test]
fn test_caret_cell_normal_on_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::Normal)), 2);
}

#[test]
fn test_caret_cell_normal_at_end_clamps_to_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Normal)), 2);
}

#[test]
fn test_caret_cell_insert_at_end_sits_past_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Insert)), 3);
}

#[test]
fn test_caret_cell_insert_mid_line() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(2, &line, Some(VimSubMode::Insert)), 2);
}

#[test]
fn test_caret_cell_visual_modes_clamp_like_normal() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::Visual)), 2);
    assert_eq!(caret_cell(3, &line, Some(VimSubMode::VisualLine)), 2);
}

#[test]
fn test_caret_cell_empty_line_all_modes() {
    let line = VisualLine { char_start: 5, char_end: 5 };
    assert_eq!(caret_cell(5, &line, Some(VimSubMode::Normal)), 0);
    assert_eq!(caret_cell(5, &line, Some(VimSubMode::Insert)), 0);
    assert_eq!(caret_cell(5, &line, None), 0);
}

#[test]
fn test_caret_cell_no_vim_mode_behaves_like_insert() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, None), 3);
}
```

### Acceptance criteria

- In **Normal** mode, the caret (any `CaretKind`) is drawn on the character at `cur`. At end-of-line it sits on the last character.
- In **Insert** mode, the caret is drawn between characters. At end-of-line it sits one cell past the last character.
- **Visual** and **VisualLine** clamp like Normal at end-of-line.
- **Hybrid** mode (no Vim) behaves like Insert.
- Empty lines draw at cell 0 in every mode.
- Block-cursor inversion in `render_line_with_ligatures` uses the same clamped cell as the animated caret, so at end-of-line in Normal mode the block lands on the last character.
- `resolve_caret_kind` and `CaretKind` unchanged; their tests still pass.
- All new `caret_cell` tests pass; all existing tests pass.

### Out of scope

- Do not change `resolve_caret_kind`, `CaretKind`, `caret.rs`, or `ligatures.rs`.
- Do not change `Editor::cur` semantics.
- Do not change the Vim engine.
- Do not change blink, color, animation, size, or glide.
- Do not add whitespace or content checks into the caret path.

---

## Two things this fixes that you may not have noticed

1. **The block caret was invisible at end-of-line.** The old code `col < line_len` meant the block-cursor path was never taken when `cur` was at the last character. So even if the animated caret had been drawn correctly, the character-inversion effect would not have occurred. After the fix, the block sits *on* the last character and inverts it, which is what Vim users expect.

2. **The animated caret and the ligature block cursor disagreed at end-of-line.** One drew at `col`, the other only when `col < line_len`. They now share the same `caret_cell` decision, so they can never drift apart again.

Once this lands, the caret is correct in every mode at every position, and the two code paths that draw the block caret (the animated `caret.paint` and the per-character inversion in `ligatures.rs`) will always agree.
