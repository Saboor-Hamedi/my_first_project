//! Pending-operator state machine handling text objects, multipliers, and motion targets.

use super::VimEngine;
use crate::editor::{Editor, VisualLine};
use crate::vim::keymap;
use crate::vim::motions::execute_normal_motion;
use crate::vim::text_objects::apply_text_object_operator;
use crate::vim::types::{VimAction, VimMotion, VimOperator, VimSubMode};

impl VimEngine {
    pub(crate) fn handle_operator_pending(
        &mut self,
        ed: &mut Editor,
        lines: &[VisualLine],
        op: VimOperator,
        c: char,
    ) -> bool {
        if let Some(inner) = self.pending_text_object_scope {
            self.pending_keys.push(c);
            if let Some(kind) = keymap::char_to_text_object_kind(c) {
                let full_cmd = self.pending_keys.clone();
                if let Some(extracted) = apply_text_object_operator(ed, op, inner, kind) {
                    self.set_register(extracted, false);
                }
                if op == VimOperator::Change {
                    self.set_mode(VimSubMode::Insert, ed);
                } else {
                    self.set_mode(VimSubMode::Normal, ed);
                }
                self.last_completed_action = Some(full_cmd);
                self.pending_keys.clear();
                return true;
            } else {
                // Invalid delimiter target, cancel operator
                self.set_mode(VimSubMode::Normal, ed);
                self.pending_keys.clear();
                return true;
            }
        }

        // Scope selectors: 'i' (inner) or 'a' (around)
        if c == 'i' {
            self.pending_text_object_scope = Some(true);
            self.pending_keys.push('i');
            return true;
        }
        if c == 'a' {
            self.pending_text_object_scope = Some(false);
            self.pending_keys.push('a');
            return true;
        }

        // Operator-pending multiplier count (e.g. `d3w` or `c2w`)
        if c.is_ascii_digit() && (c != '0' || self.count_accumulator.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count_accumulator = Some(self.count_accumulator.unwrap_or(0) * 10 + d);
            self.pending_keys.push(c);
            return true;
        }

        let count = self.count_accumulator.take().unwrap_or(1);

        // Two-key operators (e.g. `dd`, `yy`, `cc`)
        if let Some(action) = self.keymap.lookup_operator(op, c) {
            let full_cmd = format!("{}{}", self.pending_keys, c);
            self.last_completed_action = Some(full_cmd);
            self.pending_keys.clear();
            match action {
                VimAction::OperatorLine(VimOperator::Delete) => {
                    let mut deleted = String::new();
                    for _ in 0..count {
                        let line = ed.delete_line();
                        deleted.push_str(&line);
                    }
                    self.set_register(deleted, true);
                    self.pending_op = None;
                    return true;
                }
                VimAction::OperatorLine(VimOperator::Yank) => {
                    let mut yanked = String::new();
                    for _ in 0..count {
                        let line = ed.yank_line();
                        yanked.push_str(&line);
                    }
                    self.set_register(yanked, true);
                    self.pending_op = None;
                    return true;
                }
                VimAction::OperatorLine(VimOperator::Change) => {
                    let deleted = ed.delete_line();
                    self.set_register(deleted, true);
                    self.set_mode(VimSubMode::Insert, ed);
                    return true;
                }
                VimAction::Motion(VimMotion::WordForward) => {
                    // e.g. `dw`, `yw`, `cw`
                    let mut target_cur = ed.cur;
                    for _ in 0..count {
                        target_cur = ed.next_word_boundary(target_cur);
                    }
                    if target_cur > ed.cur {
                        if op == VimOperator::Delete || op == VimOperator::Change {
                            ed.save_undo_snapshot();
                            let text: String = ed.buf[ed.cur..target_cur].iter().collect();
                            self.set_register(text, false);
                            ed.buf.drain(ed.cur..target_cur);
                        } else if op == VimOperator::Yank {
                            let text: String = ed.buf[ed.cur..target_cur].iter().collect();
                            self.set_register(text, false);
                        }
                    }
                    if op == VimOperator::Change {
                        self.set_mode(VimSubMode::Insert, ed);
                    } else {
                        self.set_mode(VimSubMode::Normal, ed);
                    }
                    return true;
                }
                _ => {}
            }
        }

        // Motion after operator (e.g. `d$`, `d0`, `dj`, `dk`)
        if let Some(VimAction::Motion(m)) = self.keymap.lookup_normal(&c.into()) {
            let full_cmd = format!("{}{}", self.pending_keys, c);
            self.last_completed_action = Some(full_cmd);
            self.pending_keys.clear();
            let initial_cur = ed.cur;
            execute_normal_motion(ed, lines, m, count);
            let motion_cur = ed.cur;
            let start = initial_cur.min(motion_cur);
            let end = initial_cur.max(motion_cur);

            if start < end {
                if op == VimOperator::Delete || op == VimOperator::Change {
                    ed.save_undo_snapshot();
                    let text: String = ed.buf[start..end].iter().collect();
                    self.set_register(text, false);
                    ed.buf.drain(start..end);
                    ed.cur = start.min(ed.buf.len());
                } else if op == VimOperator::Yank {
                    let text: String = ed.buf[start..end].iter().collect();
                    self.set_register(text, false);
                    ed.cur = initial_cur;
                }
            }

            if op == VimOperator::Change {
                self.set_mode(VimSubMode::Insert, ed);
            } else {
                self.set_mode(VimSubMode::Normal, ed);
            }
            return true;
        }

        // Unknown suffix after a pending operator: cancel and consume the key.
        // (Prevents the engine from getting stuck in operator-pending state.)
        self.set_mode(VimSubMode::Normal, ed);
        self.pending_keys.clear();
        true
    }
}
