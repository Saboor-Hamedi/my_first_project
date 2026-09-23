//! Code block and code fence card container wrapper rendering.
//!
//! Provides a unified, elevated surface card with rounded corners,
//! language badge pills, and syntax-highlighted containers.

use crate::theme::Theme;
use eframe::egui::{pos2, Align2, Color32, FontId, Painter, Rect, Stroke};

/// Renders the code line surface wrapper.
pub fn render_code_wrapper_line(
    painter: &Painter,
    line_y: f32,
    line_h: f32,
    text_left: f32,
    content_right: f32,
    theme: &Theme,
    is_fence: bool,
    fence_lang: Option<&str>,
    is_active: bool,
) {
    let bg_rect = Rect::from_min_max(
        pos2(text_left - 8.0, line_y),
        pos2(content_right - 8.0, line_y + line_h),
    );

    // Code surface container with subtle border
    painter.rect(
        bg_rect,
        3.0,
        theme.surface(),
        Stroke::new(1.0, theme.border()),
        eframe::egui::StrokeKind::Inside,
    );

    // For inactive code fences (e.g. ```rust), render a sleek pill badge with language name
    if is_fence && !is_active {
        if let Some(lang) = fence_lang {
            if !lang.is_empty() {
                let badge_font = FontId::monospace(10.0);
                let badge_text = lang.to_uppercase();
                let badge_pos = pos2(content_right - 20.0, line_y + line_h * 0.5);

                let badge_bg = Color32::from_rgba_unmultiplied(
                    theme.accent.r(),
                    theme.accent.g(),
                    theme.accent.b(),
                    35,
                );
                let badge_w = (badge_text.len() as f32 * 6.5 + 10.0).max(24.0);
                let badge_rect = Rect::from_center_size(
                    badge_pos,
                    eframe::egui::vec2(badge_w, 16.0),
                );

                painter.rect_filled(badge_rect, 4.0, badge_bg);
                painter.text(
                    badge_pos,
                    Align2::CENTER_CENTER,
                    badge_text,
                    badge_font,
                    theme.accent,
                );
            }
        }
    }
}
