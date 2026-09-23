//! In-buffer search engine for Vim `/` (forward) and `?` (backward) search.
//!
//! Provides live-as-you-type match highlighting, wrapping navigation (`n`/`N`),
//! and safe cancel-on-Escape restoration.

use crate::editor::Editor;

/// State machine for Vim in-buffer search.
#[derive(Debug, Clone, Default)]
pub struct VimSearchState {
    /// Whether an interactive search session is actively taking input.
    pub active: bool,
    /// Currently typed search query string.
    pub query: String,
    /// Whether the active search is backward (`?`) or forward (`/`).
    pub backward: bool,
    /// The last confirmed search query, persisted for `n` and `N` repetition.
    pub last_query: String,
    /// Direction of the last confirmed search.
    pub last_backward: bool,
    /// Cursor position saved before search opened, restored if user cancels with Escape.
    pub saved_cursor: usize,
    /// All starting char indices in the buffer where `query` matches.
    pub match_indices: Vec<usize>,
}

impl VimSearchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts an interactive search session.
    #[allow(dead_code)]
    pub fn start(&mut self, backward: bool, initial_cur: usize) {
        self.active = true;
        self.backward = backward;
        self.query.clear();
        self.saved_cursor = initial_cur;
        self.match_indices.clear();
    }

    /// Appends a character to the query, recalculates matches, and live-jumps to nearest match.
    pub fn push_char(&mut self, c: char, ed: &mut Editor) {
        self.query.push(c);
        self.recompute_matches(&ed.buf);
        self.jump_to_nearest_match(ed);
    }

    /// Removes the last character from query, recalculating matches.
    /// Returns `true` if search session is still active, or `false` if query became empty
    /// and caller should cancel.
    pub fn pop_char(&mut self, ed: &mut Editor) -> bool {
        if self.query.pop().is_some() {
            if self.query.is_empty() {
                self.match_indices.clear();
                ed.cur = self.saved_cursor;
            } else {
                self.recompute_matches(&ed.buf);
                self.jump_to_nearest_match(ed);
            }
            true
        } else {
            self.cancel(ed);
            false
        }
    }

    /// Cancels search session and restores cursor to position before search started.
    pub fn cancel(&mut self, ed: &mut Editor) {
        self.active = false;
        self.query.clear();
        self.match_indices.clear();
        ed.cur = self.saved_cursor.min(ed.buf.len());
    }

    /// Clears active match highlighting (e.g. on Escape or `:noh`).
    pub fn clear_matches(&mut self) {
        self.match_indices.clear();
    }

    /// Confirms current search query, commits it to `last_query`, and ends search input mode.
    /// Returns `(current_match_idx_1_based, total_matches)` if matches exist.
    pub fn confirm(&mut self, ed: &mut Editor) -> Option<(usize, usize)> {
        self.active = false;
        if self.query.is_empty() {
            return None;
        }

        self.last_query = self.query.clone();
        self.last_backward = self.backward;
        self.recompute_matches(&ed.buf);

        if self.match_indices.is_empty() {
            return None;
        }

        // Find which match index cursor is currently on
        let cur_pos = ed.cur;
        let match_num = self
            .match_indices
            .iter()
            .position(|&idx| idx == cur_pos)
            .unwrap_or(0)
            + 1;

        Some((match_num, self.match_indices.len()))
    }

    /// Recomputes all starting char indices where `self.query` matches within `buf`.
    pub fn recompute_matches(&mut self, buf: &[char]) {
        self.match_indices.clear();
        let q = self.query.to_lowercase();
        if q.is_empty() || buf.is_empty() {
            return;
        }

        let q_chars: Vec<char> = q.chars().collect();
        let q_len = q_chars.len();
        if q_len > buf.len() {
            return;
        }

        for i in 0..=(buf.len() - q_len) {
            let mut matches = true;
            for j in 0..q_len {
                if buf[i + j].to_lowercase().next() != Some(q_chars[j]) {
                    matches = false;
                    break;
                }
            }
            if matches {
                self.match_indices.push(i);
            }
        }
    }

    /// Jumps the editor cursor to the closest match in search direction from `saved_cursor`.
    fn jump_to_nearest_match(&self, ed: &mut Editor) {
        if self.match_indices.is_empty() {
            return;
        }

        if self.backward {
            // Find closest match before or at saved_cursor, or wrap to last match
            if let Some(&m) = self.match_indices.iter().rev().find(|&&idx| idx <= self.saved_cursor) {
                ed.cur = m;
            } else if let Some(&m) = self.match_indices.last() {
                ed.cur = m;
            }
        } else {
            // Find closest match after or at saved_cursor, or wrap to first match
            if let Some(&m) = self.match_indices.iter().find(|&&idx| idx >= self.saved_cursor) {
                ed.cur = m;
            } else if let Some(&m) = self.match_indices.first() {
                ed.cur = m;
            }
        }
    }

    /// Executes repeat search (`n` or `N`).
    /// - `reverse`: if true, flips the base search direction.
    /// Returns `Option<(match_num_1_based, total_matches)>`.
    pub fn repeat_search(&mut self, ed: &mut Editor, reverse: bool) -> Option<(usize, usize)> {
        if self.last_query.is_empty() {
            return None;
        }

        self.query = self.last_query.clone();
        self.recompute_matches(&ed.buf);

        if self.match_indices.is_empty() {
            return None;
        }

        let go_backward = if reverse {
            !self.last_backward
        } else {
            self.last_backward
        };

        let cur_pos = ed.cur;

        if go_backward {
            // Find strictly preceding match, or wrap to last
            let target = self
                .match_indices
                .iter()
                .rev()
                .find(|&&idx| idx < cur_pos)
                .copied()
                .unwrap_or_else(|| *self.match_indices.last().unwrap());

            ed.cur = target;
        } else {
            // Find strictly succeeding match, or wrap to first
            let target = self
                .match_indices
                .iter()
                .find(|&&idx| idx > cur_pos)
                .copied()
                .unwrap_or_else(|| *self.match_indices.first().unwrap());

            ed.cur = target;
        }

        let match_num = self
            .match_indices
            .iter()
            .position(|&idx| idx == ed.cur)
            .unwrap_or(0)
            + 1;

        Some((match_num, self.match_indices.len()))
    }
}
