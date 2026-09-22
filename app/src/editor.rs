#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorSnapshot {
    pub buf: Vec<char>,
    pub cur: usize,
}

#[derive(Default, Clone)]
pub struct Editor {
    pub buf: Vec<char>, // chars, not bytes: safe with Unicode, easy cursor math
    pub cur: usize,     // cursor position as char index
    pub selection: Option<usize>, // selection anchor
    pub undo_stack: Vec<EditorSnapshot>,
    pub redo_stack: Vec<EditorSnapshot>,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Saves the current editor state into the undo history.
    pub fn save_undo_snapshot(&mut self) {
        let snap = EditorSnapshot {
            buf: self.buf.clone(),
            cur: self.cur,
        };
        if self.undo_stack.last() != Some(&snap) {
            self.undo_stack.push(snap);
            if self.undo_stack.len() > 300 {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
    }

    /// Undos the last text modification.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(EditorSnapshot {
                buf: self.buf.clone(),
                cur: self.cur,
            });
            self.buf = prev.buf;
            self.cur = prev.cur.min(self.buf.len());
            self.selection = None;
            true
        } else {
            false
        }
    }

    /// Redos the previously undone modification.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(EditorSnapshot {
                buf: self.buf.clone(),
                cur: self.cur,
            });
            self.buf = next.buf;
            self.cur = next.cur.min(self.buf.len());
            self.selection = None;
            true
        } else {
            false
        }
    }

    /// Returns the active selection as `(start, end)` char indices, if any.
    pub fn selected_range(&self) -> Option<(usize, usize)> {
        if let Some(anchor) = self.selection {
            if anchor != self.cur {
                return Some((self.cur.min(anchor), self.cur.max(anchor)));
            }
        }
        None
    }

    pub fn has_selection(&self) -> bool {
        self.selected_range().is_some()
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn select_all(&mut self) {
        if !self.buf.is_empty() {
            self.selection = Some(0);
            self.cur = self.buf.len();
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selected_range().map(|(s, e)| self.buf[s..e].iter().collect())
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((s, e)) = self.selected_range() {
            self.save_undo_snapshot();
            self.buf.drain(s..e);
            self.cur = s;
            self.selection = None;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, c: char) {
        if self.has_selection() {
            self.delete_selection();
        } else if c.is_whitespace() || self.undo_stack.is_empty() {
            self.save_undo_snapshot();
        }
        self.buf.insert(self.cur, c);
        self.cur += 1;
        self.selection = None;
    }

    #[allow(dead_code)]
    pub fn insert_str(&mut self, s: &str) {
        if self.has_selection() {
            self.delete_selection();
        }
        self.save_undo_snapshot();
        for c in s.chars() {
            self.buf.insert(self.cur, c);
            self.cur += 1;
        }
        self.selection = None;
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cur > 0 {
            self.save_undo_snapshot();
            self.cur -= 1;
            self.buf.remove(self.cur);
            self.selection = None;
        }
    }

    #[allow(dead_code)]
    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cur < self.buf.len() {
            self.save_undo_snapshot();
            self.buf.remove(self.cur);
            self.selection = None;
        }
    }

    pub fn left(&mut self) {
        if let Some((start, _)) = self.selected_range() {
            self.cur = start;
            self.selection = None;
        } else {
            self.cur = self.cur.saturating_sub(1);
            self.selection = None;
        }
    }

    pub fn right(&mut self) {
        if let Some((_, end)) = self.selected_range() {
            self.cur = end;
            self.selection = None;
        } else {
            self.cur = (self.cur + 1).min(self.buf.len());
            self.selection = None;
        }
    }

    pub fn left_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        self.cur = self.cur.saturating_sub(1);
    }

    pub fn right_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        self.cur = (self.cur + 1).min(self.buf.len());
    }

    /// Moves cursor to start of current line
    #[allow(dead_code)]
    pub fn home(&mut self) {
        self.selection = None;
        while self.cur > 0 && self.buf[self.cur - 1] != '\n' {
            self.cur -= 1;
        }
    }

    /// Moves cursor to end of current line
    #[allow(dead_code)]
    pub fn end(&mut self) {
        self.selection = None;
        while self.cur < self.buf.len() && self.buf[self.cur] != '\n' {
            self.cur += 1;
        }
    }

    /// Ctrl+Enter: Inserts a new line below the current line and moves the cursor there,
    /// without splitting the current line even if cursor is at the beginning or middle.
    pub fn insert_line_below(&mut self) {
        self.end();
        self.insert('\n');
    }

    /// Moves cursor one line up, preserving column if possible
    #[allow(dead_code)]
    pub fn up(&mut self) {
        self.selection = None;
        let (row, col) = self.row_col();
        if row == 0 {
            return;
        }
        let target_row = row - 1;
        self.set_row_col(target_row, col);
    }

    /// Moves cursor one line up, expanding or creating selection
    pub fn up_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.row_col();
        if row == 0 {
            return;
        }
        let target_row = row - 1;
        self.set_row_col(target_row, col);
    }

    /// Moves cursor one line down, preserving column if possible
    #[allow(dead_code)]
    pub fn down(&mut self) {
        self.selection = None;
        let (row, col) = self.row_col();
        let target_row = row + 1;
        self.set_row_col(target_row, col);
    }

    /// Moves cursor one line down, expanding or creating selection
    pub fn down_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.row_col();
        let target_row = row + 1;
        self.set_row_col(target_row, col);
    }

    pub fn set_row_col(&mut self, target_row: usize, target_col: usize) {
        let mut cur_row = 0;
        let mut line_start = 0;
        for (i, &c) in self.buf.iter().enumerate() {
            if cur_row == target_row {
                // Count columns in this line
                let mut col = 0;
                let mut idx = line_start;
                while idx < self.buf.len() && self.buf[idx] != '\n' && col < target_col {
                    idx += 1;
                    col += 1;
                }
                self.cur = idx;
                return;
            }
            if c == '\n' {
                cur_row += 1;
                line_start = i + 1;
            }
        }
        if cur_row == target_row {
            let mut col = 0;
            let mut idx = line_start;
            while idx < self.buf.len() && self.buf[idx] != '\n' && col < target_col {
                idx += 1;
                col += 1;
            }
            self.cur = idx;
        }
    }

    /// Ctrl+Backspace: delete spaces, then the word before them
    pub fn delete_word(&mut self) {
        while self.cur > 0 && self.buf[self.cur - 1].is_whitespace() {
            self.backspace();
        }
        while self.cur > 0 && !self.buf[self.cur - 1].is_whitespace() {
            self.backspace();
        }
    }

    /// Returns previous word boundary index before `idx`.
    pub fn prev_word_boundary(&self, idx: usize) -> usize {
        let mut i = idx.min(self.buf.len());
        while i > 0 && self.buf[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !self.buf[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    }

    /// Returns next word boundary index after `idx`.
    pub fn next_word_boundary(&self, idx: usize) -> usize {
        let mut i = idx.min(self.buf.len());
        while i < self.buf.len() && !self.buf[i].is_whitespace() {
            i += 1;
        }
        while i < self.buf.len() && self.buf[i].is_whitespace() {
            i += 1;
        }
        i
    }

    pub fn word_left(&mut self) {
        self.cur = self.prev_word_boundary(self.cur);
        self.selection = None;
    }

    pub fn word_right(&mut self) {
        self.cur = self.next_word_boundary(self.cur);
        self.selection = None;
    }

    pub fn word_left_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        self.cur = self.prev_word_boundary(self.cur);
    }

    pub fn word_right_select(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        self.cur = self.next_word_boundary(self.cur);
    }

    /// Ctrl+Delete: delete characters to the next word boundary
    pub fn delete_word_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cur < self.buf.len() {
            let next = self.next_word_boundary(self.cur);
            if next > self.cur {
                self.save_undo_snapshot();
                self.buf.drain(self.cur..next);
                self.selection = None;
            }
        }
    }

    /// Returns the start and end (including trailing newline) indices of the current line.
    pub fn current_line_span(&self) -> (usize, usize) {
        if self.buf.is_empty() {
            return (0, 0);
        }
        let cur = self.cur.min(self.buf.len());
        let mut start = cur;
        while start > 0 && self.buf[start - 1] != '\n' {
            start -= 1;
        }
        let mut end = cur;
        while end < self.buf.len() && self.buf[end] != '\n' {
            end += 1;
        }
        if end < self.buf.len() && self.buf[end] == '\n' {
            end += 1;
        }
        (start, end)
    }

    /// Duplicate the current line directly below
    pub fn duplicate_line(&mut self) {
        self.save_undo_snapshot();
        let (start, end) = self.current_line_span();
        let line_chars: Vec<char> = self.buf[start..end].to_vec();
        let has_newline = line_chars.last() == Some(&'\n');

        let insert_pos = end;
        if !has_newline {
            self.buf.insert(insert_pos, '\n');
            let mut next_pos = insert_pos + 1;
            for c in line_chars {
                self.buf.insert(next_pos, c);
                next_pos += 1;
            }
            self.cur = next_pos;
        } else {
            let mut next_pos = insert_pos;
            for c in line_chars {
                self.buf.insert(next_pos, c);
                next_pos += 1;
            }
            self.cur = next_pos.saturating_sub(1);
        }
        self.selection = None;
    }

    /// Move current line up (Alt+Up)
    pub fn move_line_up(&mut self) {
        let (cur_start, cur_end) = self.current_line_span();
        if cur_start == 0 {
            return;
        }
        self.save_undo_snapshot();
        let prev_end = cur_start;
        let mut prev_start = prev_end.saturating_sub(1);
        while prev_start > 0 && self.buf[prev_start - 1] != '\n' {
            prev_start -= 1;
        }
        let offset = self.cur.saturating_sub(cur_start);

        let cur_line: Vec<char> = self.buf.drain(cur_start..cur_end).collect();
        let mut insert_idx = prev_start;
        for c in cur_line {
            self.buf.insert(insert_idx, c);
            insert_idx += 1;
        }
        self.cur = (prev_start + offset).min(self.buf.len());
        self.selection = None;
    }

    /// Move current line down (Alt+Down)
    pub fn move_line_down(&mut self) {
        let (cur_start, cur_end) = self.current_line_span();
        if cur_end >= self.buf.len() {
            return;
        }
        self.save_undo_snapshot();
        let next_start = cur_end;
        let mut next_end = next_start;
        while next_end < self.buf.len() && self.buf[next_end] != '\n' {
            next_end += 1;
        }
        if next_end < self.buf.len() && self.buf[next_end] == '\n' {
            next_end += 1;
        }
        let offset = self.cur.saturating_sub(cur_start);

        let next_line: Vec<char> = self.buf.drain(next_start..next_end).collect();
        let mut insert_idx = cur_start;
        for c in next_line {
            self.buf.insert(insert_idx, c);
            insert_idx += 1;
        }
        self.cur = (insert_idx + offset).min(self.buf.len());
        self.selection = None;
    }

    /// Deletes current line (for dd or line cut) and returns deleted string
    pub fn delete_line(&mut self) -> String {
        self.save_undo_snapshot();
        let (start, end) = self.current_line_span();
        let line_str: String = self.buf[start..end].iter().collect();
        self.buf.drain(start..end);
        self.cur = start.min(self.buf.len());
        self.selection = None;
        line_str
    }

    /// Yanks (copies) current line
    pub fn yank_line(&self) -> String {
        let (start, end) = self.current_line_span();
        self.buf[start..end].iter().collect()
    }

    /// Smart Tab / Indent (4 spaces)
    pub fn indent(&mut self) {
        if let Some((start, end)) = self.selected_range() {
            self.save_undo_snapshot();
            let mut line_start = start;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            let mut i = line_start;
            let mut added = 0;
            while i <= end + added && i <= self.buf.len() {
                if i == 0 || (i > 0 && i <= self.buf.len() && self.buf[i - 1] == '\n') {
                    for _ in 0..4 {
                        self.buf.insert(i, ' ');
                        added += 1;
                    }
                    i += 4;
                }
                i += 1;
            }
            self.cur = (self.cur + 4).min(self.buf.len());
        } else {
            self.save_undo_snapshot();
            for _ in 0..4 {
                self.buf.insert(self.cur, ' ');
                self.cur += 1;
            }
        }
    }

    /// Smart Shift+Tab / Dedent (removes up to 4 leading spaces)
    pub fn dedent(&mut self) {
        self.save_undo_snapshot();
        let mut line_start = self.cur;
        while line_start > 0 && self.buf[line_start - 1] != '\n' {
            line_start -= 1;
        }
        let mut removed = 0;
        while removed < 4 && line_start < self.buf.len() && self.buf[line_start] == ' ' {
            self.buf.remove(line_start);
            removed += 1;
        }
        self.cur = self.cur.saturating_sub(removed);
        self.selection = None;
    }

    /// Auto-pairing / wrapping for quotes and brackets
    pub fn auto_pair(&mut self, open: char, close: char) {
        if let Some((start, end)) = self.selected_range() {
            self.save_undo_snapshot();
            let selected: Vec<char> = self.buf.drain(start..end).collect();
            self.buf.insert(start, open);
            let mut p = start + 1;
            for c in selected {
                self.buf.insert(p, c);
                p += 1;
            }
            self.buf.insert(p, close);
            self.cur = p + 1;
            self.selection = None;
        } else {
            self.save_undo_snapshot();
            self.buf.insert(self.cur, open);
            self.buf.insert(self.cur + 1, close);
            self.cur += 1;
            self.selection = None;
        }
    }

    pub fn clear(&mut self) {
        if !self.buf.is_empty() {
            self.save_undo_snapshot();
        }
        self.buf.clear();
        self.cur = 0;
        self.selection = None;
    }

    pub fn set_text(&mut self, s: &str) {
        self.buf = s.chars().collect();
        self.cur = 0;
        self.selection = None;
    }

    pub fn text(&self) -> String {
        self.buf.iter().collect()
    }

    pub fn row_col(&self) -> (usize, usize) {
        self.row_col_of(self.cur)
    }

    pub fn row_col_of(&self, idx: usize) -> (usize, usize) {
        let (mut row, mut col) = (0, 0);
        let end = idx.min(self.buf.len());
        for &c in &self.buf[..end] {
            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (row, col)
    }

    /// Computes soft-wrapped visual lines for smooth rendering and multi-line navigation.
    pub fn compute_visual_lines(&self, max_cols: usize) -> Vec<VisualLine> {
        if self.buf.is_empty() {
            return vec![VisualLine { char_start: 0, char_end: 0 }];
        }

        let max_cols = max_cols.max(15);
        let mut lines = Vec::new();
        let mut line_start = 0;

        while line_start <= self.buf.len() {
            let mut line_end = line_start;
            while line_end < self.buf.len() && self.buf[line_end] != '\n' {
                line_end += 1;
            }

            let physical_len = line_end - line_start;
            if physical_len == 0 {
                lines.push(VisualLine {
                    char_start: line_start,
                    char_end: line_end,
                });
            } else {
                let mut chunk_start = line_start;
                while chunk_start < line_end {
                    let remaining = line_end - chunk_start;
                    if remaining <= max_cols {
                        lines.push(VisualLine {
                            char_start: chunk_start,
                            char_end: line_end,
                        });
                        break;
                    }

                    // Look for whitespace wrap point within max_cols
                    let limit = chunk_start + max_cols;
                    let mut wrap_at = None;
                    for idx in (chunk_start + 1..=limit).rev() {
                        if self.buf[idx - 1].is_whitespace() {
                            wrap_at = Some(idx);
                            break;
                        }
                    }

                    let chunk_end = wrap_at.unwrap_or(limit);
                    lines.push(VisualLine {
                        char_start: chunk_start,
                        char_end: chunk_end,
                    });

                    chunk_start = chunk_end;
                    while chunk_start < line_end && self.buf[chunk_start] == ' ' {
                        chunk_start += 1;
                    }
                }
            }

            if line_end >= self.buf.len() {
                if line_end > 0 && self.buf[line_end - 1] == '\n' {
                    lines.push(VisualLine {
                        char_start: line_end,
                        char_end: line_end,
                    });
                }
                break;
            }
            line_start = line_end + 1;
        }

        if lines.is_empty() {
            lines.push(VisualLine { char_start: 0, char_end: 0 });
        }

        lines
    }

    /// Finds the visual row and column of a given character index.
    pub fn visual_row_col(&self, lines: &[VisualLine]) -> (usize, usize) {
        if lines.is_empty() {
            return (0, 0);
        }
        for (i, line) in lines.iter().enumerate() {
            if self.cur >= line.char_start && self.cur < line.char_end {
                return (i, self.cur - line.char_start);
            }
            if self.cur == line.char_end {
                if i + 1 == lines.len() || lines[i + 1].char_start > line.char_end {
                    return (i, self.cur - line.char_start);
                }
            }
            if i + 1 < lines.len() && self.cur >= line.char_end && self.cur < lines[i + 1].char_start {
                return (i, self.cur - line.char_start);
            }
        }
        let last_idx = lines.len() - 1;
        let last = &lines[last_idx];
        (last_idx, self.cur.saturating_sub(last.char_start))
    }

    pub fn up_visual(&mut self, lines: &[VisualLine]) {
        if lines.is_empty() {
            self.up();
            return;
        }
        self.selection = None;
        let (row, col) = self.visual_row_col(lines);
        if row == 0 {
            return;
        }
        let target_line = &lines[row - 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn up_visual_select(&mut self, lines: &[VisualLine]) {
        if lines.is_empty() {
            self.up_select();
            return;
        }
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.visual_row_col(lines);
        if row == 0 {
            return;
        }
        let target_line = &lines[row - 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn down_visual(&mut self, lines: &[VisualLine]) {
        if lines.is_empty() {
            self.down();
            return;
        }
        self.selection = None;
        let (row, col) = self.visual_row_col(lines);
        if row + 1 >= lines.len() {
            return;
        }
        let target_line = &lines[row + 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn down_visual_select(&mut self, lines: &[VisualLine]) {
        if lines.is_empty() {
            self.down_select();
            return;
        }
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.visual_row_col(lines);
        if row + 1 >= lines.len() {
            return;
        }
        let target_line = &lines[row + 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn home_visual(&mut self, lines: &[VisualLine]) {
        self.selection = None;
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            self.cur = line.char_start;
        }
    }

    pub fn home_visual_select(&mut self, lines: &[VisualLine]) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            self.cur = line.char_start;
        }
    }

    pub fn end_visual(&mut self, lines: &[VisualLine]) {
        self.selection = None;
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            if line.char_end > line.char_start {
                let mut target = line.char_end - 1;
                while target > line.char_start && self.buf[target].is_whitespace() {
                    target -= 1;
                }
                self.cur = target;
            } else {
                self.cur = line.char_start;
            }
        }
    }

    pub fn end_visual_select(&mut self, lines: &[VisualLine]) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            self.cur = line.char_end;
        }
    }

    pub fn page_up_visual(&mut self, lines: &[VisualLine], count: usize) {
        self.selection = None;
        let (row, col) = self.visual_row_col(lines);
        let target_row = row.saturating_sub(count);
        if let Some(target_line) = lines.get(target_row) {
            let line_len = target_line.char_end.saturating_sub(target_line.char_start);
            self.cur = target_line.char_start + col.min(line_len);
        }
    }

    pub fn page_up_visual_select(&mut self, lines: &[VisualLine], count: usize) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.visual_row_col(lines);
        let target_row = row.saturating_sub(count);
        if let Some(target_line) = lines.get(target_row) {
            let line_len = target_line.char_end.saturating_sub(target_line.char_start);
            self.cur = target_line.char_start + col.min(line_len);
        }
    }

    pub fn page_down_visual(&mut self, lines: &[VisualLine], count: usize) {
        self.selection = None;
        let (row, col) = self.visual_row_col(lines);
        let target_row = (row + count).min(lines.len().saturating_sub(1));
        if let Some(target_line) = lines.get(target_row) {
            let line_len = target_line.char_end.saturating_sub(target_line.char_start);
            self.cur = target_line.char_start + col.min(line_len);
        }
    }

    pub fn page_down_visual_select(&mut self, lines: &[VisualLine], count: usize) {
        if self.selection.is_none() {
            self.selection = Some(self.cur);
        }
        let (row, col) = self.visual_row_col(lines);
        let target_row = (row + count).min(lines.len().saturating_sub(1));
        if let Some(target_line) = lines.get(target_row) {
            let line_len = target_line.char_end.saturating_sub(target_line.char_start);
            self.cur = target_line.char_start + col.min(line_len);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualLine {
    pub char_start: usize,
    pub char_end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_typing_and_backspace() {
        let mut ed = Editor::new();
        ed.insert_str("hello world");
        assert_eq!(ed.text(), "hello world");
        assert_eq!(ed.cur, 11);

        ed.delete_word();
        assert_eq!(ed.text(), "hello ");

        ed.backspace();
        assert_eq!(ed.text(), "hello");
    }

    #[test]
    fn test_row_col_calculations() {
        let mut ed = Editor::new();
        ed.insert_str("abc\ndefgh\nijk");
        assert_eq!(ed.row_col(), (2, 3));

        ed.cur = 5; // 'e' in "defgh"
        assert_eq!(ed.row_col(), (1, 1));
    }

    #[test]
    fn test_insert_line_below() {
        let mut ed = Editor::new();
        ed.insert_str("first line\nsecond line");
        ed.cur = 2; // in the middle of "first line" ("fi|rst line")
        ed.insert_line_below();
        assert_eq!(ed.text(), "first line\n\nsecond line");
    }

    #[test]
    fn test_visual_line_wrapping() {
        let mut ed = Editor::new();
        ed.insert_str("The quick brown fox jumps over the lazy dog");
        // max_cols = 16
        let lines = ed.compute_visual_lines(16);
        assert!(lines.len() >= 3);
        let first_line: String = ed.buf[lines[0].char_start..lines[0].char_end].iter().collect();
        assert_eq!(first_line, "The quick brown ");

        ed.cur = 0;
        let (r0, c0) = ed.visual_row_col(&lines);
        assert_eq!((r0, c0), (0, 0));

        // Move visual down
        ed.down_visual(&lines);
        let (r1, _) = ed.visual_row_col(&lines);
        assert_eq!(r1, 1);

        // Move visual up
        ed.up_visual(&lines);
        let (r_back, _) = ed.visual_row_col(&lines);
        assert_eq!(r_back, 0);
    }

    #[test]
    fn test_editor_undo_redo() {
        let mut ed = Editor::new();
        ed.insert_str("First");
        ed.insert_str(" Second");
        assert_eq!(ed.text(), "First Second");

        // Undo
        assert!(ed.undo());
        assert_eq!(ed.text(), "First");

        // Redo
        assert!(ed.redo());
        assert_eq!(ed.text(), "First Second");

        // Clear and undo
        ed.clear();
        assert_eq!(ed.text(), "");
        assert!(ed.undo());
        assert_eq!(ed.text(), "First Second");
    }

    #[test]
    fn test_editor_selection_and_replace() {
        let mut ed = Editor::new();
        ed.insert_str("Hello beautiful world");
        // Select "beautiful "
        ed.cur = 6;
        ed.selection = Some(16);
        assert_eq!(ed.selected_text(), Some("beautiful ".to_string()));

        // Typing replaces selection
        ed.insert_str("brave ");
        assert_eq!(ed.text(), "Hello brave world");

        // Select all and delete
        ed.select_all();
        assert!(ed.delete_selection());
        assert_eq!(ed.text(), "");

        // Undo brings it back
        assert!(ed.undo());
        assert_eq!(ed.text(), "Hello brave world");
    }

    #[test]
    fn test_end_visual_paragraph_line() {
        let mut ed = Editor::new();
        ed.insert_str("The quick brown fox jumps over the lazy dog and runs away");
        let lines = ed.compute_visual_lines(20);
        assert!(lines.len() > 1);

        // Start at col 0 of line 0
        ed.cur = 0;
        let (row_before, _) = ed.visual_row_col(&lines);
        assert_eq!(row_before, 0);

        // Press '$' (end_visual)
        ed.end_visual(&lines);
        let (row_after, col_after) = ed.visual_row_col(&lines);

        // Must stay strictly on line 0, not jump to line 1!
        assert_eq!(row_after, 0);
        assert!(col_after > 0);
        assert_eq!(ed.buf[ed.cur], 'x');
    }
}
