use eframe::egui::text::CCursor;
use eframe::egui::{pos2, vec2, Galley, Pos2, Rect};
use std::sync::Arc;

/// Classification of a markdown line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineLineKind {
    /// Normal paragraph text
    Normal,
    /// Heading with level 1..=4
    Heading(u8),
    /// Blockquote with text
    Quote,
    /// Interactive task item (- [ ] or - [x])
    TaskItem {
        checked: bool,
        /// Buffer character offset of the check byte (' ' or 'x')
        check_char_idx: usize,
    },
    /// Bullet list item (- or * or +)
    BulletItem,
    /// Numbered list item (1., 2., etc.)
    NumberedItem(String),
    /// Horizontal divider (---, ***)
    Rule,
    /// Code fence start/end (```lang)
    CodeFence(String),
    /// Line inside a code block
    CodeLine,
    /// Markdown table row (| col1 | col2 |)
    TableRow {
        is_header: bool,
        is_separator: bool,
    },
}

/// A formatted visual line in the inline editor.
pub struct InlineLine {
    /// Starting character index in `Editor::buf`
    pub char_start: usize,
    /// Ending character index in `Editor::buf` (excluding newline)
    pub char_end: usize,
    /// Cumulative Y offset from top of document (in points)
    pub y_offset: f32,
    /// Line height (in points)
    pub height: f32,
    /// Base font size used for this line
    #[allow(dead_code)]
    pub base_font_size: f32,
    /// Type of line
    pub kind: InlineLineKind,
    /// Laid-out text galley from egui
    pub galley: Arc<Galley>,
    /// Maps each displayed character index in `galley` back to its index in `Editor::buf`
    pub char_map: Vec<usize>,
    /// Bounding box for interactive checkbox widget (if this is a TaskItem)
    pub checkbox_rect: Option<Rect>,
}

/// Complete laid-out document containing all visual lines and total height.
pub struct InlineEditorLayout {
    pub lines: Vec<InlineLine>,
    pub total_height: f32,
}

impl InlineEditorLayout {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            total_height: 0.0,
        }
    }

    /// Finds the visual line index containing the given Y offset.
    pub fn line_at_y(&self, y: f32) -> usize {
        if self.lines.is_empty() {
            return 0;
        }
        if y <= 0.0 {
            return 0;
        }

        let mut low = 0;
        let mut high = self.lines.len() - 1;

        while low <= high {
            let mid = (low + high) / 2;
            let line = &self.lines[mid];

            if y < line.y_offset {
                if mid == 0 {
                    return 0;
                }
                high = mid - 1;
            } else if y >= line.y_offset + line.height {
                low = mid + 1;
            } else {
                return mid;
            }
        }

        low.min(self.lines.len().saturating_sub(1))
    }

    /// Finds the visual line index containing the given buffer character index.
    pub fn line_for_char(&self, char_idx: usize) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        for (i, line) in self.lines.iter().enumerate() {
            if char_idx >= line.char_start && char_idx <= line.char_end {
                return i;
            }
        }

        self.lines.len().saturating_sub(1)
    }

    /// Returns the exact `(Pos2, line_height)` for a given buffer character index.
    pub fn pos_for_char(&self, char_idx: usize, ed_origin: Pos2) -> (Pos2, f32) {
        if self.lines.is_empty() {
            return (ed_origin, 20.0);
        }

        let line_idx = self.line_for_char(char_idx);
        let line = &self.lines[line_idx];
        let line_y = ed_origin.y + line.y_offset;

        // Find closest galley character in char_map
        let mut best_galley_idx = 0;
        let mut min_diff = usize::MAX;

        for (g_idx, &b_idx) in line.char_map.iter().enumerate() {
            let diff = b_idx.abs_diff(char_idx);
            if diff < min_diff {
                min_diff = diff;
                best_galley_idx = g_idx;
                if diff == 0 {
                    break;
                }
            }
        }

        let cursor = line.galley.from_ccursor(CCursor::new(best_galley_idx));
        let cursor_rect = line.galley.pos_from_cursor(&cursor);
        let x = ed_origin.x + cursor_rect.min.x;
        let y = line_y + cursor_rect.min.y;

        let caret_h = cursor_rect.height().max(16.0);
        (pos2(x, y), caret_h)
    }

    /// Maps a mouse position back to a character index in `Editor::buf`.
    pub fn char_at_pos(&self, mouse_pos: Pos2, ed_origin: Pos2) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        let rel_y = mouse_pos.y - ed_origin.y;
        let line_idx = self.line_at_y(rel_y);
        let line = &self.lines[line_idx];
        let line_y = ed_origin.y + line.y_offset;
        let rel_x = (mouse_pos.x - ed_origin.x).max(0.0);
        let rel_row_y = (mouse_pos.y - line_y).clamp(0.0, line.height.max(1.0));
        let ccursor = line.galley.cursor_from_pos(vec2(rel_x, rel_row_y));
        let g_idx = ccursor.ccursor.index;

        if let Some(&buf_idx) = line.char_map.get(g_idx) {
            buf_idx.clamp(line.char_start, line.char_end)
        } else if let Some(&last_idx) = line.char_map.last() {
            last_idx.clamp(line.char_start, line.char_end)
        } else {
            line.char_start
        }
    }
}
