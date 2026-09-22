//! Caret styles and physics settings tab.

use crate::caret::{Caret, CaretKind};
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

/// Per-style accent colors shown as indicator dots on each caret chip.
pub fn caret_dot_color(kind: CaretKind, accent: Color32) -> Color32 {
    match kind {
        CaretKind::Block     => accent,
        CaretKind::Beam      => accent,
        CaretKind::Underline => accent,
        CaretKind::Candle    => Color32::from_rgb(255, 190, 70),
        CaretKind::Fire      => Color32::from_rgb(255, 140, 30),
        CaretKind::Water     => Color32::from_rgb(65, 175, 255),
        CaretKind::Snow      => Color32::from_rgb(230, 245, 255),
        CaretKind::Electric  => Color32::from_rgb(190, 220, 255),
        CaretKind::Comet     => Color32::from_rgb(200, 200, 255),
        CaretKind::Rainbow   => Color32::from_rgb(255, 80, 200),
        CaretKind::Matrix    => Color32::from_rgb(40, 255, 90),
        CaretKind::Ice       => Color32::from_rgb(130, 230, 255),
        CaretKind::Glitch    => Color32::from_rgb(255, 50, 100),
        CaretKind::Neon      => Color32::from_rgb(180, 255, 180),
        CaretKind::Heartbeat => Color32::from_rgb(255, 60, 100),
    }
}

