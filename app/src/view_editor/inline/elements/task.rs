//! Interactive task checkbox element rendering.

use crate::theme::Theme;
use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Stroke};

/// Renders an interactive vector checkbox for markdown task list items (- [ ] / - [x]).
pub fn render_task_checkbox(
    painter: &Painter,
    box_rect: Rect,
    checked: bool,
    mouse_pos: Option<Pos2>,
    theme: &Theme,
) {
    let is_hovered = mouse_pos.map_or(false, |p| box_rect.expand(4.0).contains(p));
    let radius = 3.5;

    if checked {
        // Filled accent box
        painter.rect_filled(box_rect, radius, theme.accent);
        // High-contrast checkmark
        let check_color = if theme.is_light() { Color32::WHITE } else { theme.bg };
        let p1 = pos2(box_rect.min.x + 3.0, box_rect.min.y + 7.0);
        let p2 = pos2(box_rect.min.x + 6.0, box_rect.min.y + 10.5);
        let p3 = pos2(box_rect.min.x + 11.0, box_rect.min.y + 3.5);
        painter.line_segment([p1, p2], Stroke::new(1.8, check_color));
        painter.line_segment([p2, p3], Stroke::new(1.8, check_color));
    } else {
        // Empty outlined box with smooth hover glow
        let border_color = if is_hovered { theme.accent } else { theme.border() };
        let fill_color = if is_hovered {
            Color32::from_rgba_unmultiplied(
                theme.accent.r(),
                theme.accent.g(),
                theme.accent.b(),
                30,
            )
        } else {
            Color32::TRANSPARENT
        };
        painter.rect(
            box_rect,
            radius,
            fill_color,
            Stroke::new(1.2, border_color),
            eframe::egui::StrokeKind::Inside,
        );
    }
}
