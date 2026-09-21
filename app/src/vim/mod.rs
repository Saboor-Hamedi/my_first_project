//! Vim modal engine — object-oriented, clean state machine supporting
//! Normal, Insert, Visual, and VisualLine modes with motions, operators, and registers.

use crate::editor::{Editor, VisualLine};
use eframe::egui::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimSubMode {
    Normal,
    Insert,
    Visual,
    VisualLine,
}

#[derive(Debug, Clone)]
pub struct VimEngine {
    pub mode: VimSubMode,
    pub pending_op: Option<char>,
    pub register: String,
    pub register_is_line: bool,
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VimEngine {
    pub fn new() -> Self {
        Self {
            mode: VimSubMode::Normal,
            pending_op: None,
            register: String::new(),
            register_is_line: false,
        }
    }

    #[allow(dead_code)]
    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            VimSubMode::Normal => "-- NORMAL --",
            VimSubMode::Insert => "-- INSERT --",
            VimSubMode::Visual => "-- VISUAL --",
            VimSubMode::VisualLine => "-- VISUAL LINE --",
        }
    }

    /// Sleek, subtle compact badge label for status bar.
    pub fn compact_label(&self) -> &'static str {
        match self.mode {
            VimSubMode::Normal => "NORMAL",
            VimSubMode::Insert => "INSERT",
            VimSubMode::Visual => "VISUAL",
            VimSubMode::VisualLine => "V-LINE",
        }
    }

    /// Sets the active vim sub-mode and synchronizes editor state.
    pub fn set_mode(&mut self, mode: VimSubMode, ed: &mut Editor) {
        self.mode = mode;
        self.pending_op = None;
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

        // Escape always cancels pending operations and returns to Normal mode
        if key == Key::Escape || (ctrl && key == Key::OpenBracket) {
            self.set_mode(VimSubMode::Normal, ed);
            return true;
        }

        match self.mode {
            VimSubMode::Insert => {
                if ctrl && key == Key::D {
                    ed.duplicate_line();
                    return true;
                }
                false
            }
            VimSubMode::Normal => {
                if ctrl {
                    match key {
                        Key::R => {
                            ed.redo();
                            return true;
                        }
                        Key::ArrowLeft => {
                            ed.word_left();
                            return true;
                        }
                        Key::ArrowRight => {
                            ed.word_right();
                            return true;
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        Key::ArrowDown => {
                            ed.down_visual(lines);
                            return true;
                        }
                        Key::ArrowUp => {
                            ed.up_visual(lines);
                            return true;
                        }
                        Key::ArrowLeft => {
                            ed.left();
                            return true;
                        }
                        Key::ArrowRight => {
                            ed.right();
                            return true;
                        }
                        Key::Home => {
                            ed.home_visual(lines);
                            return true;
                        }
                        Key::End => {
                            ed.end_visual(lines);
                            return true;
                        }
                        _ => {}
                    }
                }
                false
            }
            VimSubMode::Visual | VimSubMode::VisualLine => {
                if ctrl && key == Key::C {
                    // Yank in visual mode
                    if let Some(text) = ed.selected_text() {
                        self.register = text;
                        self.register_is_line = self.mode == VimSubMode::VisualLine;
                    }
                    self.set_mode(VimSubMode::Normal, ed);
                    return true;
                }
                match key {
                    Key::ArrowDown => {
                        ed.down_visual_select(lines);
                        true
                    }
                    Key::ArrowUp => {
                        ed.up_visual_select(lines);
                        true
                    }
                    Key::ArrowLeft => {
                        ed.left_select();
                        true
                    }
                    Key::ArrowRight => {
                        ed.right_select();
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    /// Handles typed character commands in Normal and Visual modes.
    /// Returns `true` if handled, `false` to pass through to normal text insertion.
    pub fn handle_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        match self.mode {
            VimSubMode::Insert => false, // Handled as normal text typing
            VimSubMode::Normal => self.handle_normal_char(ed, lines, c),
            VimSubMode::Visual | VimSubMode::VisualLine => self.handle_visual_char(ed, lines, c),
        }
    }

    fn handle_normal_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        // Handle pending two-key operators
        if let Some(op) = self.pending_op.take() {
            match (op, c) {
                ('d', 'd') => {
                    self.register = ed.delete_line();
                    self.register_is_line = true;
                    return true;
                }
                ('d', 'w') => {
                    let next = ed.next_word_boundary(ed.cur);
                    if next > ed.cur {
                        ed.save_undo_snapshot();
                        self.register = ed.buf[ed.cur..next].iter().collect();
                        self.register_is_line = false;
                        ed.buf.drain(ed.cur..next);
                    }
                    return true;
                }
                ('y', 'y') => {
                    self.register = ed.yank_line();
                    self.register_is_line = true;
                    return true;
                }
                ('y', 'w') => {
                    let next = ed.next_word_boundary(ed.cur);
                    if next > ed.cur {
                        self.register = ed.buf[ed.cur..next].iter().collect();
                        self.register_is_line = false;
                    }
                    return true;
                }
                ('c', 'c') => {
                    self.register = ed.delete_line();
                    self.register_is_line = true;
                    self.set_mode(VimSubMode::Insert, ed);
                    return true;
                }
                ('c', 'w') => {
                    let next = ed.next_word_boundary(ed.cur);
                    if next > ed.cur {
                        ed.save_undo_snapshot();
                        self.register = ed.buf[ed.cur..next].iter().collect();
                        self.register_is_line = false;
                        ed.buf.drain(ed.cur..next);
                    }
                    self.set_mode(VimSubMode::Insert, ed);
                    return true;
                }
                ('g', 'g') => {
                    ed.cur = 0;
                    ed.clear_selection();
                    return true;
                }
                _ => {
                    // Cancel unknown operator combo
                    return true;
                }
            }
        }

        // Single-key commands and operator prefixes
        match c {
            // ── Operators ────────────────────────────────────────────────────
            'd' | 'y' | 'c' | 'g' => {
                self.pending_op = Some(c);
                true
            }
            // ── Motions ──────────────────────────────────────────────────────
            'h' => {
                ed.left();
                true
            }
            'l' => {
                ed.right();
                true
            }
            'j' => {
                ed.down_visual(lines);
                true
            }
            'k' => {
                ed.up_visual(lines);
                true
            }
            'w' => {
                ed.word_right();
                true
            }
            'b' => {
                ed.word_left();
                true
            }
            '0' => {
                ed.home_visual(lines);
                true
            }
            '$' => {
                ed.end_visual(lines);
                true
            }
            'G' => {
                ed.cur = ed.buf.len();
                ed.clear_selection();
                true
            }
            // ── Editing verbs ────────────────────────────────────────────────
            'x' => {
                if ed.cur < ed.buf.len() {
                    ed.save_undo_snapshot();
                    self.register = ed.buf[ed.cur].to_string();
                    self.register_is_line = false;
                    ed.buf.remove(ed.cur);
                }
                true
            }
            'u' => {
                ed.undo();
                true
            }
            'p' => {
                if !self.register.is_empty() {
                    ed.save_undo_snapshot();
                    if self.register_is_line {
                        ed.end();
                        ed.insert('\n');
                        ed.insert_str(&self.register.trim_end_matches('\n'));
                    } else {
                        ed.right();
                        ed.insert_str(&self.register);
                    }
                }
                true
            }
            'P' => {
                if !self.register.is_empty() {
                    ed.save_undo_snapshot();
                    if self.register_is_line {
                        ed.home();
                        ed.insert_str(&self.register.trim_end_matches('\n'));
                        ed.insert('\n');
                    } else {
                        ed.insert_str(&self.register);
                    }
                }
                true
            }
            // ── Mode transitions ─────────────────────────────────────────────
            'i' => {
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'a' => {
                ed.right();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'I' => {
                ed.home();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'A' => {
                ed.end();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'o' => {
                ed.insert_line_below();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'O' => {
                ed.home();
                ed.insert('\n');
                ed.up();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'v' => {
                self.set_mode(VimSubMode::Visual, ed);
                true
            }
            'V' => {
                self.set_mode(VimSubMode::VisualLine, ed);
                true
            }
            _ => false,
        }
    }

    fn handle_visual_char(&mut self, ed: &mut Editor, lines: &[VisualLine], c: char) -> bool {
        match c {
            // Motions in visual mode
            'h' => {
                ed.left_select();
                true
            }
            'l' => {
                ed.right_select();
                true
            }
            'j' => {
                if self.mode == VimSubMode::VisualLine {
                    ed.down_visual_select(lines);
                    let (_, end) = ed.current_line_span();
                    ed.cur = end;
                } else {
                    ed.down_visual_select(lines);
                }
                true
            }
            'k' => {
                if self.mode == VimSubMode::VisualLine {
                    ed.up_visual_select(lines);
                    let (start, _) = ed.current_line_span();
                    ed.cur = start;
                } else {
                    ed.up_visual_select(lines);
                }
                true
            }
            'w' => {
                ed.word_right_select();
                true
            }
            'b' => {
                ed.word_left_select();
                true
            }
            '0' => {
                ed.home_visual_select(lines);
                true
            }
            '$' => {
                ed.end_visual_select(lines);
                true
            }
            // Actions on selection
            'y' => {
                if let Some(text) = ed.selected_text() {
                    self.register = text;
                    self.register_is_line = self.mode == VimSubMode::VisualLine;
                }
                self.set_mode(VimSubMode::Normal, ed);
                true
            }
            'd' | 'x' => {
                if let Some(text) = ed.selected_text() {
                    self.register = text;
                    self.register_is_line = self.mode == VimSubMode::VisualLine;
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Normal, ed);
                true
            }
            'c' => {
                if let Some(text) = ed.selected_text() {
                    self.register = text;
                    self.register_is_line = self.mode == VimSubMode::VisualLine;
                }
                ed.delete_selection();
                self.set_mode(VimSubMode::Insert, ed);
                true
            }
            'v' => {
                if self.mode == VimSubMode::Visual {
                    self.set_mode(VimSubMode::Normal, ed);
                } else {
                    self.set_mode(VimSubMode::Visual, ed);
                }
                true
            }
            'V' => {
                if self.mode == VimSubMode::VisualLine {
                    self.set_mode(VimSubMode::Normal, ed);
                } else {
                    self.set_mode(VimSubMode::VisualLine, ed);
                }
                true
            }
            _ => false,
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
    fn test_vim_delete_and_yank_line() {
        let mut ed = Editor::new();
        ed.insert_str("first line\nsecond line\nthird line");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // dd on first line
        assert!(vim.handle_char(&mut ed, &[], 'd'));
        assert_eq!(vim.pending_op, Some('d'));
        assert!(vim.handle_char(&mut ed, &[], 'd'));
        assert_eq!(vim.register, "first line\n");
        assert_eq!(ed.text(), "second line\nthird line");

        // u -> undo
        assert!(vim.handle_char(&mut ed, &[], 'u'));
        assert_eq!(ed.text(), "first line\nsecond line\nthird line");
    }

    #[test]
    fn test_vim_word_motions() {
        let mut ed = Editor::new();
        ed.insert_str("one two three");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // w -> word right
        assert!(vim.handle_char(&mut ed, &[], 'w'));
        assert_eq!(ed.cur, 4);

        // w -> word right
        assert!(vim.handle_char(&mut ed, &[], 'w'));
        assert_eq!(ed.cur, 8);

        // b -> word left
        assert!(vim.handle_char(&mut ed, &[], 'b'));
        assert_eq!(ed.cur, 4);
    }

    #[test]
    fn test_vim_visual_navigation_and_gap_selection() {
        let mut ed = Editor::new();
        // Line 0: "first" (0..5), newline at 5
        // Line 1: empty line gap, newline at 6
        // Line 2: "third" (7..12)
        ed.insert_str("first\n\nthird");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // Start visual mode
        assert!(vim.handle_char(&mut ed, &[], 'v'));
        assert_eq!(vim.mode, VimSubMode::Visual);
        assert_eq!(ed.selection, Some(0));

        // 'j' moves down to empty line gap (line 1), maintaining selection
        assert!(vim.handle_char(&mut ed, &[], 'j'));
        assert_eq!(ed.selection, Some(0));
        let (row, _) = ed.row_col();
        assert_eq!(row, 1);
        assert!(ed.has_selection());

        // 'j' moves down to line 2 ("third"), selection now spans line 0, gap line 1, and line 2
        assert!(vim.handle_char(&mut ed, &[], 'j'));
        assert_eq!(ed.selection, Some(0));
        let (row, _) = ed.row_col();
        assert_eq!(row, 2);
        assert!(ed.has_selection());

        // 'k' moves back up to empty line gap, still maintaining selection
        assert!(vim.handle_char(&mut ed, &[], 'k'));
        assert_eq!(ed.selection, Some(0));
        let (row, _) = ed.row_col();
        assert_eq!(row, 1);
        assert!(ed.has_selection());
    }

    #[test]
    fn test_vim_single_paragraph_soft_wrap_navigation() {
        let mut ed = Editor::new();
        // A single long paragraph with NO newline characters (\n)
        ed.insert_str("The quick brown fox jumps over the lazy dog and runs through the forest");
        ed.cur = 0;
        let mut vim = VimEngine::new();

        // Compute soft wrapped lines at max_cols = 16
        let lines = ed.compute_visual_lines(16);
        assert!(lines.len() >= 4);

        // Initially at visual row 0
        let (r0, _) = ed.visual_row_col(&lines);
        assert_eq!(r0, 0);

        // In Vim normal mode, pressing 'j' MUST advance to visual line 1, NOT jump over the whole paragraph!
        assert!(vim.handle_char(&mut ed, &lines, 'j'));
        let (r1, _) = ed.visual_row_col(&lines);
        assert_eq!(r1, 1);

        // Pressing 'j' again advances to visual line 2
        assert!(vim.handle_char(&mut ed, &lines, 'j'));
        let (r2, _) = ed.visual_row_col(&lines);
        assert_eq!(r2, 2);

        // Pressing 'k' moves back up to visual line 1
        assert!(vim.handle_char(&mut ed, &lines, 'k'));
        let (r3, _) = ed.visual_row_col(&lines);
        assert_eq!(r3, 1);
    }
}
