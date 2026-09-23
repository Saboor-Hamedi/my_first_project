//! Shared, highly-polished reusable UI components.

use eframe::egui::{self, pos2, vec2, Color32, Id, Pos2, Rect, Stroke};
use crate::theme::Theme;

/// Renders an ultra-sleek, electron/macOS grade close button with 5px corner radius.
/// Features smooth animated hover transition, crisp anti-aliased cross lines,
/// and pointing hand cursor. Returns `true` if clicked.
pub fn render_close_button(
    ui: &egui::Ui,
    painter: &egui::Painter,
    center: Pos2,
    size: f32,
    theme: &Theme,
    id_salt: impl std::hash::Hash,
) -> bool {
    let hit_rect = Rect::from_center_size(center, vec2(size, size));
    render_close_button_rect(ui, painter, hit_rect, theme, id_salt)
}

/// Renders a close button fitting the exact rectangle provided with 5px corner radius.
pub fn render_close_button_rect(
    ui: &egui::Ui,
    painter: &egui::Painter,
    hit_rect: Rect,
    theme: &Theme,
    id_salt: impl std::hash::Hash,
) -> bool {
    let hovered = ui.rect_contains_pointer(hit_rect);
    let hover_t = ui.ctx().animate_bool(Id::new(("close_btn_hover", id_salt)), hovered);

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Modern 5px rounded rectangle background on hover
    if hover_t > 0.01 {
        let (r, g, b) = if theme.is_light() {
            (215, 50, 50)
        } else {
            (235, 65, 65)
        };
        let bg_color = Color32::from_rgba_unmultiplied(
            r,
            g,
            b,
            (42.0 * hover_t) as u8,
        );
        painter.rect_filled(hit_rect, 5.0, bg_color);
    }

    // Interpolate cross icon color from muted to crisp highlight / danger
    let base_color = theme.muted;
    let target_color = if theme.is_light() {
        Color32::from_rgb(205, 30, 30)
    } else {
        Color32::from_rgb(255, 120, 120)
    };
    let cross_color = lerp_color(base_color, target_color, hover_t);

    // Anti-aliased geometric cross lines with subpixel crispness
    let center = hit_rect.center();
    let min_dim = hit_rect.width().min(hit_rect.height());
    let arm = (min_dim * 0.23).round().max(3.5);
    let stroke = Stroke::new(1.3, cross_color);
    painter.line_segment(
        [pos2(center.x - arm, center.y - arm), pos2(center.x + arm, center.y + arm)],
        stroke,
    );
    painter.line_segment(
        [pos2(center.x + arm, center.y - arm), pos2(center.x - arm, center.y + arm)],
        stroke,
    );

    hovered && ui.input(|i| i.pointer.primary_clicked())
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}
