//! Document title bar and borderless window drag gripper.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

/// Renders the full-width modern titlebar across the entire window top.
/// Displays Codex icon, MindForge branding, active document title, drag gripper, and window controls.
pub fn render_full_titlebar(
    ui: &egui::Ui,
    painter: &egui::Painter,
    titlebar_rect: Rect,
    active_title: &str,
    is_dirty: bool,
    theme: &Theme,
) {
    // 1. Sleek card surface with 5px radius and subtle 1px border
    painter.rect(
        titlebar_rect,
        5.0,
        Color32::from_rgb(13, 14, 18),
        Stroke::new(1.0, Color32::from_rgb(32, 34, 40)),
        egui::StrokeKind::Inside,
    );

    let center_y = titlebar_rect.center().y;

    // 2. Codex Icon (📖 / book glyph)
    painter.text(
        pos2(titlebar_rect.min.x + 12.0, center_y),
        Align2::LEFT_CENTER,
        "📖",
        FontId::monospace(14.0),
        theme.accent,
    );

    // 3. MINDFORGE branding
    painter.text(
        pos2(titlebar_rect.min.x + 32.0, center_y),
        Align2::LEFT_CENTER,
        "MINDFORGE",
        FontId::monospace(12.5),
        theme.highlight,
    );

    // 4. Subtle separator
    painter.text(
        pos2(titlebar_rect.min.x + 122.0, center_y),
        Align2::LEFT_CENTER,
        "│",
        FontId::monospace(12.0),
        Color32::from_gray(55),
    );

    // 5. Active Document / Section Title
    let max_avail_w = (titlebar_rect.width() - 320.0).max(80.0);
    let max_chars = (max_avail_w / 7.5) as usize;
    let base_title = if active_title.len() > max_chars {
        format!("{}...", &active_title[..max_chars.saturating_sub(3)])
    } else {
        active_title.to_string()
    };
    let display_title = if is_dirty {
        format!("{}  ●", base_title)
    } else {
        base_title
    };
    painter.text(
        pos2(titlebar_rect.min.x + 136.0, center_y),
        Align2::LEFT_CENTER,
        display_title,
        FontId::monospace(11.5),
        Color32::from_gray(190),
    );

    // 6. Right side: Drag gripper & Window Controls (Minimize, Maximize/Restore, Close)
    render_window_controls(ui, painter, titlebar_rect, theme);
}



/// Renders modern window controls (Minimize, Maximize/Restore, Close) with drag support.
pub fn render_window_controls(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    theme: &Theme,
) {
    let btn_w = 32.0;
    let btn_h = 24.0;
    let top_y = bounds.min.y + 7.0;
    let right_x = bounds.max.x - 6.0;

    // 1. Drag gripper immediately to the left of window control buttons
    let gripper_rect = Rect::from_min_size(
        pos2(right_x - (btn_w * 3.0) - 22.0, top_y + 1.0),
        vec2(16.0, btn_h),
    );
    let is_gripper_hovered = ui.rect_contains_pointer(gripper_rect);
    if is_gripper_hovered && ui.input(|i| i.pointer.primary_down()) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    let grip_color = if is_gripper_hovered {
        theme.accent
    } else {
        Color32::from_gray(65)
    };
    for col in 0..2 {
        for row in 0..3 {
            let cx = gripper_rect.min.x + 4.0 + col as f32 * 6.5;
            let cy = gripper_rect.min.y + 4.5 + row as f32 * 5.0;
            painter.circle_filled(pos2(cx, cy), 1.2, grip_color);
        }
    }

    // 2. Minimize Button (—)
    let min_rect = Rect::from_min_size(pos2(right_x - btn_w * 3.0, top_y), vec2(btn_w, btn_h));
    let min_hovered = ui.rect_contains_pointer(min_rect);
    if min_hovered {
        painter.rect_filled(min_rect, 4.0, Color32::from_rgba_unmultiplied(255, 255, 255, 24));
        if ui.input(|i| i.pointer.primary_clicked()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }
    let min_stroke_color = if min_hovered { theme.text } else { Color32::from_gray(150) };
    let min_mid_y = (min_rect.min.y + min_rect.max.y) * 0.5 + 2.5;
    painter.line_segment(
        [
            pos2(min_rect.center().x - 5.0, min_mid_y),
            pos2(min_rect.center().x + 5.0, min_mid_y),
        ],
        Stroke::new(1.5, min_stroke_color),
    );

    // 3. Maximize / Restore Button (□)
    let max_rect = Rect::from_min_size(pos2(right_x - btn_w * 2.0, top_y), vec2(btn_w, btn_h));
    let max_hovered = ui.rect_contains_pointer(max_rect);
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if max_hovered {
        painter.rect_filled(max_rect, 4.0, Color32::from_rgba_unmultiplied(255, 255, 255, 24));
        if ui.input(|i| i.pointer.primary_clicked()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
        }
    }
    let max_stroke_color = if max_hovered { theme.text } else { Color32::from_gray(150) };
    let c = max_rect.center();
    if is_maximized {
        let s = 4.2;
        // Background window (offset up-right)
        painter.rect_stroke(
            Rect::from_center_size(pos2(c.x + 1.8, c.y - 1.8), vec2(s * 2.0 - 1.0, s * 2.0 - 1.0)),
            1.0,
            Stroke::new(1.2, max_stroke_color),
            egui::StrokeKind::Inside,
        );
        // Foreground window mask + stroke
        painter.rect_filled(
            Rect::from_center_size(pos2(c.x - 1.8, c.y + 1.8), vec2(s * 2.0, s * 2.0)),
            1.0,
            theme.bg,
        );
        painter.rect_stroke(
            Rect::from_center_size(pos2(c.x - 1.8, c.y + 1.8), vec2(s * 2.0, s * 2.0)),
            1.0,
            Stroke::new(1.2, max_stroke_color),
            egui::StrokeKind::Inside,
        );
    } else {
        let s = 4.8;
        painter.rect_stroke(
            Rect::from_center_size(c, vec2(s * 2.0, s * 2.0)),
            1.0,
            Stroke::new(1.3, max_stroke_color),
            egui::StrokeKind::Inside,
        );
    }

    // 4. Close Button (✕)
    let close_rect = Rect::from_min_size(pos2(right_x - btn_w, top_y), vec2(btn_w, btn_h));
    let close_hovered = ui.rect_contains_pointer(close_rect);
    if close_hovered {
        painter.rect_filled(close_rect, 4.0, Color32::from_rgb(225, 45, 57));
        if ui.input(|i| i.pointer.primary_clicked()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    let close_stroke_color = if close_hovered { Color32::WHITE } else { Color32::from_gray(150) };
    let c = close_rect.center();
    let d = 4.2;
    painter.line_segment(
        [pos2(c.x - d, c.y - d), pos2(c.x + d, c.y + d)],
        Stroke::new(1.5, close_stroke_color),
    );
    painter.line_segment(
        [pos2(c.x + d, c.y - d), pos2(c.x - d, c.y + d)],
        Stroke::new(1.5, close_stroke_color),
    );
}
