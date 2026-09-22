Prompt for your agent

Title: Fix paste undo granularity, caret preservation, auto-numbered/bulleted lists, and indentation preservation on newline

Context: MINDFORGE is a Rust modal editor (Vim + Hybrid modes). The Editor core already has:
- insert_str(&mut self, s: &str) for multi-character insertion
- save_undo_snapshot() which pushes a full EditorSnapshot to undo_stack (capped at 300)
- undo() / redo()
- insert(char) which snapshots per whitespace char or when the stack is empty
- insert_line_below()
- indent() / dedent() which use 4 spaces

Task 1 — Paste must undo as ONE action, not word by word

Bug report: The user pastes ~20 words. Pressing undo repeatedly removes them word by word instead of restoring the whole paste in one step.

Root cause: The paste path is going through insert(char) in a loop (or equivalent), and each character/word triggers its own save_undo_snapshot(), so the undo stack gets one snapshot per character. Undo then pops them one at a time.

Fix:
- The paste handler must call save_undo_snapshot() exactly ONCE before inserting the whole pasted string, then insert all characters WITHOUT triggering further snapshots.
- The cleanest implementation: route paste through insert_str(&str), and make sure insert_str:
  - Calls save_undo_snapshot() once at the start (after deleting any selection).
  - Does NOT call any per-character snapshot path.
  - Inserts all chars into buf in one go (e.g. splice, or reserve + extend), then advances cur.
- Audit every other bulk mutation (delete_line, duplicate_line, move_line_up/down, auto_pair, indent/dedent) to confirm each still takes exactly one snapshot. Do not change their behavior — just verify.
- Add a test: insert_str("one two three four five six ... twenty words"), then call undo() once, assert buffer is back to its pre-paste state. Calling undo() a second time should restore the state before that.

Task 2 — Caret must preserve its position across undo/redo and paste

Bug report: After undo/redo or a paste, the caret jumps to a position that is not where the user expects it.

Fix:
- EditorSnapshot already stores { buf, cur }. On undo(), self.cur = prev.cur.min(self.buf.len()) — verify this is correct and that it's actually being applied (it is, but confirm no other code path clobbers cur afterward).
- After a paste via insert_str, the caret should end up immediately AFTER the last inserted character (which the current code does with self.cur += 1 per char — replace with self.cur += inserted_len once).
- Selection must be cleared on undo/redo (already done) and after paste (already done). Confirm no lingering selection anchor survives.
- Add tests:
  - Type "abc", move caret to 1, insert_str("XYZ"), assert cur == 4 and buf == "aXYZbc".
  - Type "hello", undo(), assert cur is at the pre-undo position and matches the snapshot's cur.
  - redo() then undo() round-trip preserves cur exactly.
- Do not change undo/redo semantics — only ensure cur is snapshotted and restored faithfully.

Task 3 — Auto-continue numbered lists and bullet lists on Enter

Bug report: When the user types a line like "1. Hello" and presses Enter, the next line should start with "2. ". Same for "- Hello" and "* Hello" — Enter should insert a new "- " or "* " line. Currently nothing is inserted.

Fix:
- Implement list continuation in the Enter handler (INSERT mode). Before inserting the newline, inspect the current line (use current_line_span or equivalent):
  1. Numbered list: if the line matches ^(\s*)(\d+)\.\s+(.*)$ then after the newline insert the same leading whitespace + the next integer + ". ".
     - Increment the number by 1. Preserve leading whitespace exactly (see Task 4).
     - If the number would overflow, just keep the same number.
  2. Bulleted list: if the line matches ^(\s*)([-*+])\s+(.*)$ then after the newline insert the same leading whitespace + the same bullet char + " ".
     - Preserve the exact bullet char used (-, *, or +).
  3. If the current line is ONLY a list marker (e.g. "1. " with no content) and the user hits Enter, do NOT continue the list — just insert a plain newline and clear the marker. This matches Vim/VS Code behavior.
