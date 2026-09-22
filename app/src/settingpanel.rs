//! Settings modal right content panel renderer — redesigned for premium UX.

use crate::app::EditorInputMode;
use crate::caret::{Caret, CaretKind};
use crate::settingtabs::SettingTab;
use crate::sound::{SoundEngine, SoundProfile};
use crate::theme::{Theme, ThemeKind};
use crate::updater::{UpdateManager, UpdateStatus};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

// ─── Per-style accent colors shown as indicator dots on each caret chip ───────
fn caret_dot_color(kind: CaretKind, accent: Color32) -> Color32 {
    match kind {
        CaretKind::Block     => accent,
        CaretKind::Beam      => accent,
        CaretKind::Underline => accent,
        CaretKind::Fire      => Color32::from_rgb(255, 130, 20),
        CaretKind::Water     => Color32::from_rgb(60, 170, 255),
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

// ─── Waveform bar heights representing audio character of each profile ─────────
fn sound_wave_heights(profile: SoundProfile) -> [f32; 7] {
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

pub enum SettingPanelAction {
    TriggerBackup,
    CheckUpdates,
    DownloadUpdate,
    RestartToApply,
}

pub fn render_setting_panel(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    active_tab: SettingTab,
    editor_input_mode: &mut EditorInputMode,
    caret: &mut Caret,
    sound: &mut SoundEngine,
    theme: &mut Theme,
    backup_dir: &mut String,
    last_backup_status: Option<&str>,
    updater: &UpdateManager,
    on_save_setting: &mut dyn FnMut(&str, &str),
) -> Option<SettingPanelAction> {
    let mut action = None;
    // Clip all painting strictly to the panel rect — nothing bleeds over modal border
    let painter = painter.with_clip_rect(panel_rect.shrink(1.0));
    let painter = &painter;
    let p_origin = panel_rect.min + vec2(28.0, 24.0);

    match active_tab {
        // ═══════════════════════════════════════════════════════════════════════
        // CARET TAB
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::Carets => {
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
                "Choose from 13 animated styles & particle effects",
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
            let desc_y = chip_start_y + 4.0 * (row_h + row_gap) + 6.0;
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

        // ═══════════════════════════════════════════════════════════════════════
        // EDITOR MODE TAB (Hybrid vs Vim)
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::EditorMode => {
            painter.text(
                p_origin,
                Align2::LEFT_TOP,
                "TYPING & EDITING ENGINE",
                FontId::monospace(14.5),
                theme.highlight,
            );
            painter.text(
                p_origin + vec2(0.0, 22.0),
                Align2::LEFT_TOP,
                "Select your preferred editing cockpit (or toggle anytime via :vim)",
                FontId::monospace(11.0),
                theme.muted,
            );

            let card_w = panel_rect.width() - 56.0;
            let mut cur_y = p_origin.y + 52.0;

            // ── Card 1: Hybrid Mode ──────────────────────────────────────────
            let is_hybrid = *editor_input_mode == EditorInputMode::Hybrid;
            let hybrid_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 128.0));
            let hybrid_hover = ui.rect_contains_pointer(hybrid_card);

            if hybrid_hover && ui.input(|i| i.pointer.primary_clicked()) {
                *editor_input_mode = EditorInputMode::Hybrid;
                on_save_setting("editor_mode", "hybrid");
            }

            let hybrid_bg = if is_hybrid {
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 30)
            } else if hybrid_hover {
                Color32::from_rgb(20, 22, 28)
            } else {
                Color32::from_rgb(14, 15, 20)
            };
            painter.rect(
                hybrid_card,
                5.0,
                hybrid_bg,
                Stroke::new(1.0, if is_hybrid { theme.accent } else { Color32::from_rgb(32, 34, 44) }),
                egui::StrokeKind::Inside,
            );

            painter.text(
                hybrid_card.min + vec2(14.0, 11.0),
                Align2::LEFT_TOP,
                "⚡ Hybrid Mode (Modern Power IDE)",
                FontId::monospace(12.5),
                if is_hybrid { theme.accent } else { theme.highlight },
            );
            if is_hybrid {
                painter.text(
                    pos2(hybrid_card.max.x - 14.0, hybrid_card.min.y + 11.0),
                    Align2::RIGHT_TOP,
                    "● ACTIVE",
                    FontId::monospace(10.5),
                    theme.accent,
                );
            }

            let hybrid_features = [
                ("• Word Jump", "Ctrl+Left/Right to jump; Shift to select"),
                ("• Line Move", "Alt+Up/Down swaps lines in place smoothly"),
                ("• Duplicate", "Ctrl+D duplicates current line or selection"),
                ("• Auto-Pair", "Closes \"\", (), [], {} and wraps selected text"),
                ("• Smart Tab", "Tab indents 4 spaces; Shift+Tab dedents"),
            ];
            for (idx, (label, desc)) in hybrid_features.iter().enumerate() {
                let y = hybrid_card.min.y + 34.0 + idx as f32 * 17.5;
                painter.text(pos2(hybrid_card.min.x + 14.0, y), Align2::LEFT_TOP, *label, FontId::monospace(10.0), theme.accent);
                painter.text(pos2(hybrid_card.min.x + 110.0, y), Align2::LEFT_TOP, *desc, FontId::monospace(9.5), Color32::from_gray(175));
            }

            cur_y += 138.0;

            // ── Card 2: Vim Mode ─────────────────────────────────────────────
            let is_vim = *editor_input_mode == EditorInputMode::Vim;
            let vim_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 128.0));
            let vim_hover = ui.rect_contains_pointer(vim_card);

            if vim_hover && ui.input(|i| i.pointer.primary_clicked()) {
                *editor_input_mode = EditorInputMode::Vim;
                on_save_setting("editor_mode", "vim");
            }

            let vim_bg = if is_vim {
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 30)
            } else if vim_hover {
                Color32::from_rgb(20, 22, 28)
            } else {
                Color32::from_rgb(14, 15, 20)
            };
            painter.rect(
                vim_card,
                5.0,
                vim_bg,
                Stroke::new(1.0, if is_vim { theme.accent } else { Color32::from_rgb(32, 34, 44) }),
                egui::StrokeKind::Inside,
            );

            painter.text(
                vim_card.min + vec2(14.0, 11.0),
                Align2::LEFT_TOP,
                "⚔ Vim Mode (Modal Keyboard Engine)",
                FontId::monospace(12.5),
                if is_vim { theme.accent } else { theme.highlight },
            );
            if is_vim {
                painter.text(
                    pos2(vim_card.max.x - 14.0, vim_card.min.y + 11.0),
                    Align2::RIGHT_TOP,
                    "● ACTIVE",
                    FontId::monospace(10.5),
                    theme.accent,
                );
            }

            let vim_features = [
                ("• Motions", "h, j, k, l, w, b, 0, $, gg, G & counts (3j, 5w)"),
                ("• Operators", "dd, yy, cc, dw, x, u, Ctrl+R & text objects"),
                ("• Search", "/ and ? in-buffer search with live n/N repeat"),
                ("• Modes", "Normal, Insert (i, a, o, A, I), Visual (v, V)"),
                ("• Caret", "Dynamic Block in Normal and Beam in Insert"),
            ];
            for (idx, (label, desc)) in vim_features.iter().enumerate() {
                let y = vim_card.min.y + 34.0 + idx as f32 * 17.5;
                painter.text(pos2(vim_card.min.x + 14.0, y), Align2::LEFT_TOP, *label, FontId::monospace(10.0), theme.accent);
                painter.text(pos2(vim_card.min.x + 110.0, y), Align2::LEFT_TOP, *desc, FontId::monospace(9.5), Color32::from_gray(175));
            }
        }


        // ═══════════════════════════════════════════════════════════════════════
        // SOUNDS TAB
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::Sounds => {
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

        // ═══════════════════════════════════════════════════════════════════════
        // THEME TAB
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::Theme => {
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

        // ═══════════════════════════════════════════════════════════════════════
        // SHORTCUTS TAB
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::Shortcuts => {
            painter.text(
                p_origin,
                Align2::LEFT_TOP,
                "KEYBOARD SHORTCUTS",
                FontId::monospace(14.5),
                theme.highlight,
            );
            painter.text(
                p_origin + vec2(0.0, 22.0),
                Align2::LEFT_TOP,
                "Keyboard-driven navigation reference",
                FontId::monospace(11.5),
                theme.muted,
            );

            // Groups: (header_label, &[(key, desc)])
            let groups: &[(&str, &[(&str, &str)])] = &[
                ("DOCUMENT", &[
                    ("Ctrl + N",      "Create new note"),
                    ("Ctrl + S",      "Save active document"),
                    ("Ctrl + R",      "Rename document"),
                    ("Ctrl + D",      "Delete active document"),
                    ("Ctrl + Enter",  "Insert line below"),
                    ("Ctrl + [ / ]",  "Move text left / right (dedent/indent)"),
                ]),
                ("NAVIGATION", &[
                    ("Ctrl + P",   "Fuzzy search"),
                    ("Ctrl + B",   "Toggle sidebar"),
                    ("Ctrl + ,",   "Open preferences"),
                    ("Esc",        "Dismiss modal / return"),
                ]),
                ("SYSTEM", &[
                    ("Ctrl+Shift+W", "Close window"),
                    (":help",        "Command bar reference"),
                ]),
            ];

            let row_w = panel_rect.width() - 56.0;
            let row_h = 24.0;
            let badge_w = 108.0;
            let group_gap = 8.0;
            let header_h = 16.0;
            let item_gap = 3.0;

            let mut cur_y = p_origin.y + 48.0;

            for (g_label, items) in groups.iter() {
                // Group header line
                painter.line_segment(
                    [pos2(p_origin.x, cur_y + header_h * 0.5), pos2(p_origin.x + 28.0, cur_y + header_h * 0.5)],
                    Stroke::new(1.0, Color32::from_gray(40)),
                );
                painter.text(
                    pos2(p_origin.x + 34.0, cur_y + header_h * 0.5),
                    Align2::LEFT_CENTER,
                    *g_label,
                    FontId::monospace(10.0),
                    Color32::from_gray(90),
                );
                let label_end_x = p_origin.x + 34.0 + g_label.len() as f32 * 6.2 + 8.0;
                painter.line_segment(
                    [pos2(label_end_x, cur_y + header_h * 0.5), pos2(p_origin.x + row_w, cur_y + header_h * 0.5)],
                    Stroke::new(1.0, Color32::from_gray(40)),
                );
                cur_y += header_h + 4.0;

                for (key, desc) in items.iter() {
                    let row_rect = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(row_w, row_h));
                    let hovered = ui.rect_contains_pointer(row_rect);

                    // Row background
                    painter.rect(
                        row_rect,
                        4.0,
                        if hovered { Color32::from_rgb(20, 22, 30) } else { Color32::from_rgb(13, 14, 18) },
                        Stroke::new(1.0, if hovered { Color32::from_rgb(40, 44, 56) } else { Color32::from_rgb(22, 24, 30) }),
                        egui::StrokeKind::Inside,
                    );

                    // Keycap badge — 3D-ish: face + shadow strip at bottom
                    let badge_rect = Rect::from_min_size(
                        pos2(row_rect.min.x + 5.0, row_rect.min.y + 3.0),
                        vec2(badge_w, row_h - 6.0),
                    );
                    // Shadow strip (bottom 2px darker)
                    let shadow_strip = Rect::from_min_size(
                        pos2(badge_rect.min.x, badge_rect.max.y - 3.0),
                        vec2(badge_rect.width(), 3.0),
                    );
                    painter.rect_filled(
                        badge_rect,
                        3.0,
                        Color32::from_rgb(26, 28, 38),
                    );
                    painter.rect_filled(
                        shadow_strip,
                        egui::CornerRadius { nw: 0, ne: 0, sw: 3, se: 3 },
                        Color32::from_rgb(14, 15, 22),
                    );
                    painter.rect(
                        badge_rect,
                        3.0,
                        Color32::TRANSPARENT,
                        Stroke::new(1.0, Color32::from_rgb(48, 52, 68)),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        badge_rect.center() - vec2(0.0, 1.0),
                        Align2::CENTER_CENTER,
                        *key,
                        FontId::monospace(10.5),
                        theme.accent,
                    );

                    // Description
                    painter.text(
                        pos2(row_rect.min.x + badge_w + 14.0, row_rect.center().y),
                        Align2::LEFT_CENTER,
                        *desc,
                        FontId::monospace(11.5),
                        if hovered { Color32::WHITE } else { Color32::from_gray(180) },
                    );

                    cur_y += row_h + item_gap;
                }

                cur_y += group_gap;
            }
        }
        SettingTab::Backup => {
            // Header
            painter.text(
                p_origin,
                Align2::LEFT_TOP,
                "Auto-Backup & Snapshots",
                FontId::monospace(15.0),
                theme.highlight,
            );
            painter.text(
                p_origin + vec2(0.0, 22.0),
                Align2::LEFT_TOP,
                "Atomic SQLite database snapshots with WAL write-protection",
                FontId::monospace(11.0),
                theme.muted,
            );

            let mut cur_y = p_origin.y + 60.0;
            let card_w = panel_rect.width() - 56.0;

            // Card 1: Backup Directory Path
            let path_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 90.0));
            painter.rect(
                path_card,
                5.0,
                Color32::from_rgb(18, 20, 26),
                Stroke::new(1.0, Color32::from_rgb(34, 37, 48)),
                egui::StrokeKind::Inside,
            );

            painter.text(
                path_card.min + vec2(14.0, 12.0),
                Align2::LEFT_TOP,
                "Backup Destination Directory",
                FontId::monospace(12.5),
                Color32::from_gray(220),
            );

            // Display current path in a pill box
            let path_display = if backup_dir.is_empty() {
                "Default: mindforge_backup/".to_string()
            } else if backup_dir.len() > 42 {
                format!("...{}", &backup_dir[backup_dir.len() - 39..])
            } else {
                backup_dir.clone()
            };

            let pill_rect = Rect::from_min_size(
                path_card.min + vec2(14.0, 42.0),
                vec2(card_w - 150.0, 32.0),
            );
            painter.rect(
                pill_rect,
                4.0,
                Color32::from_rgb(12, 13, 17),
                Stroke::new(1.0, Color32::from_rgb(28, 30, 40)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                pill_rect.min + vec2(10.0, 8.0),
                Align2::LEFT_TOP,
                path_display,
                FontId::monospace(11.0),
                theme.accent,
            );

            // Browse button
            let browse_rect = Rect::from_min_size(
                path_card.min + vec2(card_w - 126.0, 42.0),
                vec2(112.0, 32.0),
            );
            let browse_hover = ui.rect_contains_pointer(browse_rect);
            let browse_bg = if browse_hover {
                Color32::from_rgb(36, 40, 54)
            } else {
                Color32::from_rgb(26, 28, 38)
            };
            painter.rect(
                browse_rect,
                4.0,
                browse_bg,
                Stroke::new(1.0, if browse_hover { theme.accent } else { Color32::from_rgb(48, 52, 68) }),
                egui::StrokeKind::Inside,
            );
            painter.text(
                browse_rect.center(),
                Align2::CENTER_CENTER,
                "📁 Browse...",
                FontId::monospace(11.5),
                if browse_hover { Color32::WHITE } else { Color32::from_gray(210) },
            );

            if browse_hover && ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    let p = folder.to_string_lossy().to_string();
                    *backup_dir = p.clone();
                    on_save_setting("backup_dir", &p);
                }
            }

            cur_y += 105.0;

            // Card 2: Instant Snapshot Action (:backup)
            let action_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 82.0));
            painter.rect(
                action_card,
                5.0,
                Color32::from_rgb(18, 20, 26),
                Stroke::new(1.0, Color32::from_rgb(34, 37, 48)),
                egui::StrokeKind::Inside,
            );

            painter.text(
                action_card.min + vec2(14.0, 12.0),
                Align2::LEFT_TOP,
                "1-Click Snapshot",
                FontId::monospace(12.5),
                Color32::from_gray(220),
            );

            let status_str = match last_backup_status {
                Some(s) if s.len() > 32 => format!("...{}", &s[s.len() - 29..]),
                Some(s) => s.to_string(),
                None => "Ready — or type :backup anytime".to_string(),
            };
            painter.text(
                action_card.min + vec2(14.0, 38.0),
                Align2::LEFT_TOP,
                status_str,
                FontId::monospace(10.5),
                theme.muted,
            );

            let run_btn_rect = Rect::from_min_size(
                action_card.min + vec2(card_w - 180.0, 24.0),
                vec2(166.0, 34.0),
            );
            let run_hover = ui.rect_contains_pointer(run_btn_rect);
            let run_bg = if run_hover {
                Color32::from_rgb(28, 48, 38)
            } else {
                Color32::from_rgb(20, 32, 26)
            };
            painter.rect(
                run_btn_rect,
                4.0,
                run_bg,
                Stroke::new(1.0, if run_hover { theme.accent } else { Color32::from_rgb(36, 68, 50) }),
                egui::StrokeKind::Inside,
            );
            painter.text(
                run_btn_rect.center(),
                Align2::CENTER_CENTER,
                "⚡ Snapshot (:backup)",
                FontId::monospace(11.5),
                theme.accent,
            );

            if run_hover && ui.input(|i| i.pointer.primary_clicked()) {
                action = Some(SettingPanelAction::TriggerBackup);
            }

            cur_y += 98.0;

            // Card 3: Safety info - clean multi-line layout with zero horizontal overflow
            let info_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 132.0));
            painter.rect(
                info_card,
                5.0,
                Color32::from_rgb(14, 15, 20),
                Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
                egui::StrokeKind::Inside,
            );

            let features = [
                ("• WAL Architecture", "Guarantees high write speed and transactional safety."),
                ("• Automated Snapshots", "Timestamped as mindforge_backup_YYYYMMDD_HHMMSS.db"),
                ("• Zero Lock Contention", "Atomic snapshots execute without freezing the editor."),
            ];
            for (idx, (label, desc)) in features.iter().enumerate() {
                let y = info_card.min.y + 12.0 + idx as f32 * 38.0;
                painter.text(
                    pos2(info_card.min.x + 14.0, y),
                    Align2::LEFT_TOP,
                    *label,
                    FontId::monospace(11.0),
                    theme.accent,
                );
                painter.text(
                    pos2(info_card.min.x + 14.0, y + 16.0),
                    Align2::LEFT_TOP,
                    *desc,
                    FontId::monospace(10.0),
                    Color32::from_gray(160),
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // UPDATES TAB
        // ═══════════════════════════════════════════════════════════════════════
        SettingTab::Updates => {
            let status = updater.status();
            let card_w = panel_rect.width() - 56.0;

            // ── Header ───────────────────────────────────────────────────────
            painter.text(
                p_origin,
                Align2::LEFT_TOP,
                "SOFTWARE UPDATES",
                FontId::monospace(14.5),
                theme.highlight,
            );
            painter.text(
                p_origin + vec2(0.0, 22.0),
                Align2::LEFT_TOP,
                "Manage updates for MindForge",
                FontId::monospace(11.5),
                theme.muted,
            );

            // Version info card
            let ver_card = Rect::from_min_size(pos2(p_origin.x, p_origin.y + 54.0), vec2(card_w, 64.0));
            painter.rect(
                ver_card,
                5.0,
                Color32::from_rgb(14, 15, 20),
                Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                ver_card.min + vec2(16.0, 12.0),
                Align2::LEFT_TOP,
                "Current Version",
                FontId::monospace(10.5),
                theme.muted,
            );
            painter.text(
                ver_card.min + vec2(16.0, 30.0),
                Align2::LEFT_TOP,
                &format!("v{}", env!("CARGO_PKG_VERSION")),
                FontId::monospace(13.5),
                theme.accent,
            );

            // Status card
            let status_card = Rect::from_min_size(pos2(p_origin.x, p_origin.y + 132.0), vec2(card_w, 120.0));
            painter.rect(
                status_card,
                5.0,
                Color32::from_rgb(14, 15, 20),
                Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
                egui::StrokeKind::Inside,
            );

            match &status {
                UpdateStatus::Idle => {
                    painter.text(
                        status_card.min + vec2(16.0, 20.0),
                        Align2::LEFT_TOP,
                        "⟳  Ready to check for updates",
                        FontId::monospace(12.0),
                        Color32::from_gray(160),
                    );
                }
                UpdateStatus::Checking => {
                    painter.text(
                        status_card.min + vec2(16.0, 20.0),
                        Align2::LEFT_TOP,
                        "⟳  Checking GitHub for latest release...",
                        FontId::monospace(12.0),
                        theme.accent,
                    );
                }
                UpdateStatus::UpToDate { version } => {
                    painter.text(
                        status_card.min + vec2(16.0, 20.0),
                        Align2::LEFT_TOP,
                        &format!("✓  You are on the latest version (v{})", version),
                        FontId::monospace(12.0),
                        Color32::from_rgb(80, 200, 120),
                    );
                }
                UpdateStatus::UpdateAvailable { new_version, release_notes, .. } => {
                    painter.text(
                        status_card.min + vec2(16.0, 12.0),
                        Align2::LEFT_TOP,
                        &format!("🔔  Update available: v{}", new_version),
                        FontId::monospace(12.5),
                        Color32::from_rgb(255, 200, 60),
                    );
                    // Truncate release notes to 2 lines
                    let notes: String = release_notes.lines().take(2).collect::<Vec<_>>().join(" • ");
                    let notes_short = if notes.len() > 80 { format!("{}…", &notes[..80]) } else { notes };
                    painter.text(
                        status_card.min + vec2(16.0, 36.0),
                        Align2::LEFT_TOP,
                        &notes_short,
                        FontId::monospace(10.5),
                        Color32::from_gray(150),
                    );
                }
                UpdateStatus::Downloading { new_version, progress, downloaded_bytes, total_bytes } => {
                    painter.text(
                        status_card.min + vec2(16.0, 12.0),
                        Align2::LEFT_TOP,
                        &format!("⬇  Downloading v{}…", new_version),
                        FontId::monospace(12.0),
                        theme.accent,
                    );
                    // Progress bar
                    let bar_bg = Rect::from_min_size(
                        status_card.min + vec2(16.0, 40.0),
                        vec2(card_w - 32.0, 10.0),
                    );
                    painter.rect_filled(bar_bg, 5.0, Color32::from_rgb(25, 28, 36));
                    if *progress > 0.0 {
                        let fill_w = (bar_bg.width() * progress).max(10.0);
                        let bar_fill = Rect::from_min_size(bar_bg.min, vec2(fill_w, bar_bg.height()));
                        painter.rect_filled(bar_fill, 5.0, theme.accent);
                    }
                    let mb_done = *downloaded_bytes as f64 / 1_048_576.0;
                    let mb_total = *total_bytes as f64 / 1_048_576.0;
                    painter.text(
                        status_card.min + vec2(16.0, 60.0),
                        Align2::LEFT_TOP,
                        &format!("{:.1} MB / {:.1} MB  ({:.0}%)", mb_done, mb_total, progress * 100.0),
                        FontId::monospace(10.5),
                        theme.muted,
                    );
                }
                UpdateStatus::ReadyToRestart { new_version, .. } => {
                    painter.text(
                        status_card.min + vec2(16.0, 20.0),
                        Align2::LEFT_TOP,
                        &format!("✓  v{} downloaded — restart to apply", new_version),
                        FontId::monospace(12.0),
                        Color32::from_rgb(80, 200, 120),
                    );
                }
                UpdateStatus::Error(msg) => {
                    let short = if msg.len() > 90 { format!("{}…", &msg[..90]) } else { msg.clone() };
                    painter.text(
                        status_card.min + vec2(16.0, 16.0),
                        Align2::LEFT_TOP,
                        &format!("✗  {}", short),
                        FontId::monospace(11.0),
                        Color32::from_rgb(220, 70, 70),
                    );
                }
            }

            // ── Action button — morphs per state ─────────────────────────────
            let (btn_label, btn_color, btn_border, can_click) = match &status {
                UpdateStatus::Idle | UpdateStatus::UpToDate { .. } | UpdateStatus::Error(_) => (
                    "⟳  Check for Updates",
                    Color32::from_rgb(18, 32, 50),
                    Color32::from_rgb(36, 68, 100),
                    true,
                ),
                UpdateStatus::Checking => (
                    "⟳  Checking…",
                    Color32::from_rgb(14, 20, 36),
                    Color32::from_rgb(30, 50, 80),
                    false,
                ),
                UpdateStatus::UpdateAvailable { .. } => (
                    "⬇  Download Update",
                    Color32::from_rgb(40, 50, 14),
                    Color32::from_rgb(100, 130, 30),
                    true,
                ),
                UpdateStatus::Downloading { .. } => (
                    "⬇  Downloading…",
                    Color32::from_rgb(14, 20, 36),
                    Color32::from_rgb(30, 50, 80),
                    false,
                ),
                UpdateStatus::ReadyToRestart { .. } => (
                    "↺  Restart to Apply Update",
                    Color32::from_rgb(14, 36, 20),
                    Color32::from_rgb(36, 120, 60),
                    true,
                ),
            };

            let btn_rect = Rect::from_min_size(
                pos2(p_origin.x, p_origin.y + 270.0),
                vec2(card_w, 42.0),
            );
            let btn_hov = can_click && ui.rect_contains_pointer(btn_rect);
            let bg = if btn_hov { Color32::from_rgb(btn_color.r().saturating_add(12), btn_color.g().saturating_add(12), btn_color.b().saturating_add(12)) } else { btn_color };
            painter.rect(
                btn_rect,
                6.0,
                bg,
                Stroke::new(1.0, btn_border),
                egui::StrokeKind::Inside,
            );
            painter.text(
                btn_rect.center(),
                Align2::CENTER_CENTER,
                btn_label,
                FontId::monospace(12.5),
                if can_click { theme.accent } else { theme.muted },
            );

            if btn_hov && ui.input(|i| i.pointer.primary_clicked()) {
                let act = match &status {
                    UpdateStatus::Idle | UpdateStatus::UpToDate { .. } | UpdateStatus::Error(_) => {
                        Some(SettingPanelAction::CheckUpdates)
                    }
                    UpdateStatus::UpdateAvailable { .. } => {
                        Some(SettingPanelAction::DownloadUpdate)
                    }
                    UpdateStatus::ReadyToRestart { .. } => {
                        Some(SettingPanelAction::RestartToApply)
                    }
                    _ => None,
                };
                if let Some(a) = act {
                    action = Some(a);
                }
            }
        }
    }

    action
}
