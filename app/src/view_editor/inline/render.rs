pub mod active;
pub mod decorations;
pub mod inactive;

use super::elements::{
    code_block_copy_button_rect, render_block_quote_wrapper,
    render_document_selection, render_horizontal_rule,
    render_table_row_decorations, render_task_checkbox,
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
    // Match normal editor pad_y exactly so Ctrl+E switching is seamless (body.rs uses 10.0).
    let pad_y = 10.0;
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

    let right_pad = 28.0;
    let content_right = (editor_rect.max.x - right_pad).max(text_left + 100.0);
    let table_margin_right = 32.0;
    let table_avail_w = (editor_rect.max.x - text_left - table_margin_right).max(120.0);
    let min_table_w_for_font = (font_size * 25.0).max(400.0);
    let table_w = table_avail_w.min(min_table_w_for_font.max(650.0));

    // 1. Intercept Copy Button clicks so clicking Copy never shifts caret or expands raw code fences
    let mut clicked_copy_button = false;
    let mut blk_check = 0;
    while blk_check < layout.lines.len() {
        if matches!(layout.lines[blk_check].kind, InlineLineKind::CodeFence(_) | InlineLineKind::CodeLine) {
            let start_k = blk_check;
            let mut end_k = blk_check;
            while end_k + 1 < layout.lines.len()
                && matches!(layout.lines[end_k + 1].kind, InlineLineKind::CodeFence(_) | InlineLineKind::CodeLine)
            {
                end_k += 1;
                if matches!(layout.lines[end_k].kind, InlineLineKind::CodeFence(_)) {
                    break;
                }
            }
            let top_y = ed_origin.y + layout.lines[start_k].y_offset;
            let btn_rect = code_block_copy_button_rect(content_right, top_y);
            let header_bar_rect = Rect::from_min_max(
                pos2(text_left, top_y),
                pos2(content_right, top_y + 28.0),
            );

            let is_btn_hovered = ui.rect_contains_pointer(btn_rect.expand(3.0));
            if is_btn_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                clicked_copy_button = true;
                if ui.input(|i| i.pointer.primary_clicked()) {
                    let mut code_text = String::new();
                    for k in (start_k + 1)..end_k {
                        let l_chars = &layout.lines[k];
                        let raw_line: String = ed.buf[l_chars.char_start..l_chars.char_end].iter().collect();
                        code_text.push_str(&raw_line);
                        code_text.push('\n');
                    }
                    ui.ctx().copy_text(code_text);
                    let copy_id = ui.id().with(("inline_code_block_copy", start_k));
                    let current_time = ui.input(|i| i.time);
                    ui.data_mut(|d| d.insert_temp(copy_id, current_time));
                    ui.ctx().request_repaint();
                    break;
                }
            } else if ui.rect_contains_pointer(header_bar_rect) && ui.input(|i| i.pointer.primary_clicked() || i.pointer.primary_down()) {
                // Clicking on header bar outside button should not expand raw fence
                clicked_copy_button = true;
            }

            blk_check = end_k + 1;
        } else {
            blk_check += 1;
        }
    }

    // Interactive mouse clicks, drag selection, and task checkboxes
    if !clicked_copy_button && handle_inline_mouse_interaction(
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

    let mouse_pos = ui.input(|i| i.pointer.interact_pos());

    // ── Layer 1: Unified Container Cards & Background Decorations ────────────
    // Render Unified Code Block Container Cards (matching preview elevated surface, no broken line strips)
    decorations::render_code_block_containers(
        ui,
        &editor_painter,
        &layout,
        ed,
        ed_origin,
        editor_rect,
        text_left,
        content_right,
        theme,
    );

    // Render Unified Table Container Cards (sleek, matching preview.rs)
    decorations::render_table_containers(
        &editor_painter,
        &layout,
        ed_origin,
        editor_rect,
        text_left,
        table_w,
        theme,
    );

    // Render line-level card backgrounds (Blockquote callout cards & table row alternating tints)
    for (line_idx, line) in layout.lines.iter().enumerate() {
        let line_y = ed_origin.y + line.y_offset;

        if line_y + line.height < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }

        let is_line_active = ed.cur >= line.char_start && ed.cur <= line.char_end;

        // Blockquote modern wrapper card (without harsh left stripe, continuous across multi-lines)
        if let InlineLineKind::Quote(depth) = line.kind {
            let is_first = line_idx == 0 || !matches!(layout.lines[line_idx - 1].kind, InlineLineKind::Quote(d) if d == depth);
            let is_last = line_idx + 1 >= layout.lines.len() || !matches!(layout.lines[line_idx + 1].kind, InlineLineKind::Quote(d) if d == depth);
            render_block_quote_wrapper(
                &editor_painter,
                line_y,
                line.height,
                text_left,
                content_right,
                theme,
                is_line_active,
                depth,
                is_first,
                is_last,
            );
        }

        // Markdown Table Row decorations (alternating tint and subtle dividers for data rows)
        if let InlineLineKind::TableRow(ref info) = line.kind {
            if !info.is_header && !info.is_separator && !is_line_active {
                let mut row_idx = 0;
                let mut k = line_idx;
                while k > 0 && matches!(layout.lines[k - 1].kind, InlineLineKind::TableRow(_)) {
                    k -= 1;
                    if let InlineLineKind::TableRow(ref prev_info) = layout.lines[k].kind {
                        if !prev_info.is_header && !prev_info.is_separator {
                            row_idx += 1;
                        }
                    }
                }
                let is_last = line_idx + 1 >= layout.lines.len()
                    || !matches!(layout.lines[line_idx + 1].kind, InlineLineKind::TableRow(_));
                let row_rect = Rect::from_min_size(pos2(text_left, line_y), vec2(table_w, line.height));
                render_table_row_decorations(
                    &editor_painter,
                    row_rect,
                    theme,
                    row_idx,
                    is_last,
                );
            }
        }
    }

    // ── Layer 2: Continuous Unified Selection Layer (The Lumina Standard) ────
    // Renders ON TOP of card backgrounds so selection is never hidden under cards or table rows
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

    // ── Layer 3: Foreground Text, Interactive Widgets, Rules & Search Highlights ──
    for line in &layout.lines {
        let line_y = ed_origin.y + line.y_offset;

        if line_y + line.height < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }

        let is_line_active = ed.cur >= line.char_start && ed.cur <= line.char_end;

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
                let screen_box = box_rect.translate(vec2(ed_origin.x, ed_origin.y));
                render_task_checkbox(
                    &editor_painter,
                    screen_box,
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

                    let galley_y_pad = ((line.height - line.galley.size().y) * 0.5).round().max(0.0);
                    let m_rect = Rect::from_min_max(
                        pos2(text_left + r1.min.x, line_y + galley_y_pad + r1.min.y),
                        pos2(text_left + r2.max.x.max(r1.min.x + 6.0), line_y + galley_y_pad + r1.min.y + r1.height().max(16.0)),
                    );
                    editor_painter.rect_filled(m_rect, 2.0, match_color);
                }
            }
        }

        // 7. Render Text Galley (vertically centered in line slot, clipped for table rows)
        let galley_y_pad = ((line.height - line.galley.size().y) * 0.5).round().max(0.0);
        if let InlineLineKind::TableRow(_) = line.kind {
            let row_clip = Rect::from_min_size(pos2(text_left, line_y), vec2(table_w, line.height)).expand(1.0);
            let clipped_painter = editor_painter.with_clip_rect(row_clip);
            clipped_painter.galley(pos2(text_left, line_y + galley_y_pad), line.galley.clone(), theme.text);
        } else {
            editor_painter.galley(pos2(text_left, line_y + galley_y_pad), line.galley.clone(), theme.text);
        }
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
        let num_font = FontId::monospace((font_size * 0.82).round().max(9.0));

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

                // Pinned to top of row (matching body.rs and standard IDEs) so heading line height changes never jump
                gutter_painter.text(
                    pos2(gutter_rect.max.x - 6.0, line_y + 3.0),
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