- Only trigger in INSERT mode.
- Do not trigger inside a selection replacement.
- Snapshot once (reuse the Enter path's existing snapshot; do not add a second one).
- Add tests:
  - "1. Hello" + Enter → next line is "2. "
  - "1. Hello\n2. World" + Enter at end of line 2 → next line is "3. "
  - "- Hello" + Enter → next line is "- "
  - "* Hello" + Enter → next line is "* "
  - "+ Hello" + Enter → next line is "+ "
  - "  1. Hello" (2 spaces indent) + Enter → next line is "  2. "
  - "1. " (marker only) + Enter → next line is "" (no marker)

Task 4 — Preserve indentation when pressing Enter

Bug report: The user has one tab (or spaces) at the start of a line. Pressing Enter does not carry that leading indentation to the next line.

Fix:
- In the Enter handler, after inserting '\n', copy the leading whitespace of the current line to the start of the new line.
- Leading whitespace = the run of spaces and tabs from line_start up to the first non-whitespace character.
- This must compose with Task 3:
  - If the line is "  - Hello", Enter → next line should be "  - " (indent preserved AND bullet continued).
  - If the line is "\tHello", Enter → next line should be "\t" (indent preserved, no bullet).
  - If the line is "    Hello", Enter → next line should be "    " (4 spaces preserved).
- Snapshot once for the whole Enter action.
- Add tests for each of the above.

Acceptance criteria
- Pasting 20 words and pressing undo ONCE restores the pre-paste buffer. A second undo restores the state before that.
- Caret position after paste is immediately after the last pasted char.
- Caret position is exactly restored by undo and by redo (round-trip stable).
- Enter after "1. Hello" produces "2. " on the next line (with the same indent).
- Enter after "- Hello", "* Hello", "+ Hello" produces the same marker on the next line (with the same indent).
- Enter after a marker-only line ("1. ", "- ") does NOT continue the list.
- Enter after an indented line preserves the leading whitespace on the new line.
- All previous tests still pass; new tests added for each behavior above.

Out of scope
- No changes to undo/redo stack size or snapshot format unless required.
- No changes to indent/dedent tab width (stays 4 spaces).
- No auto-renumbering of existing list items below the cursor.
- No Markdown parsing beyond the simple regex-like checks described.

--

Look at the screenshot — the caret (green block) is sitting **on top of the last character** (`e` in `--prune`), not **after** it. That's the bug, and it's almost certainly the same root cause for both symptoms.

## What's actually happening

In a TUI editor you have two different "caret" concepts:

1. **The logical caret** — `Editor.cur`, an *insertion index* between characters. In Insert mode, `cur == buf.len()` means "cursor is after the last char". That's correct.
2. **The visual caret** — where the renderer draws the block/bar. This is computed from `cur` **and the current mode**.

The renderer is drawing the block **on the cell at `cur`** instead of **after the cell at `cur`** when in Insert mode, and it's lagging by one frame while typing fast.

## The two bugs, precisely

**Bug A — caret drawn on the last letter instead of after it.**
Your render code is doing something like:
```rust
let (row, col) = visual_row_col(lines);   // col = index of char at cur
draw_block(row, col);                     // draws OVER that char
```
But in Insert mode, a block caret should be drawn at `col` **only if** there's a char there and you're replacing it; in Insert mode most editors draw a **bar** *before* the char, or a block *on the cell after the last char*. The typical correct rule:

- **Normal mode (block):** draw block **on** the char at `cur`.
- **Insert mode (bar or block):** draw at the cell **at `cur`**, but if `cur == line_end` (past the last char), draw at the cell **just after** the last char.
- **Never** draw at `col - 1`.

Your current code is effectively drawing at `col - 1` (or at the last char's column) when `cur` is at end-of-line.

**Bug B — caret lags by one keystroke when typing fast.**
This is a **render vs. event ordering** issue. If you're redrawing the frame *before* applying the key event (or using a cached `cur` from the previous frame), the caret will always be one character behind. This happens when:

- You `terminal.draw(...)` at the top of the loop, then `handle_key(...)` after — so the drawn frame shows the state *before* the last key.
- Or you throttle/cache the render and the cache isn't invalidated on insert.

## The fix

**1. Make the caret position a pure function of `(cur, mode, line)`** — one place, no special-casing whitespace or end-of-line:

```rust
fn caret_cell(cur: usize, line: &VisualLine, mode: Mode) -> usize {
    match mode {
        Mode::Insert => cur.saturating_sub(line.char_start), // bar: sits ON the char to its right
        Mode::Normal => cur.saturating_sub(line.char_start), // block: sits ON the char under cursor
        Mode::Visual => cur.saturating_sub(line.char_start),
        Mode::Replace => cur.saturating_sub(line.char_start),
    }
}
```

Then in the draw pass, when `cur == line.char_end` (end of visual line), draw at `line.char_end - line.char_start` (i.e. **one cell past the last char**) so the caret floats after the text instead of eating the last letter:

```rust
let cell = if cur >= line.char_end && !line.is_empty() {
    line.char_end - line.char_start      // one past the last char
} else {
    cur - line.char_start
};
```

**2. Fix the event/render order in your main loop.**

The correct order is:

```rust
loop {
    terminal.draw(|f| ui(f, &app))?;   // 1. draw current state
    if let Event::Key(k) = event::read()? {
        app.handle_key(k);             // 2. mutate state AFTER drawing
    }
}
```

If your loop currently does `handle_key` **before** `draw`, move `draw` after the mutation, or restructure as:

```rust
loop {
    if let Event::Key(k) = event::read()? {
        app.handle_key(k);
    }
    terminal.draw(|f| ui(f, &app))?;   // draw always reflects latest state
}
```

Either is fine — the rule is: **the frame you see must be rendered from state that already includes the last key.**

**3. Invalidate any render cache on every mutation.**

If you cache the visual line map or the `row_col` computation, clear it in `insert`, `insert_str`, `backspace`, `delete`, and every cursor move. Otherwise the caret is computed against a stale layout and appears one step behind.

## Tests to add

- `"git fetch --prune"`, `cur = buf.len()`, Insert mode → caret cell is `line.char_end - line.char_start` (one past `e`), **not** on `e`.
- Same string, Normal mode, `cur = buf.len() - 1` → caret cell is on `e`.
- Fast-typing simulation: feed N key events in a row without redrawing; after the final draw, caret cell == `buf.len() - line.char_start`. If you have a headless test harness, this catches the lag.
- Empty line: `cur = 0`, line is empty → caret cell is 0 (no underflow).

## One-line summary for your agent

> The caret is drawn **on** the last character instead of **after** it when `cur` is at end-of-line, and it lags one keystroke behind when typing fast because the frame is rendered before the key event is applied. Fix: (a) compute caret cell as a pure function of `(cur, mode, line)` and place it one past the last char when `cur == line.char_end`; (b) reorder the main loop so `draw` always runs on post-key state, and invalidate any cached visual-line map on every mutation.

Want me to write the exact Rust patch for your render loop and the caret-cell function, assuming a `ratatui` + `crossterm` setup?