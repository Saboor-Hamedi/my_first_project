use super::Editor;

impl Editor {
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

    pub fn delete_word(&mut self) {
        while self.cur > 0 && self.buf[self.cur - 1].is_whitespace() {
            self.backspace();
        }
        while self.cur > 0 && !self.buf[self.cur - 1].is_whitespace() {
            self.backspace();
        }
    }

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

    pub fn indent_line(&mut self) {
        self.save_undo_snapshot();
        if let Some((start, end)) = self.selected_range() {
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
            let mut line_start = self.cur;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            for _ in 0..4 {
                self.buf.insert(line_start, ' ');
            }
            self.cur = (self.cur + 4).min(self.buf.len());
        }
    }

    pub fn dedent_line(&mut self) {
        self.save_undo_snapshot();
        if let Some((start, end)) = self.selected_range() {
            let mut line_start = start;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            let mut i = line_start;
            let mut removed_total = 0;
            while i < self.buf.len() && i <= end.saturating_sub(removed_total) {
                if i == 0 || self.buf[i - 1] == '\n' {
                    let mut count = 0;
                    while count < 4 && i < self.buf.len() && self.buf[i] == ' ' {
                        self.buf.remove(i);
                        count += 1;
                        removed_total += 1;
                    }
                }
                i += 1;
            }
            self.cur = self.cur.saturating_sub(4);
        } else {
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
    }
}
