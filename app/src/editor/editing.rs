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
        if self.has_selection() {
            self.indent_line();
        } else {
            self.save_undo_snapshot();
            for _ in 0..4 {
                self.buf.insert(self.cur, ' ');
                self.cur += 1;
            }
        }
    }

    pub fn dedent(&mut self) {
        self.dedent_line();
    }

    pub fn indent_line(&mut self) {
        self.save_undo_snapshot();
        if let Some((start, end)) = self.selected_range() {
            let mut anchor = self.selection.unwrap();
            let mut cur = self.cur;

            let mut first_line_start = start;
            while first_line_start > 0 && self.buf[first_line_start - 1] != '\n' {
                first_line_start -= 1;
            }

            let mut line_starts = Vec::new();
            let mut i = first_line_start;
            while i <= self.buf.len() {
                line_starts.push(i);
                if let Some(nl) = self.buf[i..].iter().position(|&c| c == '\n') {
                    let next_start = i + nl + 1;
                    if next_start > end || (next_start == end && end > start) {
                        break;
                    }
                    i = next_start;
                } else {
                    break;
                }
            }

            line_starts.reverse();
            for ls in line_starts {
                for _ in 0..4 {
                    self.buf.insert(ls, ' ');
                }
                if anchor >= ls {
                    anchor += 4;
                }
                if cur >= ls {
                    cur += 4;
                }
            }

            self.cur = cur.min(self.buf.len());
            self.selection = Some(anchor.min(self.buf.len()));
        } else {
            let mut line_start = self.cur;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            for _ in 0..4 {
                self.buf.insert(line_start, ' ');
            }
            self.cur = (self.cur + 4).min(self.buf.len());
            self.selection = None;
        }
    }

    pub fn dedent_line(&mut self) {
        self.save_undo_snapshot();
        if let Some((start, end)) = self.selected_range() {
            let mut anchor = self.selection.unwrap();
            let mut cur = self.cur;

            let mut first_line_start = start;
            while first_line_start > 0 && self.buf[first_line_start - 1] != '\n' {
                first_line_start -= 1;
            }

            let mut line_starts = Vec::new();
            let mut i = first_line_start;
            while i <= self.buf.len() {
                line_starts.push(i);
                if let Some(nl) = self.buf[i..].iter().position(|&c| c == '\n') {
                    let next_start = i + nl + 1;
                    if next_start > end || (next_start == end && end > start) {
                        break;
                    }
                    i = next_start;
                } else {
                    break;
                }
            }

            line_starts.reverse();
            for ls in line_starts {
                let mut spaces = 0;
                while spaces < 4 && ls + spaces < self.buf.len() && self.buf[ls + spaces] == ' ' {
                    spaces += 1;
                }
                if spaces > 0 {
                    for _ in 0..spaces {
                        self.buf.remove(ls);
                    }
                    if anchor >= ls + spaces {
                        anchor -= spaces;
                    } else if anchor > ls {
                        anchor = ls;
                    }

                    if cur >= ls + spaces {
                        cur -= spaces;
                    } else if cur > ls {
                        cur = ls;
                    }
                }
            }

            self.cur = cur.min(self.buf.len());
            self.selection = if anchor != self.cur {
                Some(anchor.min(self.buf.len()))
            } else {
                None
            };
        } else {
            let mut line_start = self.cur;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            let mut spaces = 0;
            while spaces < 4 && line_start + spaces < self.buf.len() && self.buf[line_start + spaces] == ' ' {
                spaces += 1;
            }
            if spaces > 0 {
                for _ in 0..spaces {
                    self.buf.remove(line_start);
                }
                if self.cur >= line_start + spaces {
                    self.cur -= spaces;
                } else if self.cur > line_start {
                    self.cur = line_start;
                }
            }
            self.selection = None;
        }
    }
}
