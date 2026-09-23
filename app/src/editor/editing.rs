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

            // Find current line start up to cursor
            let mut line_start = self.cur;
            while line_start > 0 && self.buf[line_start - 1] != '\n' {
                line_start -= 1;
            }
            let line_to_cur: String = self.buf[line_start..self.cur].iter().collect();

            // 1. If line_to_cur is only leading whitespace (e.g. 4 spaces or 2 spaces indent):
            // Dedent by up to 4 spaces (soft tab stop)
            if !line_to_cur.is_empty() && line_to_cur.chars().all(|c| c == ' ') {
                let count = line_to_cur.len();
                let to_delete = if count % 4 == 0 { 4 } else { count % 4 };
                for _ in 0..to_delete {
                    self.cur -= 1;
                    self.buf.remove(self.cur);
                }
                self.selection = None;
                return;
            }

            // 2. If line_to_cur is an empty blockquote prefix (e.g. ">>> ", ">> ", "> "):
            // Dedent quote depth by one level or clear it completely
            let trimmed_quote = line_to_cur.trim_start();
            if trimmed_quote.starts_with('>') && trimmed_quote.trim_end().chars().all(|c| c == '>') {
                let quote_chars_count = trimmed_quote.chars().filter(|&c| c == '>').count();
                if quote_chars_count <= 1 {
                    while self.cur > line_start {
                        self.cur -= 1;
                        self.buf.remove(self.cur);
                    }
                } else {
                    let has_trailing_space = line_to_cur.ends_with(' ');
                    while self.cur > line_start {
                        self.cur -= 1;
                        self.buf.remove(self.cur);
                    }
                    let indent_len = line_to_cur.chars().take_while(|&c| c == ' ' || c == '\t').count();
                    let new_prefix = format!(
                        "{}{}{}",
                        &line_to_cur[..indent_len],
                        ">".repeat(quote_chars_count - 1),
                        if has_trailing_space { " " } else { "" }
                    );
                    for c in new_prefix.chars() {
                        self.buf.insert(self.cur, c);
                        self.cur += 1;
                    }
                }
                self.selection = None;
                return;
            }

            // 3. If line_to_cur is an empty list or task item prefix:
            // Clear the prefix completely
            let trimmed = line_to_cur.trim_start();
            if trimmed == "- " || trimmed == "* " || trimmed == "+ "
                || trimmed == "- [ ] " || trimmed == "- [x] " || trimmed == "- [X] "
                || trimmed == "- [ ]" || trimmed == "- [x]" || trimmed == "- [X]"
                || trimmed == "* [ ] " || trimmed == "* [x] " || trimmed == "* [X] "
                || trimmed == "* [ ]" || trimmed == "* [x]" || trimmed == "* [X]"
            {
                while self.cur > line_start {
                    self.cur -= 1;
                    self.buf.remove(self.cur);
                }
                self.selection = None;
                return;
            }

            // Normal single-character backspace
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

    /// Handles Enter key in insert mode:
    /// - If there is an active selection, replaces it with a simple newline without auto-indent.
    /// - Preserves exact leading whitespace (spaces and tabs) of current line.
    /// - Auto-continues numbered lists (`1. Hello` -> `2. `) and bullet lists (`- Hello`, `* Hello`, `+ Hello`).
    /// - If current line is only a marker with no content (`1. `, `- `), clears the marker and inserts a plain newline.
    /// - Composes indentation with lists (`  - Hello` -> `  - `, `  1. Hello` -> `  2. `).
    /// - Whole Enter action is recorded as a single atomic undo step.
    pub fn handle_enter(&mut self) {
        if self.has_selection() {
            self.insert('\n');
            return;
        }

        self.save_undo_snapshot();

        let mut line_start = self.cur;
        while line_start > 0 && self.buf[line_start - 1] != '\n' {
            line_start -= 1;
        }
        let mut line_end = self.cur;
        while line_end < self.buf.len() && self.buf[line_end] != '\n' {
            line_end += 1;
        }

        let line_str: String = self.buf[line_start..line_end].iter().collect();

        // 1. Extract leading whitespace (spaces and tabs)
        let indent_len = line_str.chars().take_while(|&c| c == ' ' || c == '\t').count();
        let indent = &line_str[..indent_len];
        let rest = &line_str[indent_len..];

        // If line contains only whitespace, clear it on Enter and insert plain newline at col 0
        if line_str.trim().is_empty() && !line_str.is_empty() {
            self.buf.drain(line_start..line_end);
            self.buf.insert(line_start, '\n');
            self.cur = line_start + 1;
            self.selection = None;
            return;
        }

        // 2. Check if current line is only a list marker with no content
        let is_empty_numbered = {
            let digits_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
            if digits_len > 0 && digits_len < rest.len() {
                let after_digits = &rest[digits_len..];
                if after_digits.starts_with('.') {
                    let after_dot = &after_digits[1..];
                    after_dot.trim().is_empty()
                } else {
                    false
                }
            } else {
                false
            }
        };

        let is_task = rest.starts_with("- [ ] ") || rest.starts_with("- [x] ") || rest.starts_with("- [X] ")
            || rest.starts_with("* [ ] ") || rest.starts_with("* [x] ") || rest.starts_with("* [X] ");
        let is_empty_task = is_task && rest[6..].trim().is_empty();

        let is_empty_bullet = {
            if rest.starts_with('-') || rest.starts_with('*') || rest.starts_with('+') {
                rest[1..].trim().is_empty()
            } else {
                false
            }
        };

        let quote_prefix = parse_quote_prefix(rest);
        let is_empty_quote = if let Some(qp) = quote_prefix {
            rest[qp.len()..].trim().is_empty()
        } else {
            false
        };

        let rest_trim = rest.trim();
        let is_table_row = rest_trim.contains('|')
            && (rest_trim.starts_with('|') || rest_trim.ends_with('|') || rest_trim.split('|').filter(|p| !p.trim().is_empty()).count() >= 2);
        let is_table_sep = is_table_row
            && rest_trim.contains('-')
            && rest_trim.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ');

        let is_empty_table_row = is_table_row && !is_table_sep && {
            let parts: Vec<&str> = rest_trim.trim_matches('|').split('|').collect();
            !parts.is_empty() && parts.iter().all(|p| p.trim().is_empty())
        };

        if is_empty_numbered || is_empty_task || (!is_task && is_empty_bullet) || is_empty_quote || is_empty_table_row {
            self.buf.drain(line_start..line_end);
            self.buf.insert(line_start, '\n');
            self.cur = line_start + 1;
            self.selection = None;
            return;
        }

        // 3. Check for list / quote / table continuation
        let (next_prefix, cursor_offset) = if is_task {
            (format!("\n{}- [ ] ", indent), None)
        } else if let Some(qp) = quote_prefix {
            if qp.ends_with(' ') {
                (format!("\n{}{}", indent, qp), None)
            } else {
                (format!("\n{}{} ", indent, qp), None)
            }
        } else if let Some(num) = parse_numbered_list(rest) {
            (format!("\n{}{}. ", indent, num.saturating_add(1)), None)
        } else if let Some(bullet) = parse_bullet_list(rest) {
            (format!("\n{}{} ", indent, bullet), None)
        } else if is_table_row {
            let cols: Vec<&str> = rest_trim.trim_matches('|').split('|').collect();
            let col_count = cols.len().max(1);

            let has_sep_above = {
                let mut prev_line_start = line_start;
                let mut found_sep = false;
                while prev_line_start > 0 {
                    let mut p_start = prev_line_start - 1;
                    while p_start > 0 && self.buf[p_start - 1] != '\n' {
                        p_start -= 1;
                    }
                    let p_str: String = self.buf[p_start..prev_line_start - 1].iter().collect();
                    let pt = p_str.trim();
                    if pt.contains('|') {
                        if pt.contains('-') && pt.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ') {
                            found_sep = true;
                            break;
                        }
                    } else {
                        break;
                    }
                    prev_line_start = p_start;
                }
                found_sep
            };

            let next_line_is_sep = {
                if line_end < self.buf.len() {
                    let mut next_end = line_end + 1;
                    while next_end < self.buf.len() && self.buf[next_end] != '\n' {
                        next_end += 1;
                    }
                    let next_str: String = self.buf[line_end + 1..next_end].iter().collect();
                    let nt = next_str.trim();
                    nt.starts_with('|') && nt.contains('-') && nt.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
                } else {
                    false
                }
            };

            if !is_table_sep && !has_sep_above && !next_line_is_sep {
                // If creating a table header without a separator, auto-generate separator and first row
                let mut s = format!("\n{}|", indent);
                for _ in 0..col_count {
                    s.push_str(" --- |");
                }
                s.push_str(&format!("\n{}|", indent));
                for _ in 0..col_count {
                    s.push_str("  |");
                }
                let first_cell_offset = 1 + indent.len() + 6 * col_count + 1 + 1 + indent.len() + 2;
                (s, Some(first_cell_offset))
            } else {
                let mut s = format!("\n{}|", indent);
                for _ in 0..col_count {
                    s.push_str("  |");
                }
                let first_cell_offset = 1 + indent.len() + 2;
                (s, Some(first_cell_offset))
            }
        } else {
            // Preserve exact leading whitespace
            (format!("\n{}", indent), None)
        };

        for (idx, ch) in next_prefix.chars().enumerate() {
            self.buf.insert(self.cur + idx, ch);
        }
        if let Some(offset) = cursor_offset {
            self.cur += offset.min(next_prefix.chars().count());
        } else {
            self.cur += next_prefix.chars().count();
        }
        self.selection = None;
    }
}

