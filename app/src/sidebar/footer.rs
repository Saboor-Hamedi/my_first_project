//! Sidebar footer component: Round settings icon button with shortcut tooltip.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};
use crate::sidebar::SidebarAction;

/// Renders the bottom footer of the sidebar: round settings icon button with hover tooltip.
pub fn render_sidebar_footer(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    sb_rect: Rect,
    accent: Color32,
    muted_color: Color32,
) -> Option<SidebarAction> {
    let mut action = None;

    let btn_size = 30.0;
    let btn_rect = Rect::from_min_size(
        pos2(sb_rect.min.x + 14.0, sb_rect.max.y - btn_size - 10.0),
        vec2(btn_size, btn_size),
    );

    let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
    let center = btn_rect.center();
    let is_hovered = resp.hovered() || ui.rect_contains_pointer(btn_rect);

    if is_hovered {
        painter.circle_filled(center, btn_size * 0.5, Color32::from_rgb(28, 30, 38));
        painter.circle_stroke(
            center,
            btn_size * 0.5,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 100)),
        );
    } else {
        painter.circle_filled(center, btn_size * 0.5, Color32::from_rgb(18, 19, 24));
        painter.circle_stroke(
            center,
            btn_size * 0.5,
            Stroke::new(1.0, Color32::from_rgb(34, 36, 44)),
        );
    }

    // Large gear icon centered without text label
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "⚙",
        FontId::monospace(15.5),
        if is_hovered { accent } else { muted_color },
    );

    // Hover tooltip showing shortcut
    let resp = resp.on_hover_text("Settings (Ctrl+,)");

    if resp.clicked() || (is_hovered && ui.input(|inp| inp.pointer.primary_clicked())) {
        action = Some(SidebarAction::OpenSettings);
    }

    action
}