pub fn render_carets_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    caret: &mut Caret,
    theme: &Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "CARET CUSTOMIZATION",
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        format!("Choose from {} animated styles & particle effects", CaretKind::ALL.len()),
        FontId::monospace(11.5),
        theme.muted,
    );

    // ── Chip Grid ────────────────────────────────────────────────────
    let chip_start_y = p_origin.y + 52.0;
    let col_w = 96.0;
    let row_h = 38.0; // taller chips
    let col_gap = 8.0;
    let row_gap = 8.0;

    for (idx, &kind) in CaretKind::ALL.iter().enumerate() {
        let col = idx % 4;
        let row = idx / 4;
        let chip_rect = Rect::from_min_size(
            pos2(
                p_origin.x + col as f32 * (col_w + col_gap),
                chip_start_y + row as f32 * (row_h + row_gap),
            ),
            vec2(col_w, row_h),
        );
        let is_selected = caret.kind == kind;
        let hovered = ui.rect_contains_pointer(chip_rect);
        let dot_color = caret_dot_color(kind, theme.accent);

        // Chip background — no border, use bg fill only
        let bg = if is_selected {
            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35)
        } else if hovered {
            Color32::from_rgb(22, 22, 28)
        } else {
            Color32::from_rgb(16, 17, 21)
        };

        painter.rect_filled(chip_rect, 5.0, bg);

        // Accent top-border stripe on selected chip
        if is_selected {
            let stripe = Rect::from_min_size(
                chip_rect.min,
                vec2(chip_rect.width(), 2.5),
            );
            painter.rect_filled(stripe, egui::CornerRadius { nw: 5, ne: 5, sw: 0, se: 0 }, theme.accent);
        }

        // Color indicator dot (top-right of chip)
        painter.circle_filled(
            pos2(chip_rect.max.x - 9.0, chip_rect.min.y + 9.0),
            4.0,
            dot_color,
        );

        // Label
        painter.text(
            chip_rect.center() + vec2(0.0, 2.0),
            Align2::CENTER_CENTER,
            kind.name(),
            FontId::monospace(11.0),
            if is_selected { theme.accent } else { Color32::from_gray(195) },
        );

        if hovered && ui.input(|i| i.pointer.primary_clicked()) {
            caret.kind = kind;
            on_save_setting("caret", kind.name());
        }
    }

    // ── Description card ─────────────────────────────────────────────
    let rows = (CaretKind::ALL.len() + 3) / 4;
    let desc_y = chip_start_y + rows as f32 * (row_h + row_gap) + 6.0;
    let desc_rect = Rect::from_min_size(
        pos2(p_origin.x, desc_y),
        vec2(panel_rect.width() - 56.0, 26.0),
    );
    painter.rect(
        desc_rect,
        4.0,
        Color32::from_rgb(18, 20, 26),
        Stroke::new(1.0, Color32::from_rgb(32, 34, 44)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        pos2(desc_rect.min.x + 10.0, desc_rect.center().y),
        Align2::LEFT_CENTER,
        format!("{}  —  {}", caret.kind.name().to_uppercase(), caret.kind.description()),
        FontId::monospace(11.5),
        Color32::from_gray(195),
    );

    // ── Animations pill toggle (compact) ─────────────────────────────
    let toggle_y = desc_y + 32.0;
    let pill_w = 42.0;
    let pill_h = 22.0;
    let pill_rect = Rect::from_min_size(pos2(p_origin.x, toggle_y), vec2(pill_w, pill_h));
    let pill_hover = ui.rect_contains_pointer(pill_rect);
    let anim_on = caret.animations_enabled;

    if pill_hover && ui.input(|i| i.pointer.primary_clicked()) {
        caret.animations_enabled = !caret.animations_enabled;
        on_save_setting("caret_animations", if caret.animations_enabled { "on" } else { "off" });
    }

    // Pill track
    painter.rect(
        pill_rect,
        pill_h * 0.5,
        if anim_on {
            if pill_hover {
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 55)
            } else {
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35)
            }
        } else {
            if pill_hover { Color32::from_rgb(30, 30, 38) } else { Color32::from_rgb(20, 20, 26) }
        },
        Stroke::new(1.0, if anim_on { theme.accent } else { Color32::from_gray(55) }),
        egui::StrokeKind::Inside,
    );
    // Sliding knob
    let knob_r = (pill_h * 0.5) - 3.0;
    let knob_x = if anim_on {
        pill_rect.max.x - knob_r - 4.0
    } else {
        pill_rect.min.x + knob_r + 4.0
    };
    painter.circle_filled(
        pos2(knob_x, pill_rect.center().y),
        knob_r,
        if anim_on { theme.accent } else { Color32::from_gray(80) },
    );
    // "Animations" label to the right
    painter.text(
        pos2(pill_rect.max.x + 10.0, pill_rect.center().y),
        Align2::LEFT_CENTER,
        "Animations",
        FontId::monospace(11.5),
        Color32::from_gray(170),
    );

    // ── Width Slider ──────────────────────────────────────────────────
    let slider_y = toggle_y + 40.0;
    painter.text(
        pos2(p_origin.x, slider_y + 10.0),
        Align2::LEFT_CENTER,
        "Width",
        FontId::monospace(12.0),
        Color32::from_gray(175),
    );

    let slider_x = p_origin.x + 58.0;
    let slider_w = 180.0;
    let slider_h = 20.0;
    let slider_rect = Rect::from_min_size(pos2(slider_x, slider_y), vec2(slider_w, slider_h));

    let slider_hover = ui.rect_contains_pointer(slider_rect);
    let is_down = ui.input(|i| i.pointer.primary_down() || i.pointer.primary_clicked());
    if slider_hover && is_down {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let t = ((pos.x - slider_rect.min.x) / slider_rect.width()).clamp(0.0, 1.0);
            let new_val = (1.0 + t * 9.0).round().clamp(1.0, 10.0);
            if (caret.width - new_val).abs() > 0.01 {
                caret.width = new_val;
                on_save_setting("caret_width", &format!("{:.0}", caret.width));
            }
        }
    }

    // Track
    let track_y = slider_rect.center().y;
    let track_h = 5.0;
    let track_rect = Rect::from_min_size(
        pos2(slider_rect.min.x, track_y - track_h * 0.5),
        vec2(slider_w, track_h),
    );
    painter.rect_filled(track_rect, 3.0, Color32::from_rgb(26, 28, 34));

    let t = ((caret.width - 1.0) / 9.0).clamp(0.0, 1.0);
    let active_w = slider_w * t;
    if active_w > 0.0 {
        let active_rect = Rect::from_min_size(
            pos2(slider_rect.min.x, track_y - track_h * 0.5),
            vec2(active_w, track_h),
        );
        painter.rect_filled(active_rect, 3.0, theme.accent);
    }

    // Thumb knob
    let thumb_x = slider_rect.min.x + active_w;
    painter.circle_filled(
        pos2(thumb_x, track_y),
        if slider_hover { 8.0 } else { 7.0 },
        if slider_hover { Color32::WHITE } else { theme.accent },
    );
    painter.circle_stroke(
        pos2(thumb_x, track_y),
        if slider_hover { 8.0 } else { 7.0 },
        Stroke::new(1.5, Color32::from_rgb(14, 16, 20)),
    );

    // Live numeric label next to thumb
    painter.text(
        pos2(slider_rect.max.x + 12.0, track_y),
        Align2::LEFT_CENTER,
        format!("{:.0}px", caret.width),
        FontId::monospace(11.5),
        theme.accent,
    );
}