fn parse_quote_prefix(rest: &str) -> Option<&str> {
    if !rest.starts_with('>') {
        return None;
    }
    let mut end = 0;
    let bytes = rest.as_bytes();
    while end < bytes.len() && (bytes[end] == b'>' || bytes[end] == b' ') {
        end += 1;
    }
    if end > 0 {
        Some(&rest[..end])
    } else {
        None
    }
}

fn parse_numbered_list(rest: &str) -> Option<u64> {
    let digits_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits_len == 0 || digits_len >= rest.len() {
        return None;
    }
    let after_digits = &rest[digits_len..];
    if !after_digits.starts_with('.') {
        return None;
    }
    let after_dot = &after_digits[1..];
    if !after_dot.starts_with(' ') && !after_dot.starts_with('\t') {
        return None;
    }
    let content = after_dot.trim_start_matches(|c| c == ' ' || c == '\t');
    if content.is_empty() {
        return None;
    }
    rest[..digits_len].parse::<u64>().ok()
}

fn parse_bullet_list(rest: &str) -> Option<char> {
    let mut chars = rest.chars();
    let bullet = chars.next()?;
    if bullet != '-' && bullet != '*' && bullet != '+' {
        return None;
    }
    let after_bullet = &rest[bullet.len_utf8()..];
    if !after_bullet.starts_with(' ') && !after_bullet.starts_with('\t') {
        return None;
    }
    let content = after_bullet.trim_start_matches(|c| c == ' ' || c == '\t');
    if content.is_empty() {
        return None;
    }
    Some(bullet)
}
