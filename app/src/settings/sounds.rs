//! Sound profile settings tab.

use crate::sound::{SoundEngine, SoundProfile};
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

/// Waveform bar heights representing audio character of each profile.
pub fn sound_wave_heights(profile: SoundProfile) -> [f32; 7] {
    match profile {
        SoundProfile::Off    => [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        SoundProfile::Thocky => [3.0, 6.0, 9.0, 11.0, 8.0, 5.0, 2.0],
        SoundProfile::Clacky => [4.0, 9.0, 12.0, 10.0, 7.0, 4.0, 2.0],
        SoundProfile::Creamy => [2.0, 5.0, 8.0, 10.0, 8.0, 5.0, 2.0],
        SoundProfile::Marbly => [5.0, 10.0, 13.0, 10.0, 6.0, 3.0, 1.0],
        SoundProfile::Poppy  => [6.0, 11.0, 13.0, 9.0, 5.0, 2.0, 1.0],
        SoundProfile::Clicky => [4.0, 8.0, 13.0, 12.0, 8.0, 4.0, 2.0],
    }
}

pub fn render_sounds_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    sound: &mut SoundEngine,
    theme: &Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "MECHANICAL KEYBOARD SOUNDS",
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Acoustic profiles synthesized with 0ms latency",
        FontId::monospace(11.5),
        theme.muted,
    );

    // Mute All toggle in top-right of section
    let mute_btn_w = 90.0;
    let mute_btn = Rect::from_min_size(
        pos2(panel_rect.max.x - mute_btn_w - 28.0, p_origin.y),
        vec2(mute_btn_w, 24.0),
    );
    let mute_hover = ui.rect_contains_pointer(mute_btn);
    let is_muted = sound.profile == SoundProfile::Off;

    if mute_hover && ui.input(|i| i.pointer.primary_clicked()) {
        if is_muted {
            sound.profile = SoundProfile::Thocky;
            on_save_setting("sound", "thocky");
        } else {
            sound.profile = SoundProfile::Off;
            on_save_setting("sound", "off");
        }
    }

    painter.rect(
        mute_btn,
        4.0,
        if is_muted {
            Color32::from_rgb(45, 20, 20)
        } else if mute_hover {
            Color32::from_rgb(28, 28, 34)
        } else {
            Color32::from_rgb(20, 20, 26)
        },
        Stroke::new(1.0, if is_muted { Color32::from_rgb(160, 50, 50) } else { Color32::from_gray(50) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        mute_btn.center(),
        Align2::CENTER_CENTER,
        if is_muted { "🔇 Muted" } else { "🔊 Mute" },
        FontId::monospace(10.5),
        if is_muted { Color32::from_rgb(220, 80, 80) } else { Color32::from_gray(160) },
    );

    // ── Sound Chips ───────────────────────────────────────────────────
    let sound_start_y = p_origin.y + 58.0;
    let s_col_w = 130.0;
    let s_row_h = 50.0; // taller chips for waveform

    for (i, &profile) in SoundProfile::ALL.iter().enumerate() {
        let col = i % 3;
        let row = i / 3;
        let s_rect = Rect::from_min_size(
            pos2(
                p_origin.x + col as f32 * (s_col_w + 10.0),
                sound_start_y + row as f32 * (s_row_h + 10.0),
            ),
            vec2(s_col_w, s_row_h),
        );
        let is_sel = sound.profile == profile;
        let s_hovered = ui.rect_contains_pointer(s_rect);

        let bg = if is_sel {
            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35)
        } else if s_hovered {
            Color32::from_rgb(22, 22, 28)
        } else {
            Color32::from_rgb(16, 17, 21)
        };
        let border = if is_sel {
            theme.accent
        } else if s_hovered {
            Color32::from_gray(75)
        } else {
            Color32::from_gray(35)
        };

        painter.rect(s_rect, 5.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);

        // Accent top-border stripe on selected
        if is_sel {
            let stripe = Rect::from_min_size(s_rect.min, vec2(s_rect.width(), 2.5));
            painter.rect_filled(stripe, egui::CornerRadius { nw: 5, ne: 5, sw: 0, se: 0 }, theme.accent);
        }

        // Mini waveform bars
        let bars = sound_wave_heights(profile);
        let bar_area_w = 52.0;
        let bar_w = 5.0;
        let bar_gap = 2.0;
        let bar_base_y = s_rect.center().y + 6.0;
        let bar_x0 = s_rect.min.x + (s_col_w - bar_area_w) * 0.5;
        let wave_color = if is_sel { theme.accent } else { Color32::from_gray(70) };

        for (bi, &h) in bars.iter().enumerate() {
            let bx = bar_x0 + bi as f32 * (bar_w + bar_gap);
            painter.rect_filled(
                Rect::from_min_size(pos2(bx, bar_base_y - h), vec2(bar_w, h)),
                1.5,
                wave_color,
            );
        }

        // Profile name above bars
        painter.text(
            pos2(s_rect.center().x, s_rect.min.y + 14.0),
            Align2::CENTER_CENTER,
            profile.name(),
            FontId::monospace(11.5),
            if is_sel { theme.accent } else { Color32::from_gray(190) },
        );

        if s_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            sound.profile = profile;
            on_save_setting("sound", profile.name().to_lowercase().as_str());
        }
    }

    // ── Description card ─────────────────────────────────────────────
    let rows_used = (SoundProfile::ALL.len() + 2) / 3; // ceil
    let desc_y = sound_start_y + rows_used as f32 * (s_row_h + 10.0) + 4.0;
    let desc_rect = Rect::from_min_size(
        pos2(p_origin.x, desc_y),
        vec2(panel_rect.width() - 56.0, 28.0),
    );
    painter.rect(
        desc_rect,
        4.0,
        Color32::from_rgb(18, 20, 26),
        Stroke::new(1.0, Color32::from_rgb(30, 32, 42)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        pos2(desc_rect.min.x + 12.0, desc_rect.center().y),
        Align2::LEFT_CENTER,
        format!("{}  —  {}", sound.profile.name(), sound.profile.description()),
        FontId::monospace(11.5),
        Color32::from_gray(190),
    );
}
