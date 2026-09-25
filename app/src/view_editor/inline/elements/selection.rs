//! Pixel-perfect, continuous multi-line selection rendering (The Lumina Standard).
//!
//! Provides:
//! 1. Seamless vertical fusion: consecutive visual rows meet with zero horizontal gaps or seams.
//! 2. Text-boundary precision: highlights stop strictly at the actual text content of each row,
//!    with zero bleed into whitespace or margins.
//! 3. Single-mesh rasterization: renders all row quads in a unified GPU `Mesh` to prevent
//!    anti-aliasing seams and double-alpha blending artifacts.
//! 4. Cursor integration: guarantees the selection highlight extends completely under the cursor
//!    character on both visual forward and backward motions.

use super::super::types::InlineEditorLayout;
use eframe::egui::text::CCursor;
use eframe::egui::{pos2, Color32, Mesh, Painter, Pos2, Rect, Shape};

/// Internal representation of a single visual row's selection metrics.
#[derive(Clone, Copy, Debug)]
struct RowHighlight {
    line_idx: usize,
    x_min: f32,
    x_max: f32,
    raw_top: f32,
    raw_bottom: f32,
    line_top: f32,
    line_bottom: f32,
}

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

    let mut raw_highlights: Vec<RowHighlight> = Vec::new();

    for (line_idx, line) in layout.lines.iter().enumerate() {
        // Skip lines completely outside selection (half-open range [sel_start, sel_end))
        if sel_start > line.char_end || sel_end <= line.char_start {
            continue;
        }

        let line_top = (ed_origin.y + line.y_offset).round();
        let line_bottom = (line_top + line.height).round();
        let galley_y_pad = ((line.height - line.galley.size().y) * 0.5).round().max(0.0);
        let galley_top = line_top + galley_y_pad;
        let char_w = (line.base_font_size * 0.6).round().max(8.0);

        let is_first_line = sel_start >= line.char_start && sel_start <= line.char_end;
        let is_last_line = sel_end >= line.char_start && sel_end <= line.char_end + 1;

        if line.char_map.is_empty() {
            // Empty line selected (represents newline character)
            raw_highlights.push(RowHighlight {
                line_idx,
                x_min: text_left,
                x_max: text_left + char_w,
                raw_top: line_top,
                raw_bottom: line_bottom,
                line_top,
                line_bottom,
            });
            continue;
        }

        // Map buffer indices to galley character indices
        let start_b = sel_start.max(line.char_start);
        let end_b = sel_end.min(line.char_end);

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

        let num_rows = line.galley.rows.len();
        let row_start = if is_first_line {
            c1.rcursor.row.min(num_rows.saturating_sub(1))
        } else {
            0
        };
        let row_end = if is_last_line {
            c2.rcursor.row.min(num_rows.saturating_sub(1))
        } else {
            num_rows.saturating_sub(1)
        };

        for r_idx in row_start..=row_end {
            let Some(row) = line.galley.rows.get(r_idx) else { continue };

            let is_start_row = is_first_line && r_idx == row_start;
            let is_end_row = is_last_line && r_idx == row_end;

            let (mut x_min, mut x_max) = if is_start_row && is_end_row {
                // Single row selection
                let min_x = (text_left + r1.min.x.min(r2.min.x)).max(text_left);
                let max_x = text_left + r1.max.x.max(r2.max.x);
                (min_x, max_x)
            } else if is_start_row {
                // First row of a multi-row selection: starts at r1, extends to exact row text edge
                let min_x = (text_left + r1.min.x).max(text_left);
                let max_x = text_left + row.rect.max.x;
                (min_x, max_x)
            } else if is_end_row {
                // Last row of a multi-row selection: starts at beginning of row, ends at r2
                let min_x = text_left + row.rect.min.x.max(0.0);
                let max_x = text_left + r2.max.x;
                (min_x, max_x)
            } else {
                // Intermediate fully-selected row: strictly matches actual text bounds
                let min_x = text_left + row.rect.min.x.max(0.0);
                let max_x = text_left + row.rect.max.x;
                (min_x, max_x)
            };

            if x_max <= x_min + 1.0 {
                x_max = x_min + char_w;
            }

            // Cursor Integration: ensure character under cursor at both selection ends is fully covered
            let row_top = galley_top + row.rect.min.y;
            let row_bottom = galley_top + row.rect.max.y;

            if is_start_row {
                let (cur_pos, _) = layout.pos_for_char(sel_start, ed_origin);
                if cur_pos.y >= row_top - 2.0 && cur_pos.y <= row_bottom + 2.0 {
                    x_min = x_min.min(cur_pos.x);
                    if is_end_row {
                        x_max = x_max.max(cur_pos.x + char_w);
                    }
                }
            }
            if is_end_row && sel_end > line.char_start {
                let (cur_pos, _) = layout.pos_for_char(sel_end.saturating_sub(1), ed_origin);
                if cur_pos.y >= row_top - 2.0 && cur_pos.y <= row_bottom + 2.0 {
                    x_max = x_max.max(cur_pos.x + char_w);
                }
            }

            let raw_top = (galley_top + row.rect.min.y).round();
            let raw_bottom = (galley_top + row.rect.max.y).round();

            raw_highlights.push(RowHighlight {
                line_idx,
                x_min,
                x_max,
                raw_top,
                raw_bottom,
                line_top,
                line_bottom,
            });
        }
    }

    let n = raw_highlights.len();
    if n == 0 {
        return;
    }

    let mut mesh = Mesh::default();

    if n == 1 {
        // Single visual row selection: match row/caret height precisely
        let h = &raw_highlights[0];
        let rect = Rect::from_min_max(pos2(h.x_min, h.raw_top), pos2(h.x_max, h.raw_bottom));
        mesh.add_colored_rect(rect, sel_color);
        painter.add(Shape::mesh(mesh));
        return;
    }

    // Multi-row selection: build seamlessly fused geometric shape with zero horizontal gaps
    for i in 0..n {
        let h = &raw_highlights[i];

        // Top boundary: first row starts at text top; subsequent rows seamlessly connect to preceding row
        let top = if i == 0 {
            h.raw_top
        } else {
            let prev = &raw_highlights[i - 1];
            if prev.line_idx == h.line_idx {
                prev.raw_bottom
            } else {
                h.line_top
            }
        };

        // Bottom boundary: last row ends at text bottom; preceding rows seamlessly connect to following row
        let bottom = if i + 1 == n {
            h.raw_bottom
        } else {
            let next = &raw_highlights[i + 1];
            if next.line_idx == h.line_idx {
                h.raw_bottom
            } else {
                h.line_bottom
            }
        };

        let rect = Rect::from_min_max(pos2(h.x_min, top), pos2(h.x_max, bottom));
        mesh.add_colored_rect(rect, sel_color);
    }

    painter.add(Shape::mesh(mesh));
}
