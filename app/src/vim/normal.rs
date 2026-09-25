//! Normal mode character command interpretation and action execution.
//!
//! Sections in `handle_normal_char`:
//!   0. Pending Register Selection (e.g. `"a`)
//!   1. Pending Operator & Text Object Handling (delegated to operator module)
//!   2. Double-character motion combinations (e.g. `gg`)
//!   3. Numeric count multiplier accumulation (e.g. `3` in `3j`, `12` in `12w`)
//!   4. Operator Prefixes ('d', 'y', 'c')
//!   5. Standard Normal Action Dispatch

use super::VimEngine;
use crate::editor::{Editor, VisualLine};
use crate::vim::motions::execute_normal_motion;
use crate::vim::types::{InsertPosition, VimAction};

impl VimEngine {
    pub(crate) fn handle_normal_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        // ── 0. Pending Register Selection (e.g. `"a`) ───────────────────────
        if self.pending_op.is_none() {
            if let Some(prefix) = self.pending_register.take() {
                if prefix == '"' {
                    self.pending_keys.push(c);
                    return true;
                }
            }
            if c == '"' {
                self.pending_register = Some('"');
                self.pending_keys = "\"".to_string();
                return true;
            }
        }

        // ── 0.5. Active Selection Direct Edit (e.g. after Ctrl+A or mouse drag) ──
        if ed.has_selection() {
            if c == 'd' || c == 'D' || c == 'x' {
                ed.save_undo_snapshot();
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, false);
                }
                ed.delete_selection();
                self.last_completed_action = Some(c.to_string());
                self.pending_keys.clear();
                return true;
            } else if c == 'c' || c == 'C' {
                ed.save_undo_snapshot();
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, false);
                }
                ed.delete_selection();
                self.set_mode(crate::vim::types::VimSubMode::Insert, ed);
                self.last_completed_action = Some(c.to_string());
                self.pending_keys.clear();
                return true;
            } else if c == 'y' || c == 'Y' {
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, false);
                }
                ed.clear_selection();
                self.last_completed_action = Some(c.to_string());
                self.pending_keys.clear();
                return true;
            }
        }

        // ── 1. Pending Operator Handling (e.g. `di"`, `ca(`, `ciw`, `dw`, `d$`)
        if let Some(op) = self.pending_op {
            return self.handle_operator_pending(ed, lines, op, c);
        }

        // ── 2. Pending Prefix Sequences (e.g. `gg`) ──────────────────────────
        if let Some(prefix) = self.pending_prefix.take() {
            if let Some(action) = self.keymap.lookup_combination(prefix, c) {
                let count = self.count_accumulator.take().unwrap_or(1);
                self.execute_normal_action(ed, lines, action, count);
                self.last_completed_action = Some(format!("{}{}", prefix, c));
                self.pending_keys.clear();
                return true;
            }
            // Unknown second key after prefix — clear pending and re-process 'c' normally below
            self.pending_keys.clear();
            // fall through — 'c' will be processed by the sections below
        }
        if self.keymap.is_combination_prefix(c) {
            self.pending_prefix = Some(c);
            self.pending_keys = c.to_string();
            return true;
        }

        // ── 3. Numeric Count Multiplier Accumulation ─────────────────────────
        if c.is_ascii_digit() && (c != '0' || self.count_accumulator.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count_accumulator = Some(self.count_accumulator.unwrap_or(0) * 10 + d);
            self.pending_keys.push(c);
            return true;
        }

        let count = self.count_accumulator.take().unwrap_or(1);

        // ── 4. Standard Normal Action Dispatch ───────────────────────────────
        if let Some(action) = self.keymap.lookup_normal(&c.into()) {
            let full_cmd = if !self.pending_keys.is_empty() {
                format!("{}{}", self.pending_keys, c)
            } else {
                c.to_string()
            };
            self.execute_normal_action(ed, lines, action.clone(), count);

            // If the action entered operator-pending, keep the operator char in
            // pending_keys for the HUD and do not record a completed action yet.
            if matches!(action, VimAction::Operator(_)) {
                self.pending_keys = full_cmd;
                return true;
            }

            self.last_completed_action = Some(full_cmd);
            self.pending_keys.clear();
            return true;
        }

        false
    }

    pub(crate) fn execute_normal_action(
        &mut self,
        ed: &mut Editor,
        lines: &[VisualLine],
        action: VimAction,
        count: usize,
    ) {
        match action {
            VimAction::Motion(m) => {
                execute_normal_motion(ed, lines, m, count);
            }
            VimAction::DeleteChar => {
                for _ in 0..count {
                    if ed.cur < ed.buf.len() {
                        ed.save_undo_snapshot();
                        let text = ed.buf[ed.cur].to_string();
                        self.set_register(text, false);
                        ed.buf.remove(ed.cur);
                    }
                }
            }
            VimAction::Undo => {
                for _ in 0..count {
                    ed.undo();
                }
            }
            VimAction::Redo => {
                for _ in 0..count {
                    ed.redo();
                }
            }
            VimAction::Paste { before } => {
                let paste_text = if !self.register.is_empty() {
                    Some(self.register.clone())
                } else {
                    crate::input::global::get_win32_clipboard()
                };
                if let Some(text) = paste_text {
                    for _ in 0..count {
                        ed.save_undo_snapshot();
                        if self.register_is_line {
                            if before {
                                ed.home();
                                ed.insert_str(&text.trim_end_matches('\n'));
                                ed.insert('\n');
                            } else {
                                ed.end();
                                ed.insert('\n');
                                ed.insert_str(&text.trim_end_matches('\n'));
                            }
                        } else {
                            if !before {
                                ed.right();
                            }
                            ed.insert_str(&text);
                        }
                    }
                }
            }
            VimAction::EnterInsert(pos) => {
                match pos {
                    InsertPosition::AtCursor => {}
                    InsertPosition::AfterCursor => ed.right(),
                    InsertPosition::LineStart => ed.home(),
                    InsertPosition::LineEnd => ed.end(),
                    InsertPosition::LineBelow => ed.insert_line_below(),
                    InsertPosition::LineAbove => {
                        ed.home();
                        ed.insert('\n');
                        ed.up();
                    }
                }
                self.set_mode(crate::vim::types::VimSubMode::Insert, ed);
            }
            VimAction::EnterVisual { is_line } => {
                let target_mode = if is_line {
                    crate::vim::types::VimSubMode::VisualLine
                } else {
                    crate::vim::types::VimSubMode::Visual
                };
                self.set_mode(target_mode, ed);
            }
            VimAction::EnterSearch { backward } => {
                self.search.start(backward, ed.cur);
                self.set_mode(crate::vim::types::VimSubMode::Search { backward }, ed);
            }
            VimAction::RepeatSearch { reverse } => {
                for _ in 0..count {
                    self.search.repeat_search(ed, reverse);
                }
            }
            VimAction::Operator(op) => {
                self.pending_op = Some(op);
                if count > 1 {
                    self.count_accumulator = Some(count);
                }
            }
            VimAction::OperatorToEndOfLine(op) => {
                let initial_cur = ed.cur;
                ed.end();
                let end_cur = ed.cur;
                let start = initial_cur.min(end_cur);
                let end = initial_cur.max(end_cur);
                if start < end {
                    if op == crate::vim::types::VimOperator::Delete || op == crate::vim::types::VimOperator::Change {
                        ed.save_undo_snapshot();
                        let text: String = ed.buf[start..end].iter().collect();
                        self.set_register(text, false);
                        ed.buf.drain(start..end);
                        ed.cur = start.min(ed.buf.len());
                    } else if op == crate::vim::types::VimOperator::Yank {
                        let text: String = ed.buf[start..end].iter().collect();
                        self.set_register(text, false);
                        ed.cur = initial_cur;
                    }
                }
                if op == crate::vim::types::VimOperator::Change {
                    self.set_mode(crate::vim::types::VimSubMode::Insert, ed);
                }
            }
            VimAction::OperatorLine(op) => {
                match op {
                    crate::vim::types::VimOperator::Delete => {
                        let mut deleted = String::new();
                        for _ in 0..count {
                            let line = ed.delete_line();
                            deleted.push_str(&line);
                        }
                        self.set_register(deleted, true);
                    }
                    crate::vim::types::VimOperator::Yank => {
                        let mut yanked = String::new();
                        for _ in 0..count {
                            let line = ed.yank_line();
                            yanked.push_str(&line);
                        }
                        self.set_register(yanked, true);
                    }
                    crate::vim::types::VimOperator::Change => {
                        let deleted = ed.delete_line();
                        self.set_register(deleted, true);
                        self.set_mode(crate::vim::types::VimSubMode::Insert, ed);
                    }
                }
            }
            VimAction::ToggleTaskCheckbox => {
                let (row, _) = ed.row_col();
                ed.save_undo_snapshot();
                if toggle_task_checkbox_on_line(ed, row) {
                    self.last_completed_action = Some("ToggleTaskCheckbox".into());
                } else {
                    ed.undo_stack.pop();
                }
            }
            VimAction::Cancel => {
                self.set_mode(crate::vim::types::VimSubMode::Normal, ed);
            }
            other => {
                debug_assert!(
                    false,
                    "unhandled VimAction in normal dispatch: {:?}",
                    other
                );
            }
        }
    }
}

