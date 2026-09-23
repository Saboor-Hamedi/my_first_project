use super::elements::{
    render_block_quote_wrapper, render_code_wrapper_line, render_document_selection,
    render_horizontal_rule, render_table_row_decorations, render_task_checkbox,
};
use super::interaction::handle_inline_mouse_interaction;
use super::layout::compute_inline_layout;
use super::types::InlineLineKind;
use crate::caret::Caret;
use crate::editor::Editor;
use crate::theme::Theme;
use eframe::egui::text::CCursor;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, Ui,
};

/// Renders the inline live markdown editor with dynamic typography, checkboxes,
/// blockquotes, headings, code highlights, and caret scaling.
pub fn render_inline_editor(
    ui: &Ui,
    painter: &egui::Painter,
    _window_bounds: Rect,
    editor_rect: Rect,
    ed: &mut Editor,
    caret: &mut Caret,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
    dt: f32,
    now: f64,
    mut typed: bool,
    block_scroll: bool,
    search_matches: Option<(&[usize], usize)>,
    show_line_numbers: bool,
    sound: &mut crate::sound::SoundEngine,
    is_dirty: &mut bool,
) {
    let visible_h = editor_rect.height().max(0.0);
    if visible_h < 10.0 {
        return;
    }

    let gutter_w = if show_line_numbers {
        let total_lines = (ed.buf.iter().filter(|&&c| c == '\n').count() + 1).max(1);
        let digits = total_lines.to_string().len().max(2);
        (digits as f32 * (font_size * 0.55) + 14.0).max(28.0)
    } else {
        0.0
    };

    let pad_x = if show_line_numbers { 16.0 } else { 24.0 };
    let pad_y = 12.0;
    let safe_w = editor_rect.width();
    let effective_gutter_w = if safe_w > gutter_w + 40.0 { gutter_w } else { 0.0 };
    let text_left = (editor_rect.min.x + effective_gutter_w + pad_x).min(editor_rect.max.x);
    let ed_origin = pos2(text_left, editor_rect.min.y - *scroll_y + pad_y);

    let wrap_w = (editor_rect.max.x - text_left - 24.0).max(120.0);

    // Compute complete inline document layout
    let layout = compute_inline_layout(ui, ed, wrap_w, font_size, theme, text_left);

    let total_content_h = layout.total_height;
    let max_scroll = (total_content_h + 32.0 - visible_h + font_size * 4.0).max(0.0);

    // Mouse wheel scrolling
    if !block_scroll && ui.rect_contains_pointer(editor_rect) {
        let is_ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
        if !is_ctrl {
            let scroll_delta = ui.input(|i| {
                if i.raw_scroll_delta.y.abs() > 0.0 {
                    i.raw_scroll_delta.y
                } else {
                    i.smooth_scroll_delta.y
                }
            });
            if scroll_delta.abs() > 0.0 {
                *scroll_y = (*scroll_y - scroll_delta).clamp(0.0, max_scroll);
            }
        }
    }

    // Interactive mouse clicks, drag selection, and task checkboxes
    if handle_inline_mouse_interaction(
        ui,
        editor_rect,
        ed_origin,
        &layout,
        ed,
        block_scroll,
        sound,
        is_dirty,
    ) {
        typed = true;
    }

    // Caret position & line height calculation
    let (caret_target, caret_h) = layout.pos_for_char(ed.cur, ed_origin);
    let caret_y_in_content = caret_target.y - ed_origin.y;

    // Auto-scroll follow when typing or navigating by keyboard
    if typed {
        if caret_y_in_content + caret_h + pad_y > *scroll_y + visible_h {
            *scroll_y = (caret_y_in_content + caret_h + pad_y - visible_h).clamp(0.0, max_scroll);
        }
        if caret_y_in_content < *scroll_y {
            *scroll_y = caret_y_in_content.clamp(0.0, max_scroll);
        }
    }

    let editor_painter = painter.with_clip_rect(editor_rect);

    // Selection background color derived cleanly from theme accent
    let sel_range = ed.selected_range();
    let sel_color = if theme.is_light() {
        Color32::from_rgba_unmultiplied(
            theme.accent.r(),
            theme.accent.g(),
            theme.accent.b(),
            90,
        )
    } else {
        Color32::from_rgba_unmultiplied(
            theme.accent.r(),
            theme.accent.g(),
            theme.accent.b(),
            60,
        )
    };

    // 2. Render Continuous Unified Selection Layer (The Lumina Standard)
    if let Some((sel_start, sel_end)) = sel_range {
        render_document_selection(
            &editor_painter,
            &layout,
            ed_origin,
            text_left,
            sel_start,
            sel_end,
            sel_color,
        );
    }

    let mouse_pos = ui.input(|i| i.pointer.interact_pos());

    // Frustum culling: render only lines intersecting visible viewport
    for line in &layout.lines {
        let line_y = ed_origin.y + line.y_offset;

        if line_y + line.height < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }

        let is_line_active = ed.cur >= line.char_start && ed.cur <= line.char_end;
        let content_right = editor_rect.max.x - 16.0;

        // 1. Render Blockquote modern wrapper card (without harsh left stripe)
        if let InlineLineKind::Quote = line.kind {
            render_block_quote_wrapper(
                &editor_painter,
                line_y,
                line.height,
                text_left,
                content_right,
                theme,
                is_line_active,
            );
        }

        // 2. Render Code Line & Fence surface wrapper
        match &line.kind {
            InlineLineKind::CodeLine => {
                render_code_wrapper_line(
                    &editor_painter,
                    line_y,
                    line.height,
                    text_left,
                    content_right,
                    theme,
                    false,
                    None,
                    is_line_active,
                );
            }
            InlineLineKind::CodeFence(lang) => {
                render_code_wrapper_line(
                    &editor_painter,
                    line_y,
                    line.height,
                    text_left,
                    content_right,
                    theme,
                    true,
                    Some(lang.as_str()),
                    is_line_active,
                );
            }
            _ => {}
        }

        // 3. Render Markdown Table Row decorations
        if let InlineLineKind::TableRow { is_header, is_separator } = line.kind {
            render_table_row_decorations(
                &editor_painter,
                line_y,
                line.height,
                text_left,
                content_right,
                theme,
                is_header,
                is_separator,
                is_line_active,
            );
        }

        // 4. Render Horizontal Rule
        if let InlineLineKind::Rule = line.kind {
            render_horizontal_rule(
                &editor_painter,
                line_y,
                line.height,
                text_left,
                content_right,
                theme,
                is_line_active,
            );
        }

        // 5. Render Interactive Checkbox for TaskItem
        if let InlineLineKind::TaskItem { checked, .. } = line.kind {
            if let Some(box_rect) = line.checkbox_rect {
                render_task_checkbox(
                    &editor_painter,
                    box_rect,
                    checked,
                    mouse_pos,
                    theme,
                );
            }
        }

        // 6. Render In-buffer Search Highlights
        if let Some((matches, q_len)) = search_matches {
            if q_len > 0 {
                for &m_idx in matches {
                    if m_idx + q_len <= line.char_start || m_idx >= line.char_end {
                        continue;
                    }
                    let m_start = m_idx.max(line.char_start);
                    let m_end = (m_idx + q_len).min(line.char_end);

                    let mut start_g = 0;
                    let mut end_g = line.char_map.len().saturating_sub(1);

                    for (g, &b) in line.char_map.iter().enumerate() {
                        if b <= m_start {
                            start_g = g;
                        }
                        if b >= m_end {
                            end_g = g;
                            break;
                        }
                    }

                    let c1 = line.galley.from_ccursor(CCursor::new(start_g));
                    let c2 = line.galley.from_ccursor(CCursor::new(end_g));
                    let r1 = line.galley.pos_from_cursor(&c1);
                    let r2 = line.galley.pos_from_cursor(&c2);

                    let is_active_match = m_idx == ed.cur;
                    let match_color = if is_active_match {
                        Color32::from_rgba_unmultiplied(255, 195, 45, 130)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 215, 60, 55)
                    };

                    let m_rect = Rect::from_min_max(
                        pos2(text_left + r1.min.x, line_y + r1.min.y),
                        pos2(text_left + r2.max.x.max(r1.min.x + 6.0), line_y + r1.min.y + r1.height().max(16.0)),
                    );
                    editor_painter.rect_filled(m_rect, 2.0, match_color);
                }
            }
        }

        // 7. Render Text Galley
        editor_painter.galley(pos2(text_left, line_y), line.galley.clone(), theme.text);
    }

    // 8. Render Caret Overlay (Strict Layer Priority: Always renders ON TOP of selection and text)
    let caret_clip = Rect::from_min_max(
        pos2(editor_rect.min.x, editor_rect.min.y - 5.0),
        editor_rect.max,
    );
    let caret_painter = painter.with_clip_rect(caret_clip);
    let approx_cw = (font_size * 0.6).round();
    caret.update(dt, caret_target, typed, now, approx_cw, caret_h);
    caret.paint(&caret_painter, approx_cw, caret_h, now, theme.accent, theme.is_light());

    // Line number gutter on the left
    if show_line_numbers && gutter_w > 0.0 {
        let gutter_rect = Rect::from_min_max(
            editor_rect.min,
            pos2(editor_rect.min.x + gutter_w, editor_rect.max.y),
        );
        let gutter_painter = painter.with_clip_rect(gutter_rect);
        let num_font = FontId::monospace(font_size * 0.8);

        let active_line_idx = layout.line_for_char(ed.cur);

        for (i, line) in layout.lines.iter().enumerate() {
            let line_y = ed_origin.y + line.y_offset;
            if line_y + line.height >= editor_rect.min.y && line_y <= editor_rect.max.y {
                let is_current = i == active_line_idx;
                let num_str = (i + 1).to_string();
                let color = if is_current {
                    theme.accent
                } else if theme.is_light() {
                    theme.muted
                } else {
                    Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 100)
                };

                gutter_painter.text(
                    pos2(gutter_rect.max.x - 6.0, line_y + (line.height * 0.2)),
                    Align2::RIGHT_TOP,
                    num_str,
                    num_font.clone(),
                    color,
                );
            }
        }

        // Vertical hairline separating gutter from text canvas
        let sep_color = Color32::from_rgba_unmultiplied(
            theme.border().r(),
            theme.border().g(),
            theme.border().b(),
            if theme.is_light() { 55 } else { 40 },
        );
        painter.line_segment(
            [pos2(gutter_rect.max.x + 3.0, editor_rect.min.y), pos2(gutter_rect.max.x + 3.0, editor_rect.max.y)],
            Stroke::new(1.0, sep_color),
        );
    }

    // Interactive scrollbar in right margin gutter
    if visible_h > 35.0 && total_content_h > visible_h && max_scroll > 0.0 {
        let max_thumb = visible_h.max(28.0);
        let thumb_h = ((visible_h / total_content_h) * visible_h).clamp(28.0, max_thumb);
        let scroll_ratio = (*scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = editor_rect.min.y + scroll_ratio * (visible_h - thumb_h);
        let track_x = editor_rect.max.x - 8.0;
        let thumb_rect = Rect::from_min_size(pos2(track_x - 1.5, thumb_y), vec2(3.0, thumb_h));
        let track_rect = Rect::from_min_max(
            pos2(track_x - 8.0, editor_rect.min.y),
            pos2(editor_rect.max.x, editor_rect.max.y),
        );

        let is_track_hovered = ui.rect_contains_pointer(track_rect);
        if is_track_hovered && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let avail_track = (visible_h - thumb_h).max(1.0);
                let ratio = ((pos.y - editor_rect.min.y - thumb_h * 0.5) / avail_track).clamp(0.0, 1.0);
                *scroll_y = ratio * max_scroll;
            }
        }

        let thumb_color = if is_track_hovered {
            Color32::from_rgba_unmultiplied(180, 185, 200, 160)
        } else {
            Color32::from_rgba_unmultiplied(120, 125, 140, 70)
        };
        painter.rect_filled(thumb_rect, 1.5, thumb_color);
    }
}
