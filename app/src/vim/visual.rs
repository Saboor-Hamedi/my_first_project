//! Visual and VisualLine mode character interpretation and execution.

use super::VimEngine;
use crate::editor::{Editor, VisualLine};
use crate::vim::keymap;
use crate::vim::motions::execute_visual_motion;
use crate::vim::text_objects::select_text_object;
use crate::vim::types::{VimAction, VimOperator, VimSubMode};

impl VimEngine {
    pub(crate) fn handle_visual_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        // Toggle visual modes with 'v' or 'V'
        if c == 'v' {
            if self.mode == VimSubMode::Visual {
                self.set_mode(VimSubMode::Normal, ed);
            } else {
                self.set_mode(VimSubMode::Visual, ed);
            }
            return true;
        }
        if c == 'V' {
            if self.mode == VimSubMode::VisualLine {
                self.set_mode(VimSubMode::Normal, ed);
            } else {
                self.set_mode(VimSubMode::VisualLine, ed);
            }
            return true;
        }

        // Text object selection in Visual mode (e.g. `vi"`, `va(`, `viw`)
        if let Some(inner) = self.pending_text_object_scope.take() {
            if let Some(kind) = keymap::char_to_text_object_kind(c) {
                let full_cmd = format!("{}{}", self.pending_keys, c);
                select_text_object(ed, inner, kind);
                self.last_completed_action = Some(full_cmd);
                self.pending_keys.clear();
                return true;
            }
        }
        if c == 'i' {
            self.pending_text_object_scope = Some(true);
            self.pending_keys = "vi".to_string();
            return true;
        }
        if c == 'a' {
            self.pending_text_object_scope = Some(false);
            self.pending_keys = "va".to_string();
            return true;
        }

        // Visual mode counts
        if c.is_ascii_digit() && (c != '0' || self.count_accumulator.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count_accumulator = Some(self.count_accumulator.unwrap_or(0) * 10 + d);
            self.pending_keys.push(c);
            return true;
        }

        let count = self.count_accumulator.take().unwrap_or(1);

        if let Some(action) = self.keymap.lookup_visual(&c.into()) {
            let full_cmd = if !self.pending_keys.is_empty() {
                format!("{}{}", self.pending_keys, c)
            } else {
                format!("v{}", c)
            };
            self.execute_visual_action(ed, lines, action, count);
            self.last_completed_action = Some(full_cmd);
            self.pending_keys.clear();
            return true;
        }

        false
    }

    pub(crate) fn execute_visual_action(
        &mut self,
        ed: &mut Editor,
        lines: &[VisualLine],
        action: VimAction,
        count: usize,
    ) {
        let is_line = self.mode == VimSubMode::VisualLine;
        match action {
            VimAction::Motion(m) => {
                execute_visual_motion(ed, lines, m, is_line, count);
            }
            VimAction::Operator(VimOperator::Yank) => {
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, is_line);
                }
                self.set_mode(VimSubMode::Normal, ed);
            }
            VimAction::Operator(VimOperator::Delete) => {
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, is_line);
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Normal, ed);
            }
            VimAction::Operator(VimOperator::Change) => {
                if let Some(text) = ed.selected_text() {
                    self.set_register(text, is_line);
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Insert, ed);
            }
            _ => {}
        }
    }
}
