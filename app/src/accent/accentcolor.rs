//! Accent color customization, palette overrides, and titlebar dropdown.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

/// Persistent custom color overrides for theme accent, text, and highlight/selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccentOverrides {
    pub accent: Option<Color32>,
    pub text: Option<Color32>,
    pub highlight: Option<Color32>,
}

impl AccentOverrides {
    /// Applies active overrides to the provided theme instance.
    pub fn apply(&self, theme: &mut Theme) {
        if let Some(c) = self.accent {
            theme.accent = c;
        }
        if let Some(c) = self.text {
            theme.text = c;
        }
        if let Some(c) = self.highlight {
            theme.highlight = c;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.accent.is_none() && self.text.is_none() && self.highlight.is_none()
    }

    pub fn clear(&mut self) {
        self.accent = None;
        self.text = None;
        self.highlight = None;
    }

    /// Loads overrides stored in SQLite settings table.
    pub fn load_from_db(db: &core::Database) -> Self {
        let accent = db
            .get_setting("override_accent")
            .ok()
            .flatten()
            .and_then(|s| color_from_hex(&s));
        let text = db
            .get_setting("override_text")
            .ok()
            .flatten()
            .and_then(|s| color_from_hex(&s));
        let highlight = db
            .get_setting("override_highlight")
            .ok()
            .flatten()
            .and_then(|s| color_from_hex(&s));

        Self {
            accent,
            text,
            highlight,
        }
    }

    /// Persists active overrides into SQLite settings table.
    pub fn save_to_db(&self, db: &core::Database) {
        if let Some(c) = self.accent {
            let _ = db.set_setting("override_accent", &hex_from_color(c));
        } else {
            let _ = db.set_setting("override_accent", "");
        }

        if let Some(c) = self.text {
            let _ = db.set_setting("override_text", &hex_from_color(c));
        } else {
            let _ = db.set_setting("override_text", "");
        }

        if let Some(c) = self.highlight {
            let _ = db.set_setting("override_highlight", &hex_from_color(c));
        } else {
            let _ = db.set_setting("override_highlight", "");
        }
    }
}

pub enum AccentAction {
    Changed,
    ResetAll,
    Close,
}

/// Curated modern palette swatches for instant 1-click customization.
const PALETTE_SWATCHES: &[Color32] = &[
    Color32::from_rgb(42, 161, 152),  // Solarized Cyan (#2aa198)
    Color32::from_rgb(16, 185, 129),  // Emerald Green (#10b981)
    Color32::from_rgb(245, 158, 11),  // Warm Amber (#f59e0b)
    Color32::from_rgb(249, 115, 22),  // Sunset Coral (#f97316)
    Color32::from_rgb(244, 63, 94),   // Crimson Rose (#f43f5e)
    Color32::from_rgb(168, 85, 247),  // Neon Violet (#a855f7)
    Color32::from_rgb(56, 189, 248),  // Sky Blue (#38bdf8)
    Color32::from_rgb(59, 130, 246),  // Royal Blue (#3b82f6)
    Color32::from_rgb(255, 255, 255), // Pure White (#ffffff)
    Color32::from_rgb(203, 213, 225), // Soft Silver (#cbd5e1)
];

/// Formats a Color32 into an uppercase hex color string `#RRGGBB`.
pub fn hex_from_color(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// Parses a `#rrggbb` or `rrggbb` hex string into Color32.
pub fn color_from_hex(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

/// Renders the polished Accent Color customizer dropdown beneath the titlebar palette button.
pub fn render_accent_dropdown(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    anchor_rect: Rect,
    overrides: &mut AccentOverrides,
    theme: &Theme,
    default_theme: &Theme,
) -> Option<AccentAction> {
    let mut action = None;

    // Position dropdown floating right-aligned under the anchor button
    let dropdown_w = 320.0;
    let dropdown_h = 385.0;
    let min_x = (anchor_rect.max.x - dropdown_w).max(8.0);
    let min_y = anchor_rect.max.y + 4.0;
    let dropdown_rect = Rect::from_min_size(pos2(min_x, min_y), vec2(dropdown_w, dropdown_h));

    // Dismiss if user clicked outside dropdown and anchor
    if ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !dropdown_rect.contains(pos) && !anchor_rect.contains(pos) {
                return Some(AccentAction::Close);
            }
        }
    }

    // Outer drop shadow for depth
    painter.rect(
        dropdown_rect.expand(2.0),
        8.0,
        Color32::from_black_alpha(150),
        Stroke::NONE,
        egui::StrokeKind::Outside,
    );

    // Floating card background with subtle theme surface & border
    painter.rect(
        dropdown_rect,
        6.0,
        theme.surface(),
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    // Header strip
    let header_h = 44.0;
    let header_rect = Rect::from_min_max(
        dropdown_rect.min,
        pos2(dropdown_rect.max.x, dropdown_rect.min.y + header_h),
    );
    painter.rect(
        header_rect,
        egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
        Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), 160),
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0, theme.border()),
    );

    // Header Title & Subtitle
    painter.text(
        pos2(header_rect.min.x + 14.0, header_rect.min.y + 13.0),
        Align2::LEFT_CENTER,
        "🎨 Color Customizer",
        FontId::monospace(13.0),
        theme.highlight,
    );
    painter.text(
        pos2(header_rect.min.x + 14.0, header_rect.min.y + 30.0),
        Align2::LEFT_CENTER,
        "Customize theme accents & palette overrides",
        FontId::monospace(10.0),
        theme.muted,
    );

    // Close button in header
    let close_btn_rect = Rect::from_center_size(
        pos2(header_rect.max.x - 18.0, header_rect.center().y),
        vec2(22.0, 22.0),
    );
    let is_close_hovered = ui.rect_contains_pointer(close_btn_rect);
    if is_close_hovered {
        painter.rect_filled(close_btn_rect, 4.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25));
        if ui.input(|i| i.pointer.primary_clicked()) {
            return Some(AccentAction::Close);
        }
    }
    painter.text(
        close_btn_rect.center(),
        Align2::CENTER_CENTER,
        "✕",
        FontId::monospace(12.0),
        if is_close_hovered { Color32::WHITE } else { theme.muted },
    );

    // Content container inside the card
    let inner_rect = Rect::from_min_max(
        pos2(dropdown_rect.min.x + 10.0, header_rect.max.y + 8.0),
        pos2(dropdown_rect.max.x - 10.0, dropdown_rect.max.y - 44.0),
    );

    let mut changed = false;

    // DRY: Render the 3 color sections in beautiful individual sub-cards
    let mut cur_y = inner_rect.min.y;
    let section_h = 88.0;
    let section_gap = 6.0;

    // 1. Accent Color Section
    let s1_rect = Rect::from_min_size(pos2(inner_rect.min.x, cur_y), vec2(inner_rect.width(), section_h));
    render_color_section(
        ui,
        painter,
        s1_rect,
        "Accent Color",
        &mut overrides.accent,
        default_theme.accent,
        theme,
        &mut changed,
    );
    cur_y += section_h + section_gap;

    // 2. Text Color Section
    let s2_rect = Rect::from_min_size(pos2(inner_rect.min.x, cur_y), vec2(inner_rect.width(), section_h));
    render_color_section(
        ui,
        painter,
        s2_rect,
        "Text Color",
        &mut overrides.text,
        default_theme.text,
        theme,
        &mut changed,
    );
    cur_y += section_h + section_gap;

    // 3. Selected / Highlight Color Section
    let s3_rect = Rect::from_min_size(pos2(inner_rect.min.x, cur_y), vec2(inner_rect.width(), section_h));
    render_color_section(
        ui,
        painter,
        s3_rect,
        "Selected / Hover",
        &mut overrides.highlight,
        default_theme.highlight,
        theme,
        &mut changed,
    );

    // Footer: Reset All to Default Button
    let footer_rect = Rect::from_min_max(
        pos2(dropdown_rect.min.x + 10.0, dropdown_rect.max.y - 36.0),
        pos2(dropdown_rect.max.x - 10.0, dropdown_rect.max.y - 8.0),
    );
    let reset_hovered = ui.rect_contains_pointer(footer_rect);
    let reset_bg = if reset_hovered {
        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 45)
    } else {
        Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), 180)
    };
    painter.rect(
        footer_rect,
        5.0,
        reset_bg,
        Stroke::new(1.0, if reset_hovered { theme.accent } else { theme.border() }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        footer_rect.center(),
        Align2::CENTER_CENTER,
        "↺ Reset All to Defaults",
        FontId::monospace(11.5),
        if reset_hovered { theme.accent } else { theme.muted },
    );
    if reset_hovered && ui.input(|i| i.pointer.primary_clicked()) {
        overrides.clear();
        action = Some(AccentAction::ResetAll);
    }

    if changed && action.is_none() {
        action = Some(AccentAction::Changed);
    }

    action
}

