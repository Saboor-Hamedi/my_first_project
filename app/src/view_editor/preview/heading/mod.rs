//! Preview heading rendering module providing 1:1 visual parity with Editor headings.
//!
//! Reuses the exact typography hierarchy, font sizes, colors, line heights,
//! and vertical spacing metrics established by the inline editor.

use crate::theme::Theme;
use crate::view_editor::inline::elements::{heading_color, heading_metrics};
use eframe::egui::{pos2, Painter, Rect};

/// Renders a markdown heading (levels 1..=6) with exact 1:1 parity to the editor's heading metrics.
/// Matches font size, typography baseline, line height, text color, and vertical margins.
///
/// Returns the vertical height consumed by the rendered heading block (including margins).
pub fn render_preview_heading(
    painter: &Painter,
    content_painter: &Painter,
    start_x: f32,
    current_y: f32,
    max_text_w: f32,
    font_size: f32,
    level: u8,
    text: &str,
    theme: &Theme,
    b_idx: usize,
    viewport_rect: Rect,
) -> f32 {
    let (font, line_h) = heading_metrics(level, font_size);
    let color = heading_color(level, theme);

    // Vertical top margin before heading (suppressed for the very first block)
    let margin_top = if b_idx == 0 {
        0.0
    } else {
        match level {
            1 => 12.0,
            2 => 10.0,
            3 => 8.0,
            _ => 6.0,
        }
    };

    let galley = painter.layout(text.to_string(), font, color, max_text_w);
    let block_y = current_y + margin_top;
    let actual_h = line_h.max(galley.size().y);

    // Viewport frustum culling
    if block_y + actual_h >= viewport_rect.min.y && block_y <= viewport_rect.max.y {
        // Vertically center galley in heading line slot matching editor's baseline
        let y_pad = ((actual_h - galley.size().y) * 0.5).round().max(0.0);
        content_painter.galley(pos2(start_x, block_y + y_pad), galley, color);
    }

    let margin_bottom = match level {
        1 => 6.0,
        2 => 5.0,
        3 => 4.0,
        _ => 3.0,
    };

    margin_top + actual_h + margin_bottom
}
