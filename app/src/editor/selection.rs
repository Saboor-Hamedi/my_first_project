use super::Editor;

impl Editor {
    pub fn selected_range(&self) -> Option<(usize, usize)> {
        if let Some(anchor) = self.selection {
            if self.selection_inclusive {
                let min = anchor.min(self.cur);
                let max = anchor.max(self.cur);
                let end = (max + 1).min(self.buf.len());
                return Some((min, end));
            } else if anchor != self.cur {
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
        self.selection_inclusive = false;
    }

    pub fn select_all(&mut self) {
        if !self.buf.is_empty() {
            self.selection = Some(0);
            self.cur = self.buf.len();
            self.selection_inclusive = false;
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
            self.selection_inclusive = false;
            true
        } else {
            false
        }
    }
}
