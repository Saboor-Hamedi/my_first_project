//! Pixel-perfect, continuous multi-line selection rendering (The Lumina Standard).
//!
//! Provides:
//! 1. Unified continuous geometry across line breaks without gaps or stacked disjoint blocks.
//! 2. Glyph-boundary precision with zero bleed into gutter or margins.
//! 3. Sub-pixel overlap (0.5px–1.0px) between connected rows to eliminate anti-aliasing seams.
//! 4. Seamless multi-line flow when navigating with Vim motions (v, j, k).

use super::super::types::InlineEditorLayout;
use eframe::egui::text::CCursor;
use eframe::egui::{pos2, Color32, Painter, Pos2, Rect};

/// Renders continuous, unified selection highlights across all lines in the document.
pub fn render_document_selection(
    painter: &Painter,
    layout: &InlineEditorLayout,
    ed_origin: Pos2,
    text_left: f32,
    sel_start: usize,
    sel_end: usize,
    sel_color: Color32,
) {
    if sel_start >= sel_end || layout.lines.is_empty() {
        return;
    }

    for line in &layout.lines {
        // Skip lines completely outside selection
        if sel_start > line.char_end || sel_end < line.char_start {
            continue;
        }

        let line_y = ed_origin.y + line.y_offset;
        let is_sel_first_line = sel_start >= line.char_start && sel_start <= line.char_end;
        let is_sel_last_line = sel_end >= line.char_start && sel_end <= line.char_end + 1;

        let start_b = sel_start.max(line.char_start);
        let end_b = sel_end.min(line.char_end);

        if line.char_map.is_empty() {
            // Empty line selected
            let top = if is_sel_first_line { line_y } else { line_y - 0.5 };
            let bottom = if is_sel_last_line { line_y + line.height } else { line_y + line.height + 0.5 };
            let empty_rect = Rect::from_min_max(pos2(text_left, top), pos2(text_left + 12.0, bottom));
            painter.rect_filled(empty_rect, 0.0, sel_color);
            continue;
        }

        // Map buffer indices to galley character indices
        let mut start_g = 0;
        let mut end_g = line.char_map.len().saturating_sub(1);

        for (g, &b) in line.char_map.iter().enumerate() {
            if b <= start_b {
                start_g = g;
            }
            if b >= end_b {
                end_g = g;
                break;
            }
        }

        let c1 = line.galley.from_ccursor(CCursor::new(start_g));
        let c2 = line.galley.from_ccursor(CCursor::new(end_g));
        let r1 = line.galley.pos_from_cursor(&c1);
        let r2 = line.galley.pos_from_cursor(&c2);

        let row1 = c1.rcursor.row.min(line.galley.rows.len().saturating_sub(1));
        let row2 = c2.rcursor.row.min(line.galley.rows.len().saturating_sub(1));

        // SCENARIO 1: Selection is entirely within this single line
        if is_sel_first_line && is_sel_last_line {
            if row1 == row2 {
                // Single row
                let row_h = r1.height().max(16.0);
                let top = line_y + r1.min.y;
                let bottom = top + row_h;
                let x_min = (text_left + r1.min.x.min(r2.min.x)).max(text_left);
                let x_max = (text_left + r1.max.x.max(r2.max.x)).max(x_min + 4.0);

                let sel_rect = Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom));
                painter.rect_filled(sel_rect, 0.0, sel_color);
            } else {
                // Multi-row within the same paragraph/line
                if let Some(first_row) = line.galley.rows.get(row1) {
                    let top = line_y + r1.min.y;
                    let bottom = line_y + first_row.rect.max.y + 0.5;
                    let x_min = (text_left + r1.min.x).max(text_left);
                    let x_max = text_left + first_row.rect.max.x + 4.0;
                    painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                }
                for r_idx in (row1 + 1)..row2 {
                    if let Some(mid_row) = line.galley.rows.get(r_idx) {
                        let top = line_y + mid_row.rect.min.y - 0.5;
                        let bottom = line_y + mid_row.rect.max.y + 0.5;
                        let x_min = text_left + mid_row.rect.min.x.max(0.0);
                        let x_max = text_left + mid_row.rect.max.x + 4.0;
                        painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                    }
                }
                if let Some(last_row) = line.galley.rows.get(row2) {
                    let top = line_y + last_row.rect.min.y - 0.5;
                    let bottom = top + r2.height().max(16.0);
                    let x_min = text_left + last_row.rect.min.x.max(0.0);
                    let x_max = (text_left + r2.max.x).max(x_min + 4.0);
                    painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                }
            }
            continue;
        }

        // SCENARIO 2: First line of a multi-line selection (selection continues downward)
        if is_sel_first_line {
            let row_count = line.galley.rows.len();
            if row1 >= row_count.saturating_sub(1) {
                // Starts on the last (or only) row of this line
                let top = line_y + r1.min.y;
                // Extend seamlessly to the next line with 0.5px sub-pixel overlap
                let bottom = line_y + line.height + 0.5;
                let x_min = (text_left + r1.min.x).max(text_left);
                let x_max = text_left + line.galley.size().x.max(r1.max.x + 8.0) + 6.0;
                painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
            } else {
                // Starts on an earlier row of this multi-row line
                if let Some(first_row) = line.galley.rows.get(row1) {
                    let top = line_y + r1.min.y;
                    let bottom = line_y + first_row.rect.max.y + 0.5;
                    let x_min = (text_left + r1.min.x).max(text_left);
                    let x_max = text_left + first_row.rect.max.x + 4.0;
                    painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                }
                for r_idx in (row1 + 1)..row_count.saturating_sub(1) {
                    if let Some(mid_row) = line.galley.rows.get(r_idx) {
                        let top = line_y + mid_row.rect.min.y - 0.5;
                        let bottom = line_y + mid_row.rect.max.y + 0.5;
                        let x_min = text_left + mid_row.rect.min.x.max(0.0);
                        let x_max = text_left + mid_row.rect.max.x + 4.0;
                        painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                    }
                }
                // Last row of this line connects downward to next line
                if let Some(last_row) = line.galley.rows.last() {
                    let top = line_y + last_row.rect.min.y - 0.5;
                    let bottom = line_y + line.height + 0.5;
                    let x_min = text_left + last_row.rect.min.x.max(0.0);
                    let x_max = text_left + last_row.rect.max.x + 8.0;
                    painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                }
            }
            continue;
        }

        // SCENARIO 3: Middle line of a multi-line selection (fully selected)
        if !is_sel_first_line && !is_sel_last_line {
            let top = line_y - 0.5;
            let bottom = line_y + line.height + 0.5;
            let x_min = text_left;
            let x_max = text_left + line.galley.size().x.max(24.0) + 8.0;
            painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
            continue;
        }

        // SCENARIO 4: Last line of a multi-line selection (selection ends here)
        if is_sel_last_line {
            if row2 == 0 {
                // Ends on the first row of this line
                let top = line_y - 0.5;
                let bottom = line_y + r2.min.y + r2.height().max(16.0);
                let x_min = text_left;
                let x_max = (text_left + r2.max.x).max(text_left + 4.0);
                painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
            } else {
                // Spans earlier rows of this line before ending on row2
                for r_idx in 0..row2 {
                    if let Some(prev_row) = line.galley.rows.get(r_idx) {
                        let top = if r_idx == 0 { line_y - 0.5 } else { line_y + prev_row.rect.min.y - 0.5 };
                        let bottom = line_y + prev_row.rect.max.y + 0.5;
                        let x_min = text_left;
                        let x_max = text_left + prev_row.rect.max.x + 4.0;
                        painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
                    }
                }
                // Ending row
                let top = line_y + line.galley.rows[row2].rect.min.y - 0.5;
                let bottom = line_y + r2.min.y + r2.height().max(16.0);
                let x_min = text_left;
                let x_max = (text_left + r2.max.x).max(text_left + 4.0);
                painter.rect_filled(Rect::from_min_max(pos2(x_min, top), pos2(x_max, bottom)), 0.0, sel_color);
            }
        }
    }
}
