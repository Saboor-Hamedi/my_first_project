//! Main text editor view: header, text canvas, smooth scrolling, and caret rendering.

use crate::caret::Caret;
use crate::editor::Editor;
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

/// Renders the clean editor header with active title and top-right window drag gripper.
pub fn render_editor_header(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    content_left_margin: f32,
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

    // Subtle divider line under header
    painter.line_segment(
        [
            pos2(bounds.min.x + content_left_margin, bounds.min.y + 38.0),
            pos2(bounds.max.x - 24.0, bounds.min.y + 38.0),
        ],
        Stroke::new(1.0, Color32::from_gray(24)),
    );
}

/// Renders the editor body with smooth pinned scrolling (no jumps on Enter!) and caret animation.
pub fn render_editor_body(
    ui: &egui::Ui,
    painter: &egui::Painter,
    editor_rect: Rect,
    ed: &Editor,
    caret: &mut Caret,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
    cw: f32,
    lh: f32,
    dt: f32,
    now: f64,
    typed: bool,
    block_scroll: bool,
) {
    let font = FontId::monospace(font_size);
    let (row, col) = ed.row_col();
    let caret_y_in_content = row as f32 * lh;
    let visible_h = editor_rect.height();

    let text = ed.text();
    let total_lines = text.split('\n').count();
    let total_content_h = total_lines as f32 * lh;
    let max_scroll = (total_content_h - visible_h + lh * 4.0).max(0.0);

    // Mouse wheel scrolling — smooth and completely decoupled from caret snapping!
    if !block_scroll {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 && ui.rect_contains_pointer(editor_rect) {
            *scroll_y = (*scroll_y - scroll_delta).clamp(0.0, max_scroll);
        }
    }

    // Caret follow / auto-scroll: ONLY when the user is actively typing or moving by keyboard!
    if typed {
        // Pin smoothly at bottom edge when moving downwards (zero jump overshoot)
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
    let ed_origin = editor_rect.min - vec2(0.0, *scroll_y);

    for (r, line) in text.split('\n').enumerate() {
        let line_y = ed_origin.y + r as f32 * lh;
        if line_y + lh < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }
        editor_painter.text(
            pos2(ed_origin.x, line_y),
            Align2::LEFT_TOP,
            line,
            font.clone(),
            theme.text,
        );
    }

    // Caret placement and animation
    let lines: Vec<&str> = text.split('\n').collect();
    let current_line = lines.get(row).copied().unwrap_or("");
    let current_line_prefix: String = current_line.chars().take(col).collect();

    let caret_x = if current_line_prefix.is_empty() {
        ed_origin.x
    } else {
        let galley = editor_painter.layout_no_wrap(current_line_prefix, font.clone(), Color32::WHITE);
        ed_origin.x + galley.size().x
    };
    let target = pos2(caret_x, ed_origin.y + row as f32 * lh);
    caret.update(dt, target, typed, now, cw, lh);
    caret.paint(&editor_painter, cw, lh, now, theme.accent);

    // Sleek scrollbar indicator when document exceeds viewport height
    if total_content_h > visible_h && max_scroll > 0.0 {
        let thumb_h = ((visible_h / total_content_h) * visible_h).clamp(24.0, visible_h);
        let scroll_ratio = (*scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = editor_rect.min.y + scroll_ratio * (visible_h - thumb_h);
        let track_x = editor_rect.max.x - 4.0;
        let thumb_rect = Rect::from_min_size(pos2(track_x, thumb_y), vec2(3.0, thumb_h));
        editor_painter.rect_filled(thumb_rect, 1.5, Color32::from_rgba_unmultiplied(120, 125, 140, 60));
    }
}
