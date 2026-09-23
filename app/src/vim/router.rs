//! Top-level keystroke and character event routing across Vim sub-modes.

use super::VimEngine;
use crate::editor::{Editor, VisualLine};
use crate::vim::keymap::KeyStroke;
use crate::vim::types::VimSubMode;
use eframe::egui::{Key, Modifiers};

impl VimEngine {
    /// Handles keyboard events according to active Vim mode.
    /// Returns `true` if consumed, `false` otherwise.
    pub fn handle_key(
        &mut self,
        ed: &mut Editor,
        lines: &[VisualLine],
        key: Key,
        modifiers: Modifiers,
    ) -> bool {
        let ctrl = modifiers.ctrl || modifiers.command;

        // Escape always cancels pending states, search, or visual mode, returning to Normal
        if key == Key::Escape || (ctrl && key == Key::OpenBracket) {
            if self.is_searching() {
                self.search.cancel(ed);
            } else {
                self.search.clear_matches();
            }
            self.set_mode(VimSubMode::Normal, ed);
            self.pending_keys.clear();
            return true;
        }

        // ── Tab / Shift+Tab handling across all Vim modes ────────────────────
        if key == Key::Tab && !ctrl && !modifiers.alt {
            self.pending_keys.clear();
            match self.mode {
                VimSubMode::Insert => {
                    if modifiers.shift {
                        ed.dedent();
                    } else {
                        ed.indent();
                    }
                    return true;
                }
                VimSubMode::Normal | VimSubMode::Visual | VimSubMode::VisualLine => {
                    if modifiers.shift {
                        ed.dedent_line();
                    } else {
                        ed.indent_line();
                    }
                    return true;
                }
                VimSubMode::Search { .. } => {}
            }
        }

        // ── Ctrl+W prefix ────────────────────────────────────────────────────
        if ctrl && key == Key::W && (self.mode == VimSubMode::Normal || self.mode == VimSubMode::Visual) {
            self.pending_keys = "^W".to_string();
            return true;
        }

        // ── Search Mode Key Handling ─────────────────────────────────────────
        if self.is_searching() {
            match key {
                Key::Enter => {
                    if let Some((idx, total)) = self.search.confirm(ed) {
                        self.status_feedback = Some(format!(
                            "search: {} ({}/{})",
                            self.search.last_query, idx, total
                        ));
                    } else if !self.search.query.is_empty() {
                        self.status_feedback = Some(format!("Pattern not found: {}", self.search.query));
                    }
                    self.set_mode(VimSubMode::Normal, ed);
                    return true;
                }
                Key::Backspace => {
                    if !self.search.pop_char(ed) {
                        self.set_mode(VimSubMode::Normal, ed);
                    }
                    return true;
                }
                _ => return false,
            }
        }

        // ── Mode-Specific Key Handling ───────────────────────────────────────
        match self.mode {
            VimSubMode::Insert => {
                if ctrl && key == Key::D {
                    ed.duplicate_line();
                    return true;
                }
                false
            }
            VimSubMode::Normal => {
                let stroke = KeyStroke::Key { key, ctrl };
                if let Some(action) = self.keymap.lookup_normal(&stroke) {
                    let count = self.count_accumulator.take().unwrap_or(1);
                    self.execute_normal_action(ed, lines, action, count);
                    self.pending_keys.clear();
                    return true;
                }
                false
            }
            VimSubMode::Visual | VimSubMode::VisualLine => {
                let stroke = KeyStroke::Key { key, ctrl };
                if let Some(action) = self.keymap.lookup_visual(&stroke) {
                    let count = self.count_accumulator.take().unwrap_or(1);
                    self.execute_visual_action(ed, lines, action, count);
                    self.pending_keys.clear();
                    return true;
                }
                false
            }
            VimSubMode::Search { .. } => false,
        }
    }

    /// Handles typed character commands across all Vim modes.
    /// Returns `true` if handled, `false` to pass through to normal text insertion.
    pub fn handle_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        // Search mode accepts characters directly into search buffer
        if self.is_searching() {
            self.search.push_char(c, ed);
            return true;
        }

        match self.mode {
            VimSubMode::Insert => false, // Pass through to text editing
            VimSubMode::Normal => self.handle_normal_char(ed, lines, c),
            VimSubMode::Visual | VimSubMode::VisualLine => self.handle_visual_char(ed, lines, c),
            VimSubMode::Search { .. } => unreachable!(),
        }
    }
}
