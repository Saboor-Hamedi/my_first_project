//! Vim modal engine facade and state machine coordinator.
//!
//! Organizes modal state transitions, count multipliers, in-buffer search,
//! text objects, and keymap routing into a clean object-oriented architecture.

pub mod keymap;
pub mod motions;
pub mod search;
pub mod text_objects;
pub mod types;

pub use keymap::{KeyStroke, VimKeymap};
pub use motions::{execute_normal_motion, execute_visual_motion};
pub use search::VimSearchState;
pub use text_objects::{apply_text_object_operator, select_text_object};
#[allow(unused_imports)]
pub use types::{
    InsertPosition, TextObjectKind, VimAction, VimMotion, VimOperator, VimSubMode,
};

use crate::editor::{Editor, VisualLine};
use eframe::egui::{Key, Modifiers};

/// High-level Vim state machine coordinator.
#[derive(Debug, Clone)]
pub struct VimEngine {
    /// Active sub-mode.
    pub mode: VimSubMode,
    /// Configurable keymap binding table.
    pub keymap: VimKeymap,
    /// In-buffer interactive search state machine.
    pub search: VimSearchState,
    /// Pending operator awaiting a motion or text object (e.g. `d`, `y`, `c`).
    pub pending_op: Option<VimOperator>,
    /// Pending text object scope (`Some(true)` for inner `i`, `Some(false)` for around `a`).
    pub pending_text_object_scope: Option<bool>,
    /// Pending prefix key for two-character sequences like `gg`.
    pub pending_prefix: Option<char>,
    /// Accumulated numeric motion/operator multiplier count (e.g. `3` in `3j` or `5` in `5w`).
    pub count_accumulator: Option<usize>,
    /// Clipboard register text.
    pub register: String,
    /// Whether register contains line-wise content (for `p`/`P` pasting).
    pub register_is_line: bool,
    /// Temporary feedback or search status notification.
    pub status_feedback: Option<String>,
    /// Pending register selection prefix (`"` waiting for register name).
    pub pending_register: Option<char>,
    /// Keystroke HUD buffer tracking pending/partial commands (e.g. `d`, `2j`, `gg`, `"a`).
    pub pending_keys: String,
    /// Timestamp when pending_keys was last updated (for ~1s timeoutlen).
    pub pending_keys_time: f64,
    /// Last completed action (e.g. "ci\"", "vi\"", "40j", "dw", "x") for HUD flash.
    pub last_completed_action: Option<String>,
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VimEngine {
    /// Constructs a new VimEngine in Normal mode with default keymaps.
    pub fn new() -> Self {
        Self {
            mode: VimSubMode::Normal,
            keymap: VimKeymap::new_standard(),
            search: VimSearchState::new(),
            pending_op: None,
            pending_text_object_scope: None,
            pending_prefix: None,
            count_accumulator: None,
            register: String::new(),
            register_is_line: false,
            status_feedback: None,
            pending_register: None,
            pending_keys: String::new(),
            pending_keys_time: 0.0,
            last_completed_action: None,
        }
    }

    /// Full descriptive label for status displays.
    #[allow(dead_code)]
    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            VimSubMode::Normal => "-- NORMAL --",
            VimSubMode::Insert => "-- INSERT --",
            VimSubMode::Visual => "-- VISUAL --",
            VimSubMode::VisualLine => "-- VISUAL LINE --",
            VimSubMode::Search { .. } => "-- SEARCH --",
        }
    }

    /// Sleek, subtle compact badge label for the status bar.
    pub fn compact_label(&self) -> String {
        if let Some(count) = self.count_accumulator {
            return format!("COUNT: {}", count);
        }
        match self.mode {
            VimSubMode::Normal => "NORMAL".into(),
            VimSubMode::Insert => "INSERT".into(),
            VimSubMode::Visual => "VISUAL".into(),
            VimSubMode::VisualLine => "V-LINE".into(),
            VimSubMode::Search { backward: false } => "/ SEARCH".into(),
            VimSubMode::Search { backward: true } => "? SEARCH".into(),
        }
    }

    /// Returns true if user is in interactive search mode.
    pub fn is_searching(&self) -> bool {
        matches!(self.mode, VimSubMode::Search { .. })
    }

    /// Sets the active vim sub-mode and synchronizes editor selection state.
    pub fn set_mode(&mut self, mode: VimSubMode, ed: &mut Editor) {
        self.mode = mode;
        self.pending_op = None;
        self.pending_text_object_scope = None;
        self.pending_prefix = None;
        self.count_accumulator = None;
        self.pending_register = None;
        self.pending_keys.clear();

        match mode {
            VimSubMode::Normal => {
                ed.clear_selection();
            }
            VimSubMode::Visual => {
                if ed.selection.is_none() {
                    ed.selection = Some(ed.cur);
                }
            }
            VimSubMode::VisualLine => {
                let (start, end) = ed.current_line_span();
                ed.selection = Some(start);
                ed.cur = end;
            }
            VimSubMode::Insert => {
                ed.clear_selection();
            }
            VimSubMode::Search { .. } => {}
        }
    }

    /// Clears pending keys on timeout (~1s timeoutlen).
    pub fn update_hud(&mut self, now: f64) {
        if !self.pending_keys.is_empty() && (now - self.pending_keys_time) > 1.0 {
            self.pending_keys.clear();
            self.pending_op = None;
            self.pending_text_object_scope = None;
            self.pending_prefix = None;
            self.pending_register = None;
            self.count_accumulator = None;
        }
    }

    /// Returns the active pending keys for HUD display (empty in Insert mode).
    pub fn pending_keys(&self) -> &str {
        if self.mode == VimSubMode::Insert {
            ""
        } else {
            &self.pending_keys
        }
    }

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

    fn handle_normal_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
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

        // ── 1. Pending Text Object Handling (e.g. `di"`, `ca(`, `ciw`) ────────
        if let Some(op) = self.pending_op {
            if let Some(inner) = self.pending_text_object_scope {
                self.pending_keys.push(c);
                if let Some(kind) = keymap::char_to_text_object_kind(c) {
                    let full_cmd = self.pending_keys.clone();
                    if let Some(extracted) = apply_text_object_operator(ed, op, inner, kind) {
                        self.register = extracted;
                        self.register_is_line = false;
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
                        self.register = deleted;
                        self.register_is_line = true;
                        self.pending_op = None;
                        return true;
                    }
                    VimAction::OperatorLine(VimOperator::Yank) => {
                        let mut yanked = String::new();
                        for _ in 0..count {
                            let line = ed.yank_line();
                            yanked.push_str(&line);
                        }
                        self.register = yanked;
                        self.register_is_line = true;
                        self.pending_op = None;
                        return true;
                    }
                    VimAction::OperatorLine(VimOperator::Change) => {
                        self.register = ed.delete_line();
                        self.register_is_line = true;
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
                                self.register = ed.buf[ed.cur..target_cur].iter().collect();
                                self.register_is_line = false;
                                ed.buf.drain(ed.cur..target_cur);
                            } else if op == VimOperator::Yank {
                                self.register = ed.buf[ed.cur..target_cur].iter().collect();
                                self.register_is_line = false;
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
                        self.register = ed.buf[start..end].iter().collect();
                        self.register_is_line = false;
                        ed.buf.drain(start..end);
                        ed.cur = start.min(ed.buf.len());
                    } else if op == VimOperator::Yank {
                        self.register = ed.buf[start..end].iter().collect();
                        self.register_is_line = false;
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

            // Unknown motion / operator combo, cancel
            self.set_mode(VimSubMode::Normal, ed);
            self.pending_keys.clear();
            return true;
        }

        // ── 2. Pending Prefix Sequences (e.g. `gg`) ──────────────────────────
        if let Some(prefix) = self.pending_prefix.take() {
            if prefix == 'g' && c == 'g' {
                ed.cur = 0;
                ed.clear_selection();
                self.last_completed_action = Some("gg".to_string());
                self.pending_keys.clear();
                return true;
            }
        }
        if c == 'g' {
            self.pending_prefix = Some('g');
            self.pending_keys = "g".to_string();
            return true;
        }

        // ── 3. Count Multipliers ('1'..='9', or subsequent '0') ──────────────
        if c.is_ascii_digit() && (c != '0' || self.count_accumulator.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count_accumulator = Some(self.count_accumulator.unwrap_or(0) * 10 + d);
            self.pending_keys.push(c);
            return true;
        }

        let count = self.count_accumulator.take().unwrap_or(1);

        // ── 4. Operator Prefixes ('d', 'y', 'c') ─────────────────────────────
        if matches!(c, 'd' | 'y' | 'c') {
            self.pending_op = match c {
                'd' => Some(VimOperator::Delete),
                'y' => Some(VimOperator::Yank),
                'c' => Some(VimOperator::Change),
                _ => None,
            };
            if count > 1 {
                self.count_accumulator = Some(count);
            }
            self.pending_keys.push(c);
            return true;
        }

        // ── 5. Standard Normal Action Dispatch ───────────────────────────────
        if let Some(action) = self.keymap.lookup_normal(&c.into()) {
            let full_cmd = if !self.pending_keys.is_empty() {
                format!("{}{}", self.pending_keys, c)
            } else {
                c.to_string()
            };
            self.execute_normal_action(ed, lines, action, count);
            self.last_completed_action = Some(full_cmd);
            self.pending_keys.clear();
            return true;
        }

        false
    }

    fn execute_normal_action(
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
                        self.register = ed.buf[ed.cur].to_string();
                        self.register_is_line = false;
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
                if !self.register.is_empty() {
                    for _ in 0..count {
                        ed.save_undo_snapshot();
                        if self.register_is_line {
                            if before {
                                ed.home();
                                ed.insert_str(&self.register.trim_end_matches('\n'));
                                ed.insert('\n');
                            } else {
                                ed.end();
                                ed.insert('\n');
                                ed.insert_str(&self.register.trim_end_matches('\n'));
                            }
                        } else {
                            if !before {
                                ed.right();
                            }
                            ed.insert_str(&self.register);
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
                self.set_mode(VimSubMode::Insert, ed);
            }
            VimAction::EnterVisual { is_line } => {
                self.set_mode(
                    if is_line {
                        VimSubMode::VisualLine
                    } else {
                        VimSubMode::Visual
                    },
                    ed,
                );
            }
            VimAction::EnterSearch { backward } => {
                self.search.start(backward, ed.cur);
                self.set_mode(VimSubMode::Search { backward }, ed);
            }
            VimAction::RepeatSearch { reverse } => {
                if let Some((idx, total)) = self.search.repeat_search(ed, reverse) {
                    self.status_feedback = Some(format!(
                        "search: {} ({}/{})",
                        self.search.last_query, idx, total
                    ));
                }
            }
            VimAction::DuplicateLine => {
                ed.duplicate_line();
            }
            _ => {}
        }
    }

    fn handle_visual_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
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

    fn execute_visual_action(
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
                    self.register = text;
                    self.register_is_line = is_line;
                }
                self.set_mode(VimSubMode::Normal, ed);
            }
            VimAction::Operator(VimOperator::Delete) => {
                if let Some(text) = ed.selected_text() {
                    self.register = text;
                    self.register_is_line = is_line;
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Normal, ed);
            }
            VimAction::Operator(VimOperator::Change) => {
                if let Some(text) = ed.selected_text() {
                    self.register = text;
                    self.register_is_line = is_line;
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Insert, ed);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_mode_transitions() {
        let mut ed = Editor::new();
        ed.insert_str("Hello World");
        ed.cur = 0;
        let mut vim = VimEngine::new();
        assert_eq!(vim.mode, VimSubMode::Normal);

        // 'i' -> Insert
        assert!(vim.handle_char(&mut ed, &[], 'i'));
        assert_eq!(vim.mode, VimSubMode::Insert);

        // Esc -> Normal
        assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
        assert_eq!(vim.mode, VimSubMode::Normal);

        // 'v' -> Visual
        assert!(vim.handle_char(&mut ed, &[], 'v'));
        assert_eq!(vim.mode, VimSubMode::Visual);
        assert!(ed.selection.is_some());

        // 'w' -> move in visual mode, expanding selection
        assert!(vim.handle_char(&mut ed, &[], 'w'));
        assert!(ed.has_selection());

        // Esc -> Normal
        assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
        assert_eq!(vim.mode, VimSubMode::Normal);
        assert!(!ed.has_selection());

        // 'v' -> Visual, then 'v' again -> Normal
        assert!(vim.handle_char(&mut ed, &[], 'v'));
        assert_eq!(vim.mode, VimSubMode::Visual);
        assert!(vim.handle_char(&mut ed, &[], 'v'));
        assert_eq!(vim.mode, VimSubMode::Normal);
        assert!(!ed.has_selection());
    }

    #[test]
    fn test_vim_motion_multipliers() {
        let mut ed = Editor::new();
        ed.insert_str("word1 word2 word3 word4 word5");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // Type '3', then 'w' -> jump forward 3 words
        assert!(vim.handle_char(&mut ed, &[], '3'));
        assert_eq!(vim.count_accumulator, Some(3));
        assert!(vim.handle_char(&mut ed, &[], 'w'));
        assert_eq!(ed.cur, 18); // Start of "word4"

        // Type '2', then 'b' -> jump back 2 words
        assert!(vim.handle_char(&mut ed, &[], '2'));
        assert!(vim.handle_char(&mut ed, &[], 'b'));
        assert_eq!(ed.cur, 6); // Start of "word2"
    }

    #[test]
    fn test_vim_text_objects() {
        let mut ed = Editor::new();
        ed.insert_str("let x = \"hello world\";");
        ed.cur = 12; // inside "hello world"
        let mut vim = VimEngine::new();

        // 'c', 'i', '"' -> change inside quotes
        assert!(vim.handle_char(&mut ed, &[], 'c'));
        assert_eq!(vim.pending_op, Some(VimOperator::Change));
        assert!(vim.handle_char(&mut ed, &[], 'i'));
        assert_eq!(vim.pending_text_object_scope, Some(true));
        assert!(vim.handle_char(&mut ed, &[], '"'));

        assert_eq!(vim.mode, VimSubMode::Insert);
        assert_eq!(ed.text(), "let x = \"\";");
        assert_eq!(ed.cur, 9);
        assert_eq!(vim.register, "hello world");
    }

    #[test]
    fn test_vim_bracket_text_objects() {
        let mut ed = Editor::new();
        ed.insert_str("fn call(param1, param2);");
        ed.cur = 10;
        let mut vim = VimEngine::new();

        // 'd', 'i', '(' -> delete inside parens
        assert!(vim.handle_char(&mut ed, &[], 'd'));
        assert!(vim.handle_char(&mut ed, &[], 'i'));
        assert!(vim.handle_char(&mut ed, &[], '('));

        assert_eq!(vim.mode, VimSubMode::Normal);
        assert_eq!(ed.text(), "fn call();");
        assert_eq!(vim.register, "param1, param2");
    }

    #[test]
    fn test_vim_search() {
        let mut ed = Editor::new();
        ed.insert_str("alpha beta gamma beta delta");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // Press '/' -> enter search mode
        assert!(vim.handle_char(&mut ed, &[], '/'));
        assert!(vim.is_searching());

        // Type "beta"
        for c in "beta".chars() {
            assert!(vim.handle_char(&mut ed, &[], c));
        }
        assert_eq!(vim.search.query, "beta");
        assert_eq!(vim.search.match_indices, vec![6, 17]);
        assert_eq!(ed.cur, 6); // live jumps to first match

        // Press Enter to confirm search
        assert!(vim.handle_key(&mut ed, &[], Key::Enter, Modifiers::default()));
        assert_eq!(vim.mode, VimSubMode::Normal);
        assert_eq!(vim.search.last_query, "beta");
        assert_eq!(ed.cur, 6);

        // Press 'n' -> next match
        assert!(vim.handle_char(&mut ed, &[], 'n'));
        assert_eq!(ed.cur, 17);

        // Press 'n' -> wraps to first match
        assert!(vim.handle_char(&mut ed, &[], 'n'));
        assert_eq!(ed.cur, 6);

        // Press 'N' -> previous match (wraps to 17)
        assert!(vim.handle_char(&mut ed, &[], 'N'));
        assert_eq!(ed.cur, 17);
    }

    #[test]
    fn test_vim_keystroke_hud() {
        let mut ed = Editor::new();
        ed.insert_str("line 1\nline 2\nline 3\n");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // 1. Partial operator 'd'
        assert!(vim.handle_char(&mut ed, &[], 'd'));
        assert_eq!(vim.pending_keys(), "d");

        // Complete 'dw'
        assert!(vim.handle_char(&mut ed, &[], 'w'));
        assert_eq!(vim.pending_keys(), "");

        // 2. Multiplier '2'
        assert!(vim.handle_char(&mut ed, &[], '2'));
        assert_eq!(vim.pending_keys(), "2");

        // Complete '2j'
        assert!(vim.handle_char(&mut ed, &[], 'j'));
        assert_eq!(vim.pending_keys(), "");

        // 3. Prefix 'g'
        assert!(vim.handle_char(&mut ed, &[], 'g'));
        assert_eq!(vim.pending_keys(), "g");

        // Complete 'gg'
        assert!(vim.handle_char(&mut ed, &[], 'g'));
        assert_eq!(vim.pending_keys(), "");

        // 4. Register prefix '"' then 'a'
        assert!(vim.handle_char(&mut ed, &[], '"'));
        assert_eq!(vim.pending_keys(), "\"");
        assert!(vim.handle_char(&mut ed, &[], 'a'));
        assert_eq!(vim.pending_keys(), "\"a");

        // Cancel with Escape
        assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
        assert_eq!(vim.pending_keys(), "");

        // 5. HUD empty in Insert mode
        vim.set_mode(VimSubMode::Insert, &mut ed);
        assert_eq!(vim.pending_keys(), "");
        vim.set_mode(VimSubMode::Normal, &mut ed);

        // 6. Timeout after 1.0s clears pending keys
        assert!(vim.handle_char(&mut ed, &[], 'd'));
        assert_eq!(vim.pending_keys(), "d");
        vim.pending_keys_time = 100.0;
        vim.update_hud(101.5);
        assert_eq!(vim.pending_keys(), "");
    }

    #[test]
    fn test_vim_tab_indentation() {
        let mut ed = Editor::new();
        let mut vim = VimEngine::new();

        // In Insert mode at line start: Tab inserts 4 spaces
        vim.set_mode(VimSubMode::Insert, &mut ed);
        assert!(vim.handle_key(&mut ed, &[], Key::Tab, Modifiers::default()));
        assert_eq!(ed.text(), "    ");

        // Shift+Tab dedents line
        let mut shift_mod = Modifiers::default();
        shift_mod.shift = true;
        assert!(vim.handle_key(&mut ed, &[], Key::Tab, shift_mod));
        assert_eq!(ed.text(), "");

        // In Normal mode: Tab indents line
        ed.insert_str("hello");
        vim.set_mode(VimSubMode::Normal, &mut ed);
        assert!(vim.handle_key(&mut ed, &[], Key::Tab, Modifiers::default()));
        assert_eq!(ed.text(), "    hello");

        // In Normal mode: Shift+Tab dedents line
        assert!(vim.handle_key(&mut ed, &[], Key::Tab, shift_mod));
        assert_eq!(ed.text(), "hello");
    }
}
