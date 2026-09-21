//! Main text editor view: header, text canvas, smooth scrolling, and caret rendering.

use crate::caret::Caret;
use crate::editor::{Editor, VisualLine};
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

/// Renders the clean editor header with active title and top-right window drag gripper.
pub fn render_editor_header(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    content_left_margin: f32,
    content_right_margin: f32,
    active_title: &str,
    theme: &Theme,
) {
    let header_y = bounds.min.y + 14.0;
    let display_title = if active_title.len() > 40 {
        format!("📄 {}...", &active_title[..40])
    } else {
        format!("📄 {}", active_title)
    };

    painter.text(
        pos2(bounds.min.x + content_left_margin, header_y),
        Align2::LEFT_TOP,
        display_title,
        FontId::monospace(13.5),
        theme.highlight,
    );

    // Sleek Window Gripper on the top right side for dragging borderless window
    let gripper_rect = Rect::from_min_size(
        pos2(bounds.max.x - 38.0, bounds.min.y + 11.0),
        vec2(24.0, 20.0),
    );
    let is_hovered = ui.rect_contains_pointer(gripper_rect);
    if is_hovered && ui.input(|i| i.pointer.primary_down()) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    let grip_color = if is_hovered {
        theme.accent
    } else {
        Color32::from_gray(80)
    };

    // 6-dot matrix gripper (2 cols x 3 rows)
    for col in 0..2 {
        for row in 0..3 {
            let cx = gripper_rect.min.x + 6.0 + col as f32 * 8.0;
            let cy = gripper_rect.min.y + 4.0 + row as f32 * 6.0;
            painter.circle_filled(pos2(cx, cy), 1.5, grip_color);
        }
    }

    // Subtle divider line under header, symmetrically aligned with content margins
    painter.line_segment(
        [
            pos2(bounds.min.x + content_left_margin, bounds.min.y + 38.0),
            pos2(bounds.max.x - content_right_margin, bounds.min.y + 38.0),
        ],
        Stroke::new(1.0, Color32::from_gray(24)),
    );
}

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
) {
    let font = FontId::monospace(font_size);
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

    let ed_origin = editor_rect.min - vec2(0.0, *scroll_y);

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
                    let highlight_rect = Rect::from_min_size(pos2(ed_origin.x, line_y), vec2(sel_w, lh));
                    editor_painter.rect_filled(highlight_rect, 2.0, sel_color);
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
                    let highlight_rect = Rect::from_min_size(pos2(sel_x, line_y), vec2(sel_w, lh));
                    editor_painter.rect_filled(highlight_rect, 2.0, sel_color);
                }
            }
        }

        let line_text: String = ed.buf[line.char_start..line.char_end].iter().collect();
        editor_painter.text(
            pos2(ed_origin.x, line_y),
            Align2::LEFT_TOP,
            line_text,
            font.clone(),
            theme.text,
        );
    }

    // Caret placement and animation
    let caret_x = ed_origin.x + col as f32 * cw;
    let target = pos2(caret_x, ed_origin.y + row as f32 * lh);
    caret.update(dt, target, typed, now, cw, lh);
    caret.paint(&editor_painter, cw, lh, now, theme.accent);

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
