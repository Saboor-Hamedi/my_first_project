//! Multi-line document layout calculation and Galley compilation.

use super::parser::{build_line_layout, classify_line};
use super::types::{InlineEditorLayout, InlineLine, InlineLineKind};
use crate::editor::Editor;
use crate::theme::Theme;
use eframe::egui::{pos2, vec2, Rect};

/// Computes the complete layout of all lines in the editor buffer.
pub fn compute_inline_layout(
    ui: &eframe::egui::Ui,
    ed: &Editor,
    wrap_width: f32,
    base_font_size: f32,
    theme: &Theme,
    ed_origin_x: f32,
) -> InlineEditorLayout {
    let mut layout = InlineEditorLayout::new();
    let buf = &ed.buf;
    let n = buf.len();

    let mut line_start = 0;
    let mut cumulative_y = 0.0;
    let mut in_code_block = false;

    while line_start <= n {
        let mut line_end = line_start;
        while line_end < n && buf[line_end] != '\n' {
            line_end += 1;
        }

        let line_chars = &buf[line_start..line_end];
        let (mut kind, _prefix_len) = classify_line(line_chars);

        // Check code fence state
        if let InlineLineKind::CodeFence(_) = &kind {
            in_code_block = !in_code_block;
        } else if in_code_block {
            kind = InlineLineKind::CodeLine;
        }

        // Active line determination: cursor sits on this line
        let is_active = ed.cur >= line_start && (ed.cur <= line_end || line_end == n);

        let (mut job, char_map, min_line_h) = build_line_layout(
            line_chars,
            line_start,
            is_active,
            base_font_size,
            theme,
            &kind,
            in_code_block && !matches!(kind, InlineLineKind::CodeFence(_)),
        );

        job.wrap.max_width = wrap_width.max(100.0);
        let galley = ui.fonts(|f| f.layout_job(job));

        let galley_h = galley.size().y;
        let line_h = galley_h.max(min_line_h);

        // Compute interactive checkbox bounds if applicable
        let checkbox_rect = if let InlineLineKind::TaskItem { .. } = kind {
            let box_size = 14.0;
            let box_x = ed_origin_x + 3.0;
            let box_y = cumulative_y + (line_h - box_size) * 0.5;
            Some(Rect::from_min_size(pos2(box_x, box_y), vec2(box_size, box_size)))
        } else {
            None
        };

        layout.lines.push(InlineLine {
            char_start: line_start,
            char_end: line_end,
            y_offset: cumulative_y,
            height: line_h,
            base_font_size,
            kind,
            galley,
            char_map,
            checkbox_rect,
        });

        cumulative_y += line_h;

        if line_end >= n {
            break;
        }
        line_start = line_end + 1;
    }

    layout.total_height = cumulative_y;
    layout
}
