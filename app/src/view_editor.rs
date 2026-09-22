//! Main text editor view: header, text canvas, smooth scrolling, and caret rendering.

use crate::caret::Caret;
use crate::editor::{Editor, VisualLine};
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

/// Renders the clean editor header with active title and top-right window drag gripper.
pub fn render_editor_header(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    content_left_margin: f32,
    content_right_margin: f32,
    active_title: &str,
    is_dirty: bool,
    theme: &Theme,
) {
    let header_y = bounds.min.y + 14.0;
    let base_title = if active_title.len() > 40 {
        format!("📄 {}...", &active_title[..40])
    } else {
        format!("📄 {}", active_title)
    };
    let display_title = if is_dirty {
        format!("{}  ●", base_title)
    } else {
        base_title
    };

    painter.text(
        pos2(bounds.min.x + content_left_margin, header_y),
        Align2::LEFT_TOP,
        display_title,
        FontId::monospace(13.5),
        theme.highlight,
    );

    // Sleek Window Gripper on the top right side for dragging borderless window
    let gripper_rect = Rect::from_min_size(
        pos2(bounds.max.x - 38.0, bounds.min.y + 11.0),
        vec2(24.0, 20.0),
    );
    let is_hovered = ui.rect_contains_pointer(gripper_rect);
    if is_hovered && ui.input(|i| i.pointer.primary_down()) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    let grip_color = if is_hovered {
        theme.accent
    } else {
        Color32::from_gray(80)
    };

    // 6-dot matrix gripper (2 cols x 3 rows)
    for col in 0..2 {
        for row in 0..3 {
            let cx = gripper_rect.min.x + 6.0 + col as f32 * 8.0;
            let cy = gripper_rect.min.y + 4.0 + row as f32 * 6.0;
            painter.circle_filled(pos2(cx, cy), 1.5, grip_color);
        }
    }

    // Subtle divider line under header, symmetrically aligned with content margins
    painter.line_segment(
        [
            pos2(bounds.min.x + content_left_margin, bounds.min.y + 38.0),
            pos2(bounds.max.x - content_right_margin, bounds.min.y + 38.0),
        ],
        Stroke::new(1.0, Color32::from_gray(24)),
    );
}

