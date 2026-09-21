#[derive(Default, Clone)]
pub struct Editor {
    pub buf: Vec<char>, // chars, not bytes: safe with Unicode, easy cursor math
    pub cur: usize,     // cursor position as char index
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, c: char) {
        self.buf.insert(self.cur, c);
        self.cur += 1;
    }

    #[allow(dead_code)]
    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.cur > 0 {
            self.cur -= 1;
            self.buf.remove(self.cur);
        }
    }

    #[allow(dead_code)]
    pub fn delete(&mut self) {
        if self.cur < self.buf.len() {
            self.buf.remove(self.cur);
        }
    }

    pub fn left(&mut self) {
        self.cur = self.cur.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cur = (self.cur + 1).min(self.buf.len());
    }

    /// Moves cursor to start of current line
    #[allow(dead_code)]
    pub fn home(&mut self) {
        while self.cur > 0 && self.buf[self.cur - 1] != '\n' {
            self.cur -= 1;
        }
    }

    /// Moves cursor to end of current line
    #[allow(dead_code)]
    pub fn end(&mut self) {
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
        let (row, col) = self.row_col();
        let target_row = row + 1;
        self.set_row_col(target_row, col);
    }

    #[allow(dead_code)]
    fn set_row_col(&mut self, target_row: usize, target_col: usize) {
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

    pub fn clear(&mut self) {
        self.buf.clear();
        self.cur = 0;
    }

    pub fn set_text(&mut self, s: &str) {
        self.buf = s.chars().collect();
        self.cur = 0;
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
        let (row, col) = self.visual_row_col(lines);
        if row == 0 {
            return;
        }
        let target_line = &lines[row - 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn down_visual(&mut self, lines: &[VisualLine]) {
        let (row, col) = self.visual_row_col(lines);
        if row + 1 >= lines.len() {
            return;
        }
        let target_line = &lines[row + 1];
        let line_len = target_line.char_end.saturating_sub(target_line.char_start);
        self.cur = target_line.char_start + col.min(line_len);
    }

    pub fn home_visual(&mut self, lines: &[VisualLine]) {
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            self.cur = line.char_start;
        }
    }

    pub fn end_visual(&mut self, lines: &[VisualLine]) {
        let (row, _) = self.visual_row_col(lines);
        if let Some(line) = lines.get(row) {
            self.cur = line.char_end;
        }
    }

    pub fn page_up_visual(&mut self, lines: &[VisualLine], count: usize) {
        let (row, col) = self.visual_row_col(lines);
        let target_row = row.saturating_sub(count);
        if let Some(target_line) = lines.get(target_row) {
            let line_len = target_line.char_end.saturating_sub(target_line.char_start);
            self.cur = target_line.char_start + col.min(line_len);
        }
    }

    pub fn page_down_visual(&mut self, lines: &[VisualLine], count: usize) {
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
}