/// Flips `- [ ]`/`* [ ]` <-> `- [x]`/`* [x]` (case-insensitive `x`) on one
/// line, if that line is actually a task item. Returns whether it changed
/// anything — lines that aren't task items are left alone rather than having
/// checkbox syntax invented on them.
pub fn toggle_task_checkbox_on_line(ed: &mut Editor, row: usize) -> bool {
    let (start, end) = ed.line_char_range(row);
    if start >= end || end > ed.buf.len() {
        return false;
    }
    let line: Vec<char> = ed.buf[start..end].to_vec();

    let indent = line.iter().take_while(|&&c| c == ' ' || c == '\t').count();
    let rest = &line[indent..];
    let is_task = |c0: char, c1: char| (c0 == '-' || c0 == '*') && c1 == ' ';
    if rest.len() < 6 || !is_task(rest[0], rest[1]) || rest[2] != '[' || rest[4] != ']' || rest[5] != ' ' {
        return false;
    }

    let checked = rest[3] == 'x' || rest[3] == 'X';
    let new_char = if checked { ' ' } else { 'x' };
    ed.buf[start + indent + 3] = new_char;
    true
}

/// Returns the inclusive range of line numbers selected in the editor.
pub fn selected_line_range(ed: &Editor) -> std::ops::RangeInclusive<usize> {
    if let Some((start, end)) = ed.selected_range() {
        let (start_row, _) = ed.row_col_of(start);
        let end_idx = if end > start { end.saturating_sub(1) } else { start };
        let (end_row, _) = ed.row_col_of(end_idx);
        start_row.min(end_row)..=start_row.max(end_row)
    } else {
        let (row, _) = ed.row_col();
        row..=row
    }
}