/// Renders the editor body with soft-wrapped visual lines, smooth scrolling, caret animation, and interactive scrollbar.
pub fn render_editor_body(
    ui: &egui::Ui,
    painter: &egui::Painter,
    window_bounds: Rect,
    editor_rect: Rect,
    ed: &mut Editor,
    visual_lines: &[VisualLine],
    caret: &mut Caret,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
    cw: f32,
    lh: f32,
    dt: f32,
    now: f64,
    mut typed: bool,
    block_scroll: bool,
    search_matches: Option<(&[usize], usize)>,
) {
    let font = FontId::monospace(font_size);
    let font_h = painter.layout_no_wrap("M".to_owned(), font.clone(), Color32::WHITE).size().y;
    let y_pad = ((lh - font_h) * 0.5).round().max(0.0);
    let stroke_w = (font_size * 0.088).clamp(1.2, 1.8);
    let visible_h = editor_rect.height();

    let total_content_h = visual_lines.len() as f32 * lh;
    let max_scroll = (total_content_h - visible_h + lh * 4.0).max(0.0);

    // Mouse wheel scrolling: prioritize smooth scroll delta, fallback to scaled raw delta
    if !block_scroll {
        let scroll_delta = ui.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.001 {
                i.smooth_scroll_delta.y
            } else {
                i.raw_scroll_delta.y * 0.5
            }
        });
        if scroll_delta != 0.0 && ui.rect_contains_pointer(editor_rect) {
            *scroll_y = (*scroll_y - scroll_delta).clamp(0.0, max_scroll);
        }
    }

    let pad_x = 4.0;
    let pad_y = 2.0;
    let ed_origin = editor_rect.min - vec2(0.0, *scroll_y) + vec2(pad_x, pad_y);

    // Direct mouse click in editor moves cursor to clicked visual row and col
    if !block_scroll && ui.rect_contains_pointer(editor_rect) && ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let clicked_row = ((pos.y - ed_origin.y) / lh).floor() as isize;
            if clicked_row >= 0 && (clicked_row as usize) < visual_lines.len() {
                let r = clicked_row as usize;
                let line = &visual_lines[r];
                let clicked_col = (((pos.x - ed_origin.x).max(0.0)) / cw).round() as usize;
                let line_len = line.char_end.saturating_sub(line.char_start);
                ed.cur = line.char_start + clicked_col.min(line_len);
                ed.selection = None;
                typed = true;
            }
        }
    }

    // Caret position in visual lines
    let (row, col) = ed.visual_row_col(visual_lines);
    let caret_y_in_content = row as f32 * lh;

    // Auto-scroll / caret follow: ONLY when user is typing or navigating by keyboard!
    if typed {
        // Pin smoothly at bottom edge when moving downwards
        if caret_y_in_content + lh > *scroll_y + visible_h {
            *scroll_y = (caret_y_in_content + lh - visible_h).clamp(0.0, max_scroll);
        }
        // Pin smoothly at top edge when moving upwards
        if caret_y_in_content < *scroll_y {
            *scroll_y = caret_y_in_content.clamp(0.0, max_scroll);
        }
    }

    // Clip drawing strictly to editor bounds
    let editor_painter = painter.with_clip_rect(editor_rect);

    let sel_range = ed.selected_range();
    let sel_color = Color32::from_rgba_unmultiplied(
        theme.accent.r(),
        theme.accent.g(),
        theme.accent.b(),
        65,
    );

    // Caret placement and animation
    let caret_x = ed_origin.x + col as f32 * cw;
    let target = pos2(caret_x, ed_origin.y + row as f32 * lh);
    caret.update(dt, target, typed, now, cw, lh);
    caret.paint(&editor_painter, cw, lh, now, theme.accent);

    let is_block = caret.kind == crate::caret::CaretKind::Block;

    // Frustum culling: render only lines intersecting the visible viewport
    for (r, line) in visual_lines.iter().enumerate() {
        let line_y = ed_origin.y + r as f32 * lh;
        if line_y + lh < editor_rect.min.y || line_y > editor_rect.max.y {
            continue;
        }

        // Render selection highlight background behind text on this line
        if let Some((sel_start, sel_end)) = sel_range {
            if line.char_start == line.char_end {
                // Empty line gap between paragraphs (\n\n):
                // Highlight a visible block only when the active selection spans across this empty line.
                if sel_start <= line.char_start && sel_end > line.char_end {
                    let sel_w = cw.max(12.0);
                    let highlight_rect = Rect::from_min_size(pos2(ed_origin.x, line_y), vec2(sel_w, lh + 0.5));
                    editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
                }
            } else {
                let intersect_start = sel_start.max(line.char_start);
                let intersect_end = sel_end.min(line.char_end);
                if intersect_start < intersect_end {
                    let start_col = intersect_start - line.char_start;
                    let end_col = intersect_end - line.char_start;
                    let sel_x = ed_origin.x + start_col as f32 * cw;
                    let mut sel_w = (end_col - start_col) as f32 * cw;
                    // If selection extends past this line's content (e.g. across newline to subsequent lines),
                    // extend the highlight to visually show that the newline / trailing whitespace is selected.
                    if sel_end > line.char_end {
                        sel_w += cw.max(10.0);
                    }
                    let highlight_rect = Rect::from_min_size(pos2(sel_x, line_y), vec2(sel_w, lh + 0.5));
                    editor_painter.rect_filled(highlight_rect, 0.0, sel_color);
                }
            }
        }

        // Render in-buffer search match highlights behind text on this line
        if let Some((matches, q_len)) = search_matches {
            if q_len > 0 {
                for &m_idx in matches {
                    if m_idx + q_len <= line.char_start || m_idx >= line.char_end {
                        continue;
                    }
                    let match_start = m_idx.max(line.char_start);
                    let match_end = (m_idx + q_len).min(line.char_end);
                    if match_start < match_end {
                        let start_col = match_start - line.char_start;
                        let end_col = match_end - line.char_start;
                        let match_x = ed_origin.x + start_col as f32 * cw;
                        let match_w = (end_col - start_col) as f32 * cw;
                        let is_current = m_idx == ed.cur;
                        let match_color = if is_current {
                            Color32::from_rgba_unmultiplied(255, 195, 45, 120) // brighter amber for active match
                        } else {
                            Color32::from_rgba_unmultiplied(255, 215, 60, 50)  // soft subtle amber
                        };
                        editor_painter.rect_filled(
                            Rect::from_min_size(pos2(match_x, line_y), vec2(match_w, lh)),
                            2.0,
                            match_color,
                        );
                    }
                }
            }
        }

        let text_y = line_y + y_pad;
        let y_mid = line_y + (lh * 0.5).round();

        let line_len = line.char_end.saturating_sub(line.char_start);
        let block_col = if r == row && is_block && col < line_len {
            Some(col)
        } else {
            None
        };
        let line_chars = &ed.buf[line.char_start..line.char_end];
        render_line_with_ligatures(
            &editor_painter,
            ed_origin.x,
            text_y,
            line_y,
            y_mid,
            line_chars,
            &font,
            theme.text,
            cw,
            lh,
            stroke_w,
            block_col,
        );
    }

    // Interactive scrollbar indicator in the right margin gutter
    if total_content_h > visible_h && max_scroll > 0.0 {
        let thumb_h = ((visible_h / total_content_h) * visible_h).clamp(28.0, visible_h);
        let scroll_ratio = (*scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = editor_rect.min.y + scroll_ratio * (visible_h - thumb_h);
        let track_x = window_bounds.max.x - 8.0;
        let thumb_rect = Rect::from_min_size(pos2(track_x - 1.5, thumb_y), vec2(3.0, thumb_h));
        let track_rect = Rect::from_min_max(
            pos2(track_x - 8.0, editor_rect.min.y),
            pos2(window_bounds.max.x, editor_rect.max.y),
        );

        let is_track_hovered = ui.rect_contains_pointer(track_rect);
        if is_track_hovered && ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let ratio = ((pos.y - editor_rect.min.y - thumb_h * 0.5) / (visible_h - thumb_h)).clamp(0.0, 1.0);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LigatureKind {
    ArrowRight,        // ->
    ArrowLeft,         // <-
    FatArrowRight,     // =>
    TripleEquals,      // ===
    NotTripleEquals,   // !==
    DoubleEquals,      // ==
    NotEquals,         // !=
    LessOrEqual,       // <=
    GreaterOrEqual,    // >=
    LongArrowRight,    // -->
    LongArrowLeft,     // <--
    LongFatArrowRight, // ==>
    LongFatArrowLeft,  // <==
}

impl LigatureKind {
    fn char_len(&self) -> usize {
        match self {
            Self::TripleEquals
            | Self::NotTripleEquals
            | Self::LongArrowRight
            | Self::LongArrowLeft
            | Self::LongFatArrowRight
            | Self::LongFatArrowLeft => 3,
            Self::ArrowRight
            | Self::ArrowLeft
            | Self::FatArrowRight
            | Self::DoubleEquals
            | Self::NotEquals
            | Self::LessOrEqual
            | Self::GreaterOrEqual => 2,
        }
    }
}

fn detect_ligature(chars: &[char], i: usize) -> Option<LigatureKind> {
    if i + 3 <= chars.len() {
        match (chars[i], chars[i + 1], chars[i + 2]) {
            ('=', '=', '=') => return Some(LigatureKind::TripleEquals),
            ('!', '=', '=') => return Some(LigatureKind::NotTripleEquals),
            ('-', '-', '>') => return Some(LigatureKind::LongArrowRight),
            ('<', '-', '-') => return Some(LigatureKind::LongArrowLeft),
            ('=', '=', '>') => return Some(LigatureKind::LongFatArrowRight),
            ('<', '=', '=') => return Some(LigatureKind::LongFatArrowLeft),
            _ => {}
        }
    }
    if i + 2 <= chars.len() {
        match (chars[i], chars[i + 1]) {
            ('-', '>') => return Some(LigatureKind::ArrowRight),
            ('<', '-') => return Some(LigatureKind::ArrowLeft),
            ('=', '>') => return Some(LigatureKind::FatArrowRight),
            ('<', '=') => return Some(LigatureKind::LessOrEqual),
            ('>', '=') => return Some(LigatureKind::GreaterOrEqual),
            ('=', '=') => return Some(LigatureKind::DoubleEquals),
            ('!', '=') => return Some(LigatureKind::NotEquals),
            _ => {}
        }
    }
    None
}

fn draw_ligature(
    painter: &egui::Painter,
    kind: LigatureKind,
    x_start: f32,
    y_mid: f32,
    cw: f32,
    stroke: Stroke,
) {
    let cols = kind.char_len() as f32;
    let x_end = x_start + cols * cw;
    let dx = cw * 0.44;
    let dy = cw * 0.38;
    let bar_gap = (cw * 0.17).clamp(2.0, 3.2);

    match kind {
        LigatureKind::ArrowRight => {
            let tip = pos2(x_end - cw * 0.18, y_mid);
            let start = pos2(x_start + cw * 0.10, y_mid);
            painter.line_segment([start, tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::ArrowLeft => {
            let tip = pos2(x_start + cw * 0.18, y_mid);
            let end = pos2(x_end - cw * 0.10, y_mid);
            painter.line_segment([tip, end], stroke);
            painter.line_segment([pos2(tip.x + dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x + dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::LongArrowRight => {
            let tip = pos2(x_end - cw * 0.18, y_mid);
            let start = pos2(x_start + cw * 0.10, y_mid);
            painter.line_segment([start, tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::LongArrowLeft => {
            let tip = pos2(x_start + cw * 0.18, y_mid);
            let end = pos2(x_end - cw * 0.10, y_mid);
            painter.line_segment([tip, end], stroke);
            painter.line_segment([pos2(tip.x + dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x + dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::FatArrowRight => {
            let tip = pos2(x_end - cw * 0.18, y_mid);
            let shaft_end_x = tip.x - dx * 0.45;
            let start_x = x_start + cw * 0.10;
            painter.line_segment(
                [pos2(start_x, y_mid - bar_gap), pos2(shaft_end_x, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(start_x, y_mid + bar_gap), pos2(shaft_end_x, y_mid + bar_gap)],
                stroke,
            );
            painter.line_segment([pos2(tip.x - dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::LongFatArrowRight => {
            let tip = pos2(x_end - cw * 0.18, y_mid);
            let shaft_end_x = tip.x - dx * 0.45;
            let start_x = x_start + cw * 0.10;
            painter.line_segment(
                [pos2(start_x, y_mid - bar_gap), pos2(shaft_end_x, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(start_x, y_mid + bar_gap), pos2(shaft_end_x, y_mid + bar_gap)],
                stroke,
            );
            painter.line_segment([pos2(tip.x - dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x - dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::LongFatArrowLeft => {
            let tip = pos2(x_start + cw * 0.18, y_mid);
            let shaft_start_x = tip.x + dx * 0.45;
            let end_x = x_end - cw * 0.10;
            painter.line_segment(
                [pos2(shaft_start_x, y_mid - bar_gap), pos2(end_x, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(shaft_start_x, y_mid + bar_gap), pos2(end_x, y_mid + bar_gap)],
                stroke,
            );
            painter.line_segment([pos2(tip.x + dx, tip.y - dy), tip], stroke);
            painter.line_segment([pos2(tip.x + dx, tip.y + dy), tip], stroke);
        }
        LigatureKind::TripleEquals => {
            let x_left = x_start + cw * 0.10;
            let x_right = x_end - cw * 0.10;
            let sep = (cw * 0.25).clamp(2.8, 4.2);
            painter.line_segment([pos2(x_left, y_mid - sep), pos2(x_right, y_mid - sep)], stroke);
            painter.line_segment([pos2(x_left, y_mid), pos2(x_right, y_mid)], stroke);
            painter.line_segment([pos2(x_left, y_mid + sep), pos2(x_right, y_mid + sep)], stroke);
        }
        LigatureKind::NotTripleEquals => {
            let x_left = x_start + cw * 0.10;
            let x_right = x_end - cw * 0.10;
            let sep = (cw * 0.25).clamp(2.8, 4.2);
            painter.line_segment([pos2(x_left, y_mid - sep), pos2(x_right, y_mid - sep)], stroke);
            painter.line_segment([pos2(x_left, y_mid), pos2(x_right, y_mid)], stroke);
            painter.line_segment([pos2(x_left, y_mid + sep), pos2(x_right, y_mid + sep)], stroke);
            let mid_x = (x_left + x_right) * 0.5;
            painter.line_segment(
                [
                    pos2(mid_x + cw * 0.35, y_mid - sep - cw * 0.22),
                    pos2(mid_x - cw * 0.35, y_mid + sep + cw * 0.22),
                ],
                stroke,
            );
        }
        LigatureKind::DoubleEquals => {
            let x_left = x_start + cw * 0.10;
            let x_right = x_end - cw * 0.10;
            painter.line_segment(
                [pos2(x_left, y_mid - bar_gap), pos2(x_right, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(x_left, y_mid + bar_gap), pos2(x_right, y_mid + bar_gap)],
                stroke,
            );
        }
        LigatureKind::NotEquals => {
            let x_left = x_start + cw * 0.10;
            let x_right = x_end - cw * 0.10;
            painter.line_segment(
                [pos2(x_left, y_mid - bar_gap), pos2(x_right, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(x_left, y_mid + bar_gap), pos2(x_right, y_mid + bar_gap)],
                stroke,
            );
            let mid_x = (x_left + x_right) * 0.5;
            painter.line_segment(
                [
                    pos2(mid_x + cw * 0.35, y_mid - bar_gap - cw * 0.28),
                    pos2(mid_x - cw * 0.35, y_mid + bar_gap + cw * 0.28),
                ],
                stroke,
            );
        }
        LigatureKind::LessOrEqual => {
            let base_y = y_mid + bar_gap + 1.2;
            painter.line_segment(
                [pos2(x_start + cw * 0.20, base_y), pos2(x_end - cw * 0.20, base_y)],
                stroke,
            );
            let tip = pos2(x_start + cw * 0.28, y_mid - 1.5);
            painter.line_segment([pos2(x_end - cw * 0.28, y_mid - cw * 0.42 - 1.5), tip], stroke);
            painter.line_segment([pos2(x_end - cw * 0.28, y_mid + cw * 0.10 - 1.5), tip], stroke);
        }
        LigatureKind::GreaterOrEqual => {
            let base_y = y_mid + bar_gap + 1.2;
            painter.line_segment(
                [pos2(x_start + cw * 0.20, base_y), pos2(x_end - cw * 0.20, base_y)],
                stroke,
            );
            let tip = pos2(x_end - cw * 0.28, y_mid - 1.5);
            painter.line_segment([pos2(x_start + cw * 0.28, y_mid - cw * 0.42 - 1.5), tip], stroke);
            painter.line_segment([pos2(x_start + cw * 0.28, y_mid + cw * 0.10 - 1.5), tip], stroke);
        }
    }
}

fn render_line_with_ligatures(
    painter: &egui::Painter,
    origin_x: f32,
    text_y: f32,
    line_y: f32,
    y_mid: f32,
    chars: &[char],
    font: &FontId,
    text_color: Color32,
    cw: f32,
    lh: f32,
    stroke_w: f32,
    block_cursor_col: Option<usize>,
) {
    let mut i = 0;
    let mut text_start = 0;
    let mut text_buf = String::new();

    let stroke = Stroke::new(stroke_w, text_color);
    let contrast_stroke = Stroke::new(stroke_w, Color32::from_rgb(10, 12, 16));

    let flush_text = |p: &egui::Painter, buf: &mut String, start_col: usize| {
        if buf.is_empty() {
            return;
        }
        let chunk_chars: Vec<char> = buf.chars().collect();
        let chunk_len = chunk_chars.len();
        let chunk_x = origin_x + start_col as f32 * cw;

        if let Some(b_col) = block_cursor_col {
            if b_col >= start_col && b_col < start_col + chunk_len {
                let local_idx = b_col - start_col;
                if local_idx > 0 {
                    let before: String = chunk_chars[..local_idx].iter().collect();
                    p.text(pos2(chunk_x, text_y), Align2::LEFT_TOP, before, font.clone(), text_color);
                }
                let cur_char = chunk_chars[local_idx].to_string();
                let cur_x = origin_x + b_col as f32 * cw;
                p.text(pos2(cur_x, text_y), Align2::LEFT_TOP, cur_char, font.clone(), Color32::from_rgb(10, 12, 16));
                if local_idx + 1 < chunk_len {
                    let after: String = chunk_chars[local_idx + 1..].iter().collect();
                    let after_x = origin_x + (b_col + 1) as f32 * cw;
                    p.text(pos2(after_x, text_y), Align2::LEFT_TOP, after, font.clone(), text_color);
                }
                buf.clear();
                return;
            }
        }

        p.text(pos2(chunk_x, text_y), Align2::LEFT_TOP, buf.as_str(), font.clone(), text_color);
        buf.clear();
    };

    while i < chars.len() {
        if let Some(lig) = detect_ligature(chars, i) {
            flush_text(painter, &mut text_buf, text_start);
            let lig_len = lig.char_len();
            let x_start = origin_x + i as f32 * cw;

            draw_ligature(painter, lig, x_start, y_mid, cw, stroke);

            if let Some(b_col) = block_cursor_col {
                if b_col >= i && b_col < i + lig_len {
                    let cursor_rect = Rect::from_min_size(
                        pos2(origin_x + b_col as f32 * cw, line_y),
                        vec2(cw, lh),
                    );
                    let clipped_painter = painter.with_clip_rect(cursor_rect);
                    draw_ligature(&clipped_painter, lig, x_start, y_mid, cw, contrast_stroke);
                }
            }

            i += lig_len;
            text_start = i;
        } else {
            if text_buf.is_empty() {
                text_start = i;
            }
            text_buf.push(chars[i]);
            i += 1;
        }
    }

    flush_text(painter, &mut text_buf, text_start);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ligatures() {
        let chars: Vec<char> = "fn foo() -> i32 => val === 1 !== 0 <= 5 >= 2 != 3 == 4".chars().collect();
        // find ->
        let arrow_pos = chars.windows(2).position(|w| w == ['-', '>']).unwrap();
        assert_eq!(detect_ligature(&chars, arrow_pos), Some(LigatureKind::ArrowRight));

        // find =>
        let fat_pos = chars.windows(2).position(|w| w == ['=', '>']).unwrap();
        assert_eq!(detect_ligature(&chars, fat_pos), Some(LigatureKind::FatArrowRight));

        // find ===
        let triple_pos = chars.windows(3).position(|w| w == ['=', '=', '=']).unwrap();
        assert_eq!(detect_ligature(&chars, triple_pos), Some(LigatureKind::TripleEquals));

        // find !==
        let not_triple_pos = chars.windows(3).position(|w| w == ['!', '=', '=']).unwrap();
        assert_eq!(detect_ligature(&chars, not_triple_pos), Some(LigatureKind::NotTripleEquals));

        // find <=
        let le_pos = chars.windows(2).position(|w| w == ['<', '=']).unwrap();
        assert_eq!(detect_ligature(&chars, le_pos), Some(LigatureKind::LessOrEqual));

        // find >=
        let ge_pos = chars.windows(2).position(|w| w == ['>', '=']).unwrap();
        assert_eq!(detect_ligature(&chars, ge_pos), Some(LigatureKind::GreaterOrEqual));

        // find !=
        let ne_pos = chars.windows(2).position(|w| w == ['!', '='] && chars.get(arrow_pos).is_some()).unwrap();
        assert_eq!(detect_ligature(&chars, ne_pos), Some(LigatureKind::NotTripleEquals)); // wait, the first != in the string is in !==

        // find the standalone !=
        let standalone_ne = chars.windows(2).enumerate().find(|&(idx, w)| w == ['!', '='] && chars.get(idx + 2) != Some(&'=')).unwrap().0;
        assert_eq!(detect_ligature(&chars, standalone_ne), Some(LigatureKind::NotEquals));

        // find ==
        let eq_pos = chars.windows(2).enumerate().find(|&(idx, w)| w == ['=', '='] && (idx == 0 || chars[idx-1] != '=') && chars.get(idx+2) != Some(&'=')).unwrap().0;
        assert_eq!(detect_ligature(&chars, eq_pos), Some(LigatureKind::DoubleEquals));
    }
}

