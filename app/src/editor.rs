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
    pub fn home(&mut self) {
        while self.cur > 0 && self.buf[self.cur - 1] != '\n' {
            self.cur -= 1;
        }
    }

    /// Moves cursor to end of current line
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
    pub fn up(&mut self) {
        let (row, col) = self.row_col();
        if row == 0 {
            return;
        }
        let target_row = row - 1;
        self.set_row_col(target_row, col);
    }

    /// Moves cursor one line down, preserving column if possible
    pub fn down(&mut self) {
        let (row, col) = self.row_col();
        let target_row = row + 1;
        self.set_row_col(target_row, col);
    }

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

    #[allow(dead_code)]
    pub fn set_text(&mut self, s: &str) {
        self.buf = s.chars().collect();
        self.cur = self.buf.len();
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
}
