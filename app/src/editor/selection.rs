use super::Editor;

impl Editor {
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
        self.selected_range()
            .map(|(s, e)| self.buf[s..e].iter().collect())
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
}
