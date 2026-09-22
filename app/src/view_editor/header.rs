//! Document title bar and borderless window drag gripper.

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
    is_dirty: bool,
    theme: &Theme,
) {
    let header_y = bounds.min.y + 14.0;
    let base_title = if active_title.len() > 40 {
        format!("📄 {}...", &active_title[..40])
    } else {
        format!("📄 {}", active_title)
    };
    let display_title = if is_dirty {
        format!("{}  ●", base_title)
    } else {
        base_title
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

/// Renders symmetrical headers for both editor and preview side-by-side.
pub fn render_split_editor_header(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    left_rect: Rect,
    right_rect: Rect,
    active_title: &str,
    is_dirty: bool,
    theme: &Theme,
) {
    let header_y = bounds.min.y + 14.0;
    let divider_line_y = bounds.min.y + 38.0;

    // ── Left Pane Header (Editor) ────────────────────────────────────────────
    let base_title = if active_title.len() > 30 {
        format!("📄 {}...", &active_title[..30])
    } else {
        format!("📄 {}", active_title)
    };
    let display_title = if is_dirty {
        format!("{}  ●", base_title)
    } else {
        base_title
    };

    painter.text(
        pos2(left_rect.min.x, header_y),
        Align2::LEFT_TOP,
        display_title,
        FontId::monospace(13.5),
        theme.highlight,
    );

    painter.line_segment(
        [
            pos2(left_rect.min.x, divider_line_y),
            pos2(left_rect.max.x, divider_line_y),
        ],
        Stroke::new(1.0, Color32::from_gray(24)),
    );

    // ── Right Pane Header (Preview) ──────────────────────────────────────────
    let preview_title = format!("👁 Preview: {}", active_title);
    let preview_display = if preview_title.len() > 32 {
        format!("{}...", &preview_title[..32])
    } else {
        preview_title
    };

    painter.text(
        pos2(right_rect.min.x + 8.0, header_y),
        Align2::LEFT_TOP,
        preview_display,
        FontId::monospace(13.5),
        theme.accent,
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

    painter.line_segment(
        [
            pos2(right_rect.min.x, divider_line_y),
            pos2(right_rect.max.x, divider_line_y),
        ],
        Stroke::new(1.0, Color32::from_gray(24)),
    );
}