/// DRY helper: renders a single polished color category sub-card.
fn render_color_section(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    label: &str,
    current_override: &mut Option<Color32>,
    default_color: Color32,
    theme: &Theme,
    changed: &mut bool,
) {
    let active_color = current_override.unwrap_or(default_color);
    let is_overridden = current_override.is_some();

    // Section sub-card frame with subtle border
    painter.rect(
        rect,
        5.0,
        Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), 130),
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    let inner_pad = 8.0;

    // Top row: Label & current hex pill
    let top_y = rect.min.y + inner_pad + 6.0;
    painter.text(
        pos2(rect.min.x + inner_pad, top_y),
        Align2::LEFT_CENTER,
        label,
        FontId::monospace(11.5),
        theme.text,
    );

    // Active color preview chip & hex badge
    let hex_text = hex_from_color(active_color);
    let chip_w = 14.0;
    let badge_rect = Rect::from_min_max(
        pos2(rect.max.x - inner_pad - 82.0, rect.min.y + inner_pad),
        pos2(rect.max.x - inner_pad, rect.min.y + inner_pad + 18.0),
    );

    painter.rect(
        badge_rect,
        3.0,
        Color32::from_rgba_unmultiplied(theme.surface().r(), theme.surface().g(), theme.surface().b(), 200),
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    let chip_rect = Rect::from_min_size(
        pos2(badge_rect.min.x + 3.0, badge_rect.min.y + 2.0),
        vec2(chip_w, chip_w),
    );
    painter.rect_filled(chip_rect, 2.0, active_color);

    painter.text(
        pos2(chip_rect.max.x + 5.0, badge_rect.center().y),
        Align2::LEFT_CENTER,
        hex_text,
        FontId::monospace(9.5),
        if is_overridden { theme.accent } else { theme.muted },
    );

    // Reset button chip if this property is overridden
    if is_overridden {
        let reset_chip = Rect::from_center_size(
            pos2(badge_rect.min.x - 10.0, badge_rect.center().y),
            vec2(14.0, 14.0),
        );
        let is_reset_hover = ui.rect_contains_pointer(reset_chip);
        if is_reset_hover {
            painter.rect_filled(reset_chip, 3.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25));
            if ui.input(|i| i.pointer.primary_clicked()) {
                *current_override = None;
                *changed = true;
            }
        }
        painter.text(
            reset_chip.center(),
            Align2::CENTER_CENTER,
            "×",
            FontId::monospace(11.0),
            if is_reset_hover { theme.accent } else { theme.muted },
        );
    }

    // Row of clickable palette chips
    let swatch_y = rect.min.y + 32.0;
    let swatch_size = 20.0;
    let swatch_gap = 5.0;

    for (i, &swatch) in PALETTE_SWATCHES.iter().enumerate() {
        if i >= 10 {
            break;
        }
        let sx = rect.min.x + inner_pad + (i as f32 * (swatch_size + swatch_gap));
        let swatch_rect = Rect::from_min_size(pos2(sx, swatch_y), vec2(swatch_size, swatch_size));
        let is_hovered = ui.rect_contains_pointer(swatch_rect);
        let is_selected = active_color == swatch;

        if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            *current_override = Some(swatch);
            *changed = true;
        }

        let stroke = if is_selected {
            Stroke::new(1.8, Color32::WHITE)
        } else if is_hovered {
            Stroke::new(1.3, theme.accent)
        } else {
            Stroke::new(0.8, Color32::from_gray(50))
        };

        painter.rect(
            swatch_rect,
            3.5,
            swatch,
            stroke,
            egui::StrokeKind::Inside,
        );

        if is_selected {
            painter.circle_filled(swatch_rect.center(), 2.2, Color32::WHITE);
        }
    }

    // Custom Color Picker row
    let picker_row_y = swatch_y + swatch_size + 6.0;
    let mut srgba = [
        active_color.r(),
        active_color.g(),
        active_color.b(),
        255,
    ];

    let picker_area = Rect::from_min_size(
        pos2(rect.min.x + inner_pad, picker_row_y),
        vec2(rect.width() - inner_pad * 2.0, 20.0),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(picker_area), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
            let resp = ui.color_edit_button_srgba_unmultiplied(&mut srgba);
            if resp.changed() {
                *current_override = Some(Color32::from_rgb(srgba[0], srgba[1], srgba[2]));
                *changed = true;
            }
            ui.label(
                egui::RichText::new("Custom Color Picker (RGB / Hex)")
                    .size(10.0)
                    .monospace()
                    .color(theme.muted),
            );
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeKind;

    #[test]
    fn test_hex_conversion() {
        let col = Color32::from_rgb(1, 43, 54);
        let hex = hex_from_color(col);
        assert_eq!(hex, "#012B36");
        let parsed = color_from_hex(&hex).unwrap();
        assert_eq!(parsed, col);
    }

    #[test]
    fn test_accent_overrides_apply_and_clear() {
        let mut theme = Theme::from_kind(ThemeKind::Shell);
        let custom_accent = Color32::from_rgb(255, 0, 128);
        let custom_text = Color32::from_rgb(200, 220, 240);

        let mut overrides = AccentOverrides {
            accent: Some(custom_accent),
            text: Some(custom_text),
            highlight: None,
        };

        overrides.apply(&mut theme);
        assert_eq!(theme.accent, custom_accent);
        assert_eq!(theme.text, custom_text);

        overrides.clear();
        assert!(overrides.is_empty());
    }
}
