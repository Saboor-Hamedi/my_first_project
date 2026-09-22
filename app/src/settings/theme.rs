//! Appearance & theme settings tab.

use crate::theme::{Theme, ThemeKind};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

pub fn render_theme_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    theme: &mut Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "APPEARANCE & THEME",
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Curated developer palettes that transform the entire interface",
        FontId::monospace(11.5),
        theme.muted,
    );

    let theme_start_y = p_origin.y + 54.0;
    let col_gap = 10.0;
    let row_gap = 8.0;
    let card_w = (panel_rect.width() - 56.0 - col_gap) * 0.5;
    let card_h = 58.0;

    for (i, &t_kind) in ThemeKind::ALL.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let t_preset = Theme::from_kind(t_kind);
        let t_rect = Rect::from_min_size(
            pos2(
                p_origin.x + col as f32 * (card_w + col_gap),
                theme_start_y + row as f32 * (card_h + row_gap),
            ),
            vec2(card_w, card_h),
        );

        let is_sel = theme.kind == t_kind;
        let hovered = ui.rect_contains_pointer(t_rect);

        let bg = if is_sel {
            Color32::from_rgba_unmultiplied(t_preset.accent.r(), t_preset.accent.g(), t_preset.accent.b(), 26)
        } else if hovered {
            Color32::from_rgb(22, 24, 30)
        } else {
            Color32::from_rgb(15, 16, 21)
        };

        let border = if is_sel {
            Stroke::new(1.5, t_preset.accent)
        } else if hovered {
            Stroke::new(1.0, Color32::from_gray(75))
        } else {
            Stroke::new(1.0, Color32::from_rgb(32, 35, 45))
        };

        painter.rect(t_rect, 6.0, bg, border, egui::StrokeKind::Inside);

        // Top accent stripe on selected card
        if is_sel {
            let stripe = Rect::from_min_size(t_rect.min, vec2(t_rect.width(), 2.5));
            painter.rect_filled(stripe, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, t_preset.accent);
        }

        // Header line: Theme Name (left) and Status / Checkmark (right)
        painter.text(
            pos2(t_rect.min.x + 12.0, t_rect.min.y + 11.0),
            Align2::LEFT_TOP,
            t_kind.display_name(),
            FontId::monospace(12.0),
            if is_sel { t_preset.accent } else { Color32::from_gray(215) },
        );

        if is_sel {
            painter.text(
                pos2(t_rect.max.x - 12.0, t_rect.min.y + 11.0),
                Align2::RIGHT_TOP,
                "● ACTIVE",
                FontId::monospace(10.0),
                t_preset.accent,
            );
        }

        // Palette preview swatches: bg, text, accent, highlight
        let swatches = [
            t_preset.bg,
            t_preset.text,
            t_preset.accent,
            t_preset.highlight,
        ];
        let sw_y = t_rect.min.y + 32.0;
        let sw_h = 15.0;
        let sw_w = ((card_w - 24.0) - 3.0 * 6.0) / 4.0;

        for (s_idx, color) in swatches.iter().enumerate() {
            let sw_x = t_rect.min.x + 12.0 + s_idx as f32 * (sw_w + 6.0);
            let sw_rect = Rect::from_min_size(pos2(sw_x, sw_y), vec2(sw_w, sw_h));
            painter.rect_filled(sw_rect, 3.0, *color);
            painter.rect(
                sw_rect,
                3.0,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
                egui::StrokeKind::Inside,
            );
        }

        if hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            *theme = Theme::from_kind(t_kind);
            on_save_setting("theme", t_kind.name());
        }
    }
}
