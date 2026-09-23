//! Markdown table rendering and cell column grid layout.

use crate::theme::Theme;
use eframe::egui::{pos2, Color32, Painter, Rect, Stroke};

/// Renders table row grid lines and background highlighting for markdown tables.
pub fn render_table_row_decorations(
    painter: &Painter,
    line_y: f32,
    line_h: f32,
    text_left: f32,
    content_right: f32,
    theme: &Theme,
    is_header: bool,
    is_separator: bool,
    is_active: bool,
) {
    let row_rect = Rect::from_min_max(
        pos2(text_left - 6.0, line_y),
        pos2(content_right - 8.0, line_y + line_h),
    );

    if is_separator {
        // Draw crisp horizontal divider line across the table columns
        let mid_y = line_y + line_h * 0.5;
        painter.line_segment(
            [pos2(text_left - 6.0, mid_y), pos2(content_right - 8.0, mid_y)],
            Stroke::new(1.2, theme.border()),
        );
        return;
    }

    if is_header && !is_active {
        // Elevated header background pill
        let header_bg = Color32::from_rgba_unmultiplied(
            theme.accent.r(),
            theme.accent.g(),
            theme.accent.b(),
            18,
        );
        painter.rect_filled(row_rect, 3.0, header_bg);

        // Header bottom border
        let bottom_y = line_y + line_h;
        painter.line_segment(
            [pos2(text_left - 6.0, bottom_y), pos2(content_right - 8.0, bottom_y)],
            Stroke::new(1.5, theme.accent),
        );
    } else if !is_active {
        // Subtle row bottom hairline divider
        let bottom_y = line_y + line_h;
        let divider_color = Color32::from_rgba_unmultiplied(
            theme.border().r(),
            theme.border().g(),
            theme.border().b(),
            80,
        );
        painter.line_segment(
            [pos2(text_left - 6.0, bottom_y), pos2(content_right - 8.0, bottom_y)],
            Stroke::new(0.8, divider_color),
        );
    }
}
