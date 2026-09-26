//! Coding font ligature scanner, custom vector geometry renderer, and tests.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LigatureKind {
    ArrowRight,            // ->
    ArrowLeft,             // <-
    FatArrowRight,         // =>
    TripleEquals,          // ===
    NotTripleEquals,       // !==
    DoubleEquals,          // ==
    NotEquals,             // !=
    LessOrEqual,           // <=
    GreaterOrEqual,        // >=
    LongArrowRight,        // -->
    LongArrowLeft,         // <--
    LongFatArrowRight,     // ==>
    LongFatArrowLeft,      // <==
    VeryLongArrowRight,    // --->
    VeryLongArrowLeft,     // <---
    VeryLongFatArrowRight, // ===>
    VeryLongFatArrowLeft,  // <===
}

impl LigatureKind {
    pub fn char_len(&self) -> usize {
        match self {
            Self::VeryLongArrowRight
            | Self::VeryLongArrowLeft
            | Self::VeryLongFatArrowRight
            | Self::VeryLongFatArrowLeft => 4,
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

pub fn detect_ligature(chars: &[char], i: usize) -> Option<LigatureKind> {
    if i + 4 <= chars.len() {
        match (chars[i], chars[i + 1], chars[i + 2], chars[i + 3]) {
            ('=', '=', '=', '>') => return Some(LigatureKind::VeryLongFatArrowRight),
            ('<', '=', '=', '=') => return Some(LigatureKind::VeryLongFatArrowLeft),
            ('-', '-', '-', '>') => return Some(LigatureKind::VeryLongArrowRight),
            ('<', '-', '-', '-') => return Some(LigatureKind::VeryLongArrowLeft),
            _ => {}
        }
    }
    if i + 3 <= chars.len() {
        match (chars[i], chars[i + 1], chars[i + 2]) {
            ('=', '=', '>') => return Some(LigatureKind::LongFatArrowRight),
            ('<', '=', '=') => return Some(LigatureKind::LongFatArrowLeft),
            ('-', '-', '>') => return Some(LigatureKind::LongArrowRight),
            ('<', '-', '-') => return Some(LigatureKind::LongArrowLeft),
            ('=', '=', '=') => return Some(LigatureKind::TripleEquals),
            ('!', '=', '=') => return Some(LigatureKind::NotTripleEquals),
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

pub fn draw_ligature(
    painter: &egui::Painter,
    kind: LigatureKind,
    x_start: f32,
    y_mid: f32,
    cw: f32,
    stroke: Stroke,
) {
    let cols = kind.char_len() as f32;
    let x_end = x_start + cols * cw;
    let bar_gap = (cw * 0.17).clamp(2.0, 3.2);

    // Aerodynamic, sharp, modern arrowhead dimensions matching Fira Code / JetBrains Mono
    let thin_head_dx = (cw * 0.50).clamp(4.0, 8.5);
    let thin_head_dy = (cw * 0.26).clamp(2.2, 4.5);

    let fat_head_dx = (cw * 0.54).clamp(4.5, 9.5);
    let fat_head_dy = (cw * 0.32).max(bar_gap + 1.2);

    match kind {
        LigatureKind::ArrowRight
        | LigatureKind::LongArrowRight
        | LigatureKind::VeryLongArrowRight => {
            let tip = pos2(x_end - cw * 0.14, y_mid);
            let start = pos2(x_start + cw * 0.08, y_mid);
            painter.line_segment([start, tip], stroke);
            painter.line_segment([pos2(tip.x - thin_head_dx, y_mid - thin_head_dy), tip], stroke);
            painter.line_segment([pos2(tip.x - thin_head_dx, y_mid + thin_head_dy), tip], stroke);
        }
        LigatureKind::ArrowLeft
        | LigatureKind::LongArrowLeft
        | LigatureKind::VeryLongArrowLeft => {
            let tip = pos2(x_start + cw * 0.14, y_mid);
            let end = pos2(x_end - cw * 0.08, y_mid);
            painter.line_segment([tip, end], stroke);
            painter.line_segment([pos2(tip.x + thin_head_dx, y_mid - thin_head_dy), tip], stroke);
            painter.line_segment([pos2(tip.x + thin_head_dx, y_mid + thin_head_dy), tip], stroke);
        }
        LigatureKind::FatArrowRight
        | LigatureKind::LongFatArrowRight
        | LigatureKind::VeryLongFatArrowRight => {
            let tip = pos2(x_end - cw * 0.14, y_mid);
            // Exact geometric intersection so the parallel bars join seamlessly into the arrowhead wings with zero gap
            let intersect_x = tip.x - bar_gap * (fat_head_dx / fat_head_dy);
            let start_x = x_start + cw * 0.08;
            painter.line_segment(
                [pos2(start_x, y_mid - bar_gap), pos2(intersect_x, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(start_x, y_mid + bar_gap), pos2(intersect_x, y_mid + bar_gap)],
                stroke,
            );
            painter.line_segment([pos2(tip.x - fat_head_dx, y_mid - fat_head_dy), tip], stroke);
            painter.line_segment([pos2(tip.x - fat_head_dx, y_mid + fat_head_dy), tip], stroke);
        }
        LigatureKind::LongFatArrowLeft
        | LigatureKind::VeryLongFatArrowLeft => {
            let tip = pos2(x_start + cw * 0.14, y_mid);
            let intersect_x = tip.x + bar_gap * (fat_head_dx / fat_head_dy);
            let end_x = x_end - cw * 0.08;
            painter.line_segment(
                [pos2(intersect_x, y_mid - bar_gap), pos2(end_x, y_mid - bar_gap)],
                stroke,
            );
            painter.line_segment(
                [pos2(intersect_x, y_mid + bar_gap), pos2(end_x, y_mid + bar_gap)],
                stroke,
            );
            painter.line_segment([pos2(tip.x + fat_head_dx, y_mid - fat_head_dy), tip], stroke);
            painter.line_segment([pos2(tip.x + fat_head_dx, y_mid + fat_head_dy), tip], stroke);
        }
        LigatureKind::TripleEquals => {
            let x_left = x_start + cw * 0.08;
            let x_right = x_end - cw * 0.08;
            let sep = (cw * 0.25).clamp(2.8, 4.2);
            painter.line_segment([pos2(x_left, y_mid - sep), pos2(x_right, y_mid - sep)], stroke);
            painter.line_segment([pos2(x_left, y_mid), pos2(x_right, y_mid)], stroke);
            painter.line_segment([pos2(x_left, y_mid + sep), pos2(x_right, y_mid + sep)], stroke);
        }
        LigatureKind::NotTripleEquals => {
            let x_left = x_start + cw * 0.08;
            let x_right = x_end - cw * 0.08;
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
            let x_left = x_start + cw * 0.08;
            let x_right = x_end - cw * 0.08;
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
            let x_left = x_start + cw * 0.08;
            let x_right = x_end - cw * 0.08;
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

pub fn render_line_with_ligatures(
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
pub mod tests {
    use super::*;
    use std::assert_eq;

    #[test]
    pub fn test_detect_ligatures() {
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
        assert_eq!(detect_ligature(&chars, ne_pos), Some(LigatureKind::NotTripleEquals));

        // find the standalone !=
        let standalone_ne = chars.windows(2).enumerate().find(|&(idx, w)| w == ['!', '='] && chars.get(idx + 2) != Some(&'=')).unwrap().0;
        assert_eq!(detect_ligature(&chars, standalone_ne), Some(LigatureKind::NotEquals));

        // find ==
        let eq_pos = chars.windows(2).enumerate().find(|&(idx, w)| w == ['=', '='] && (idx == 0 || chars[idx-1] != '=') && chars.get(idx+2) != Some(&'=')).unwrap().0;
        assert_eq!(detect_ligature(&chars, eq_pos), Some(LigatureKind::DoubleEquals));

        // Test multi-char arrows ==>, ===>, -->, --->
        let fat_chars: Vec<char> = "==> ===> --> --->".chars().collect();
        assert_eq!(detect_ligature(&fat_chars, 0), Some(LigatureKind::LongFatArrowRight));
        assert_eq!(detect_ligature(&fat_chars, 4), Some(LigatureKind::VeryLongFatArrowRight));
        assert_eq!(detect_ligature(&fat_chars, 9), Some(LigatureKind::LongArrowRight));
        assert_eq!(detect_ligature(&fat_chars, 13), Some(LigatureKind::VeryLongArrowRight));
    }
}
