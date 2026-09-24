//! Preview table rendering module providing 1:1 visual parity with Editor tables.
//!
//! Reuses the exact container stroke, header accent tint, alternating row stripes,
//! hairline divider lines, and cell padding metrics established by the inline editor.

use crate::theme::Theme;
use crate::view_editor::inline::elements::{
    cell_color, render_table_block_decorations, render_table_row_decorations, table_metrics,
};
use crate::view_editor::preview::build_inline_job;
use eframe::egui::{pos2, vec2, Painter, Rect};

/// Renders a markdown table with exact 1:1 visual parity to the editor's table layout.
/// Matches container borders, header pill background, alternating row tints,
/// and exact font & padding metrics.
///
/// Returns the vertical height consumed by the rendered table.
pub fn render_preview_table(
    painter: &Painter,
    content_painter: &Painter,
    start_x: f32,
    current_y: f32,
    max_text_w: f32,
    font_size: f32,
    headers: &[String],
    rows: &[Vec<String>],
    theme: &Theme,
    viewport_rect: Rect,
) -> f32 {
    let col_count = headers.len().max(1);
    let table_margin_right = 16.0;
    let min_table_w = (col_count as f32 * font_size * 5.0).max(300.0);
    let table_w = (max_text_w - table_margin_right).min(min_table_w.max(650.0));

    // Cell padding matches editor's exact table slot layout (10.0px horizontal)
    let cell_pad_x = 10.0;
    let col_w = ((table_w - cell_pad_x * 2.0) / col_count as f32).max(font_size * 4.0);

    // 1. Precompute header galleys and height matching editor table_metrics
    let (_, header_min_h) = table_metrics(font_size, true, false, false);
    let header_galleys: Vec<_> = headers
        .iter()
        .map(|h_text| {
            let job = build_inline_job(h_text, font_size * 0.95, cell_color(theme, true), theme, col_w - cell_pad_x * 2.0);
            painter.layout_job(job)
        })
        .collect();
    let header_h = header_galleys
        .iter()
        .map(|g| g.size().y + 8.0)
        .fold(header_min_h, f32::max);

    // 2. Precompute data row galleys and heights matching editor table_metrics
    let (_, row_min_h) = table_metrics(font_size, false, false, false);
    let mut row_galleys_list: Vec<Vec<_>> = Vec::with_capacity(rows.len());
    let mut row_heights: Vec<f32> = Vec::with_capacity(rows.len());

    for row in rows {
        let mut row_galleys = Vec::with_capacity(col_count);
        let mut max_cell_h = row_min_h;
        for c_idx in 0..col_count {
            let cell_text = row.get(c_idx).map(|s| s.as_str()).unwrap_or("");
            let job = build_inline_job(cell_text, font_size * 0.90, cell_color(theme, false), theme, col_w - cell_pad_x * 2.0);
            let galley = painter.layout_job(job);
            if galley.size().y + 6.0 > max_cell_h {
                max_cell_h = galley.size().y + 6.0;
            }
            row_galleys.push(galley);
        }
        row_heights.push(max_cell_h);
        row_galleys_list.push(row_galleys);
    }

    let total_table_h = header_h + row_heights.iter().sum::<f32>();

    // Viewport frustum culling
    if current_y + total_table_h >= viewport_rect.min.y && current_y <= viewport_rect.max.y {
        let table_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(table_w, total_table_h));
        let header_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(table_w, header_h));

        // Outer container border and header decorations matching editor styling directly (DRY)
        render_table_block_decorations(
            content_painter,
            table_rect,
            Some(header_rect),
            theme,
        );

        // Strictly clip table cell drawing so zoomed text never bleeds past table boundary
        let table_painter = content_painter.with_clip_rect(table_rect.expand(1.0));

        // Render header cell galleys
        for (c_idx, galley) in header_galleys.iter().enumerate() {
            let cx = start_x + cell_pad_x + (c_idx as f32 * col_w);
            let cy = current_y + ((header_h - galley.size().y) * 0.5).max(4.0);
            table_painter.galley(pos2(cx, cy), galley.clone(), cell_color(theme, true));
        }

        // Render data rows
        let mut row_y = current_y + header_h;
        for (r_idx, (r_galleys, &rh)) in row_galleys_list.iter().zip(row_heights.iter()).enumerate() {
            let row_rect = Rect::from_min_size(pos2(start_x, row_y), vec2(table_w, rh));
            let is_last = r_idx + 1 == rows.len();

            // Alternating tint & hairline divider matching editor styling directly (DRY)
            render_table_row_decorations(&table_painter, row_rect, theme, r_idx, is_last);

            // Render cell galleys
            for (c_idx, galley) in r_galleys.iter().enumerate() {
                let cx = start_x + cell_pad_x + (c_idx as f32 * col_w);
                let cy = row_y + ((rh - galley.size().y) * 0.5).max(3.0);
                table_painter.galley(pos2(cx, cy), galley.clone(), cell_color(theme, false));
            }

            row_y += rh;
        }
    }

    total_table_h + 10.0
}
