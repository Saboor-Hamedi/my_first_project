//! Horizontal divider rule element rendering.

use crate::theme::Theme;
use eframe::egui::{pos2, Painter, Stroke};

/// Renders a crisp horizontal rule divider when the line is inactive.
pub fn render_horizontal_rule(
    painter: &Painter,
    line_y: f32,
    line_h: f32,
    text_left: f32,
    content_right: f32,
    theme: &Theme,
    is_active: bool,
) {
    if !is_active {
        let mid_y = (line_y + line_h * 0.5).round();
        painter.line_segment(
            [pos2(text_left - 4.0, mid_y), pos2(content_right - 8.0, mid_y)],
            Stroke::new(1.0, theme.border()),
        );
    }
}
