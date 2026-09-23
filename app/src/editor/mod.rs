mod editing;
mod lines;
mod movement;
mod selection;
mod undo;
mod visual;

#[cfg(test)]
mod tests;

pub use crate::types::{EditorSnapshot, VisualLine};
pub use visual::caret_cell;

#[derive(Default, Clone)]
pub struct Editor {
    pub buf: Vec<char>,
    pub cur: usize,
    pub selection: Option<usize>,
    pub selection_inclusive: bool,
    pub undo_stack: Vec<EditorSnapshot>,
    pub redo_stack: Vec<EditorSnapshot>,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        if !self.buf.is_empty() {
            self.save_undo_snapshot();
        }
        self.buf.clear();
        self.cur = 0;
        self.selection = None;
        self.selection_inclusive = false;
    }

    pub fn set_text(&mut self, s: &str) {
        self.buf = s.chars().collect();
        self.cur = 0;
        self.selection = None;
        self.selection_inclusive = false;
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
