//! Editor text body layout, scrolling, selection, search highlights, and rendering.

use super::ligatures::render_line_with_ligatures;
use crate::caret::Caret;
use crate::editor::{Editor, VisualLine};
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Color32, FontId, Rect};

/// Renders the editor body with soft-wrapped visual lines, smooth scrolling, caret animation, and interactive scrollbar.
pub fn render_editor_body(
    ui: &egui::Ui,
    painter: &egui::Painter,
    window_bounds: Rect,
    editor_rect: Rect,
    ed: &mut Editor,
    visual_lines: &[VisualLine],
    caret: &mut Caret,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
    cw: f32,
    lh: f32,
    dt: f32,
    now: f64,
    mut typed: bool,
    block_scroll: bool,
    search_matches: Option<(&[usize], usize)>,
) {
    let font = FontId::monospace(font_size);
    let font_h = painter.layout_no_wrap("M".to_owned(), font.clone(), Color32::WHITE).size().y;
    let y_pad = ((lh - font_h) * 0.5).round().max(0.0);
    let stroke_w = (font_size * 0.088).clamp(1.2, 1.8);
    let visible_h = editor_rect.height();

    let total_content_h = visual_lines.len() as f32 * lh;
    let max_scroll = (total_content_h - visible_h + lh * 4.0).max(0.0);

    // Mouse wheel scrolling: prioritize smooth scroll delta, fallback to scaled raw delta
    if !block_scroll {
        let scroll_delta = ui.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.001 {
                i.smooth_scroll_delta.y
            } else {
                i.raw_scroll_delta.y * 0.5
            }
        });
        if scroll_delta != 0.0 && ui.rect_contains_pointer(editor_rect) {
            *scroll_y = (*scroll_y - scroll_delta).clamp(0.0, max_scroll);
        }
    }

    let pad_x = 4.0;
    let pad_y = 2.0;
    let ed_origin = editor_rect.min - vec2(0.0, *scroll_y) + vec2(pad_x, pad_y);

    // Direct mouse click in editor moves cursor to clicked visual row and col
    if !block_scroll && ui.rect_contains_pointer(editor_rect) && ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let clicked_row = ((pos.y - ed_origin.y) / lh).floor() as isize;
            if clicked_row >= 0 && (clicked_row as usize) < visual_lines.len() {
                let r = clicked_row as usize;
                let line = &visual_lines[r];
                let clicked_col = (((pos.x - ed_origin.x).max(0.0)) / cw).round() as usize;
                let line_len = line.char_end.saturating_sub(line.char_start);
                ed.cur = line.char_start + clicked_col.min(line_len);
                ed.selection = None;
                typed = true;
            }
        }
    }

    // Caret position in visual lines
    let (row, col) = ed.visual_row_col(visual_lines);
    let caret_y_in_content = row as f32 * lh;

    // Auto-scroll / caret follow: ONLY when user is typing or navigating by keyboard!
    if typed {
        // Pin smoothly at bottom edge when moving downwards
        if caret_y_in_content + lh > *scroll_y + visible_h {
            *scroll_y = (caret_y_in_content + lh - visible_h).clamp(0.0, max_scroll);
        }
        // Pin smoothly at top edge when moving upwards
        if caret_y_in_content < *scroll_y {
            *scroll_y = caret_y_in_content.clamp(0.0, max_scroll);
        }
    }

    // Clip drawing strictly to editor bounds
    let editor_painter = painter.with_clip_rect(editor_rect);

    let sel_range = ed.selected_range();
    let sel_color = Color32::from_rgba_unmultiplied(
        theme.accent.r(),
        theme.accent.g(),
        theme.accent.b(),
        65,
    );

    // Caret placement and animation
    let caret_x = ed_origin.x + col as f32 * cw;
    let target = pos2(caret_x, ed_origin.y + row as f32 * lh);
    caret.update(dt, target, typed, now, cw, lh);
    caret.paint(&editor_painter, cw, lh, now, theme.accent);

    let is_block = caret.kind == crate::caret::CaretKind::Block;

    // Frustum culling: render only lines intersecting the visible viewport
    for (r, line) in visual_lines.iter().enumerate() {
        let line_y = ed_origin.y + r as f32 * lh;
        if line_y + lh < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }

        // Render selection highlight background behind text on this line
        if let Some((sel_start, sel_end)) = sel_range {
            if line.char_start == line.char_end {
                // Empty line gap between paragraphs (\n\n):
                // Highlight a visible block only when the active selection spans across this empty line.
                if sel_start <= line.char_start && sel_end > line.char_end {
                    let sel_w = cw.max(12.0);
                    let highlight_rect = Rect::from_min_size(pos2(ed_origin.x, line_y), vec2(sel_w, lh + 0.5));
                    editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
                }
            } else {
                let intersect_start = sel_start.max(line.char_start);
                let intersect_end = sel_end.min(line.char_end);
                if intersect_start < intersect_end {
                    let start_col = intersect_start - line.char_start;
                    let end_col = intersect_end - line.char_start;
                    let sel_x = ed_origin.x + start_col as f32 * cw;
                    let mut sel_w = (end_col - start_col) as f32 * cw;
                    // If selection extends past this line's content (e.g. across newline to subsequent lines),
                    // extend the highlight to visually show that the newline / trailing whitespace is selected.
                    if sel_end > line.char_end {
                        sel_w += cw.max(10.0);
                    }
                    let highlight_rect = Rect::from_min_size(pos2(sel_x, line_y), vec2(sel_w, lh + 0.5));
                    editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
                }
            }
        }

        // Render in-buffer search match highlights behind text on this line
        if let Some((matches, q_len)) = search_matches {
            if q_len > 0 {
                for &m_idx in matches {
                    if m_idx + q_len <= line.char_start || m_idx >= line.char_end {
                        continue;
                    }
                    let match_start = m_idx.max(line.char_start);
                    let match_end = (m_idx + q_len).min(line.char_end);
                    if match_start < match_end {
                        let start_col = match_start - line.char_start;
                        let end_col = match_end - line.char_start;
                        let match_x = ed_origin.x + start_col as f32 * cw;
                        let match_w = (end_col - start_col) as f32 * cw;
                        let is_current = m_idx == ed.cur;
                        let match_color = if is_current {
                            Color32::from_rgba_unmultiplied(255, 195, 45, 120) // brighter amber for active match
                        } else {
                            Color32::from_rgba_unmultiplied(255, 215, 60, 50)  // soft subtle amber
                        };
                        editor_painter.rect_filled(
                            Rect::from_min_size(pos2(match_x, line_y), vec2(match_w, lh)),
                            2.0,
                            match_color,
                        );
                    }
                }
            }
        }

        let text_y = line_y + y_pad;
        let y_mid = line_y + (lh * 0.5).round();

        let line_len = line.char_end.saturating_sub(line.char_start);
        let block_col = if r == row && is_block && col < line_len {
            Some(col)
        } else {
            None
        };
        let line_chars = &ed.buf[line.char_start..line.char_end];
        render_line_with_ligatures(
            &editor_painter,
            ed_origin.x,
            text_y,
            line_y,
            y_mid,
            line_chars,
            &font,
            theme.text,
            cw,
            lh,
            stroke_w,
            block_col,
        );
    }

    // Interactive scrollbar indicator in the right margin gutter
    if total_content_h > visible_h && max_scroll > 0.0 {
        let thumb_h = ((visible_h / total_content_h) * visible_h).clamp(28.0, visible_h);
        let scroll_ratio = (*scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = editor_rect.min.y + scroll_ratio * (visible_h - thumb_h);
        let track_x = window_bounds.max.x - 8.0;
        let thumb_rect = Rect::from_min_size(pos2(track_x - 1.5, thumb_y), vec2(3.0, thumb_h));
        let track_rect = Rect::from_min_max(
            pos2(track_x - 8.0, editor_rect.min.y),
            pos2(window_bounds.max.x, editor_rect.max.y),
        );

        let is_track_hovered = ui.rect_contains_pointer(track_rect);
        if is_track_hovered && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let ratio = ((pos.y - editor_rect.min.y - thumb_h * 0.5) / (visible_h - thumb_h)).clamp(0.0, 1.0);
                *scroll_y = ratio * max_scroll;
            }
        }

        let thumb_color = if is_track_hovered {
            Color32::from_rgba_unmultiplied(180, 185, 200, 160)
        } else {
            Color32::from_rgba_unmultiplied(120, 125, 140, 70)
        };
        painter.rect_filled(thumb_rect, 1.5, thumb_color);
    }
}
