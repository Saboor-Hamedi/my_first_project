//! Blockquote styling and card wrapper rendering.
//!
//! Renders a minimal, elegant callout wrapper:
//! 1. A soft background surface slightly elevated from the editor canvas.
//! 2. A sleek 3px vertical accent pill on the left edge with rounded caps.
//! 3. Compact geometry — card occupies exactly the line slot, no extra padding.
//! 4. Seamlessly integrates with active theme and user accent color.

use crate::theme::Theme;
use eframe::egui::{pos2, Color32, Painter, Rect};

/// Renders the blockquote card wrapper (background + left accent bar) for one inline line.
pub fn render_block_quote_wrapper(
    painter: &Painter,
    line_y: f32,
    line_h: f32,
    text_left: f32,
    content_right: f32,
    theme: &Theme,
    is_active: bool,
) {
    // Card spans from a small offset left of text to right content edge.
    // The accent bar sits at the very left of this card area.
    let bar_w = 3.0;
    let bar_gap = 2.0;   // gap between accent bar and card start
    let card_left = text_left - (bar_w + bar_gap * 2.0 + 8.0);

    // Card background — subtle luminance step from the editor bg.
    // Blend toward a neutral tone rather than pure add/sub to avoid color shift.
    let bg = theme.bg;
    let step: i16 = if theme.is_light() { -9 } else { 14 };
    let bg_color = Color32::from_rgb(
        (bg.r() as i16 + step).clamp(0, 255) as u8,
        (bg.g() as i16 + step).clamp(0, 255) as u8,
        (bg.b() as i16 + step).clamp(0, 255) as u8,
    );

    let card_rect = Rect::from_min_max(
        pos2(card_left, line_y),
        pos2(content_right, line_y + line_h),
    );
    painter.rect_filled(card_rect, 5.0, bg_color);

    // Accent bar — flush with card left edge, inset 2px vertically for rounded ends.
    let bar_inset_y = 3.0;
    let bar_rect = Rect::from_min_max(
        pos2(card_left + bar_gap, line_y + bar_inset_y),
        pos2(card_left + bar_gap + bar_w, line_y + line_h - bar_inset_y),
    );

    // Full accent when caret is on this line, soft 65% opacity when resting.
    let accent_alpha: u8 = if is_active { 255 } else { 165 };
    let bar_color = Color32::from_rgba_unmultiplied(
        theme.accent.r(),
        theme.accent.g(),
        theme.accent.b(),
        accent_alpha,
    );
    painter.rect_filled(bar_rect, 1.5, bar_color);
}
