# Editor Engines & Typography Architecture

MindForge features a dual-engine architecture designed for writing and code editing:

1. **Hybrid Mode Engine** (`app/src/hybrid/mod.rs`): Modern, non-modal IDE ergonomics with smart code-editing helpers.
2. **Vim Modal Engine** (`app/src/vim/mod.rs`): Authentic Vi/Vim modal editing with normal, insert, and visual selection modes.

Both engines operate on a unified, high-performance character-buffer `Editor` (`app/src/editor.rs`).

---

## 🧩 The Core Editor Buffer (`app/src/editor.rs`)

The `Editor` struct maintains:
- `buf: Vec<char>`: Complete Unicode-aware text buffer.
- `cur: usize`: Active cursor index within `0..=buf.len()`.
- `selection: Option<usize>`: Selection anchor index. When `Some(anchor)` and `anchor != cur`, the selection range is `(cur.min(anchor), cur.max(anchor))`.
- `undo_stack: Vec<UndoSnapshot>`: Snapshot-based undo history capturing buffer content and cursor location.
- `redo_stack: Vec<UndoSnapshot>`: Snapshot-based redo history.

### Undo & Redo Snapshots
Snapshots are recorded when:
- Whitespace is typed (word-boundary grouping).
- Significant operations take place (pasting, deletion, duplicate line, backspace on non-empty buffer, clear).
- Dedicated undo/redo key combinations (`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+Y`, and Vim `u`, `Ctrl+R`) step through the stacks with audio feedback.

---

## 🚀 Hybrid Mode (`app/src/hybrid/mod.rs`)

Hybrid mode is active by default. It gives users familiar desktop editing mechanics enriched with IDE-grade enhancements:

### 1. Smart Auto-Pairing
Typing any opening delimiter automatically balances its closing pair:
- `(` inserts `()` and leaves cursor in the center.
- `[` inserts `[]`.
- `{` inserts `{}`.
- `"` inserts `""`.
- `'` inserts `''`.
- `*` inserts `**` (Markdown bold/italic helper).
- If the cursor is positioned directly before the closing delimiter, typing the closing character seamlessly steps over it without inserting duplicates.
- If text is actively selected, typing any pair wraps the selected text inside the pair!

### 2. Line Duplication (`Ctrl+D`)
Pressing `Ctrl+D` computes the active line boundary using `ed.current_line_span()` and duplicates the line directly below, advancing the cursor while preserving indentation.

### 3. Word-Boundary Deletion
- `Ctrl + Backspace`: Skips backwards over whitespace and drains the preceding word.
- `Ctrl + Delete`: Skips forwards over whitespace and drains the succeeding word.

### 4. Tab Indentation & Dedenting
- `Tab`: Inserts 4 spaces at cursor.
- `Shift + Tab`: Dedents up to 4 leading spaces from the line.

---

## ⚡ Vim Engine (`app/src/vim/`)

The Vim Engine provides modal editing organized into a decoupled, object-oriented module hierarchy:

- **`types.rs`**: Core enums (`VimSubMode`, `VimMotion`, `VimOperator`, `TextObjectKind`, `VimAction`).
- **`keymap.rs`**: Configurable keymap registry decoupling key bindings from actions for future user customization.
- **`motions.rs`**: Executes motions with numeric count multipliers (e.g. `3j`, `5w`, `10k`).
- **`text_objects.rs`**: Nested delimiter and word matching for `ci"`, `di(`, `da{`, `yaw`, etc.
- **`search.rs`**: In-buffer search engine (`/` and `?`) with live editor highlights and `n`/`N` wrapping navigation.
- **`mod.rs`**: `VimEngine` coordinator managing state transitions, registers, and bottom bar status feedback.

### Motion Multipliers
Typing a count before a motion (e.g., `3j`, `5w`, `2dd`, `3yy`, `2p`) executes the motion or operator repeated times. The status badge displays `COUNT: X` during accumulation.

### In-Buffer Search (`/` and `?`)
- Typing `/` (forward) or `?` (backward) in Normal mode opens search input in the bottom bar with live match counter `(X matches)`.
- Live matches are highlighted directly in the editor buffer across all visible lines.
- `Enter` confirms and jumps to match; `Escape` cancels and restores prior cursor position.
- `n` and `N` cycle forward and backward through matches.

### Text Objects
- Supports inner (`i`) and around (`a`) scopes for quotes (`"`, `'`, ``` ` ```), brackets (`()`, `{}`, `[]`, `<>`), and words (`w`).
- `ci"` deletes content inside quotes and switches to Insert mode.
- `di(` deletes content within matching parentheses.
- `da{` deletes braces and their contents.
- `vi"` selects the inner text object in Visual mode.

### Dynamic Caret Morphing
- When entering **Normal Mode**, the caret dynamically morphs into a solid **Block Caret** matching the current character dimensions.
- When entering **Insert Mode** (`i`, `a`, `I`, `A`, `o`, `O`), the caret smoothly morphs into an elegant vertical **Beam Caret**.

### Visual Mode & Gap Highlighting
- In Visual mode (`v`) or Visual Line mode (`V`), moving with `j` and `k` uses visual line selection so the initial anchor is strictly preserved.
- When navigating across empty lines (`\n\n`), the selection renderer highlights empty line gaps when the active selection spans across them.

---

## 📐 Visual Rendering & Selection Logic (`app/src/view_editor.rs`)

### Soft Line Wrapping (`compute_visual_lines`)
`Editor::compute_visual_lines(max_cols)` breaks physical lines into soft-wrapped visual rows at word boundaries. If a line cannot be broken at whitespace, it gracefully wraps at `max_cols`.

### Selection Rendering Algorithm
In `render_editor_body`:
```rust
if let Some((sel_start, sel_end)) = sel_range {
    if line.char_start == line.char_end {
        // Empty line gap (\n\n):
        // Highlight a visible block when this empty line falls within active selection
        if sel_start <= line.char_start && sel_end > line.char_end {
            let sel_w = cw.max(12.0);
            let highlight_rect = Rect::from_min_size(pos2(ed_origin.x, line_y), vec2(sel_w, lh + 0.5));
            editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
        }
    } else {
        let intersect_start = sel_start.max(line.char_start);
        let intersect_end = sel_end.min(line.char_end);
        if intersect_start < intersect_end {
            let start_col = intersect_start - line.char_start;
            let end_col = intersect_end - line.char_start;
            let sel_x = ed_origin.x + start_col as f32 * cw;
            let mut sel_w = (end_col - start_col) as f32 * cw;
            // If selection extends past this line, extend highlight for newline indicator
            if sel_end > line.char_end {
                sel_w += cw.max(10.0);
            }
            let highlight_rect = Rect::from_min_size(pos2(sel_x, line_y), vec2(sel_w, lh + 0.5));
            editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
        }
    }
}
```
This guarantees that:
1. Normal text selection is pixel-perfect to character boundaries.
2. Trailing newlines visually indicate line span.
3. Multi-line selections (`Ctrl+A` or drag/visual select) touch edge-to-edge vertically without seams or rounded notches (using `0.0` corner radius and `lh + 0.5` subpixel bleed).
4. Completely empty lines between paragraphs display a visible highlight block matching modern editors.
