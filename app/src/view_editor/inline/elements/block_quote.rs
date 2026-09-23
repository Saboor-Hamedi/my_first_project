//! Blockquote / command input styling and card wrapper rendering.
//!
//! Complies with the Lumina Premium Inline standard:
//! 1. Borderless definition through background luminance shift (+5% to +10%), zero hard borders.
//! 2. Baseline alignment with surrounding editor content.
//! 3. Compact geometry matching font_line_height + comfortable vertical padding.
//! 4. Thematic integration with theme accent prompt and text color.

use crate::theme::Theme;
use eframe::egui::{pos2, Color32, Painter, Rect, Stroke};

/// Renders the seamless, borderless background wrapper for a blockquote line.
pub fn render_block_quote_wrapper(
    painter: &Painter,
    line_y: f32,
    line_h: f32,
    text_left: f32,
    content_right: f32,
    theme: &Theme,
    is_active: bool,
) {
    // Compact geometry: font_line_height + 4px vertical padding (2px top / 2px bottom)
    // Horizontal padding aligns with editor margins
    let bg_rect = Rect::from_min_max(
        pos2(text_left - 6.0, line_y - 2.0),
        pos2(content_right, line_y + line_h + 2.0),
    );

    // Background color differentiation (+5% to +10% luminance shift from canvas)
    let base = theme.bg;
    let bg_color = if theme.is_light() {
        Color32::from_rgb(
            base.r().saturating_sub(12),
            base.g().saturating_sub(12),
            base.b().saturating_sub(12),
        )
    } else {
        Color32::from_rgb(
            base.r().saturating_add(14),
            base.g().saturating_add(14),
            base.b().saturating_add(16),
        )
    };

    // Borderless definition: no visible stroke/border.
    // Subtle accent focus indicator (<10% opacity) only when actively editing.
    let stroke = if is_active {
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(
                theme.accent.r(),
                theme.accent.g(),
                theme.accent.b(),
                22, // ~8.6% opacity, strictly < 10%
            ),
        )
    } else {
        Stroke::NONE
    };

    painter.rect(
        bg_rect,
        4.0,
        bg_color,
        stroke,
        eframe::egui::StrokeKind::Inside,
    );
}
