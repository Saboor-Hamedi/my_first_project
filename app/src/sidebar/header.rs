//! Sidebar header component: MindForge branding and Stats navigation.

use eframe::egui::{self, vec2, Align2, Color32, FontId, Rect};
use crate::sidebar::SidebarAction;

/// Renders the top header of the sidebar: branding, shortcut hint, and Stats view button.
pub fn render_sidebar_header(
    ui: &egui::Ui,
    painter: &egui::Painter,
    sb_origin: egui::Pos2,
    sidebar_w: f32,
    active_mode_idx: usize,
    accent: Color32,
    text_color: Color32,
) -> Option<SidebarAction> {
    let mut action = None;

    // 1. Branding Title
    painter.text(
        sb_origin,
        Align2::LEFT_TOP,
        "MINDFORGE",
        FontId::monospace(15.0),
        accent,
    );

    // 2. Subtitle: Clean toggle hint (no verbose navigating tags)
    painter.text(
        sb_origin + vec2(0.0, 20.0),
        Align2::LEFT_TOP,
        "Ctrl+B to toggle",
        FontId::monospace(10.5),
        Color32::from_gray(110),
    );

    // 3. Navigation: Only 📊 Stats (Notes is handled directly by Documents explorer below)
    let btn_rect = Rect::from_min_size(
        sb_origin + vec2(0.0, 46.0),
        vec2(sidebar_w - 32.0, 24.0),
    );
    let is_sel = active_mode_idx == 1;
    let is_hovered = ui.rect_contains_pointer(btn_rect);

    if is_hovered || is_sel {
        painter.rect_filled(
            btn_rect,
            4.0,
            if is_sel {
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35)
            } else {
                Color32::from_rgb(20, 20, 24)
            },
        );
        if is_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            action = Some(SidebarAction::SwitchMode(1));
        }
    }

    painter.text(
        btn_rect.min + vec2(8.0, 4.0),
        Align2::LEFT_TOP,
        "📊 Stats",
        FontId::monospace(12.0),
        if is_sel { accent } else { text_color },
    );

    action
}
