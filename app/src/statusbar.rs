//! Dedicated bottom command & status bar dock.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub fn render_bottom_dock(
    ui: &egui::Ui,
    painter: &egui::Painter,
    dock_rect: Rect,
    _content_left_margin: f32,
    in_command: bool,
    cmd_text: &str,
    cmd_cur: usize,
    cmd_selection: Option<(usize, usize)>,
    status_msg: &str,
    status_time: f64,
    now: f64,
    cursor_row: usize,
    cursor_col: usize,
    total_words: usize,
    mode_badge: Option<&str>,
    search_prompt: Option<(&str, &str, usize)>,
    theme: &Theme,
) {
    let accent = theme.accent;
    let muted = theme.muted;

    // Detached modern bottom dock card with uniform 5px radius and subtle 1px border derived from theme
    painter.rect(
        dock_rect,
        5.0,
        theme.surface(),
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    let cmd_x = dock_rect.min.x + 14.0;
    let badge_h = 20.0;
    let badge_y = dock_rect.center().y - badge_h * 0.5;
    let cmd_y = dock_rect.center().y - 8.0;
    let mut text_x = cmd_x;

    // Right side stats: Line, Col, word count (padded before resize knob)
    let stats = if dock_rect.width() > 620.0 {
        format!("Ln {}, Col {}  ·  {} words", cursor_row, cursor_col, total_words)
    } else {
        format!("Ln {}, Col {}", cursor_row, cursor_col)
    };
    let stats_galley = painter.layout_no_wrap(stats, FontId::monospace(12.0), theme.muted);
    let stats_pos = pos2(dock_rect.max.x - 34.0, dock_rect.center().y - 7.0);
    let stats_left_x = stats_pos.x - stats_galley.size().x;
    let max_cmd_x = (stats_left_x - 16.0).max(cmd_x + 120.0);
    let cmd_input_left = cmd_x + 58.0;
    let cmd_avail_w = (max_cmd_x - cmd_input_left).max(40.0);

    if in_command {
        // [:CMD] badge
        let badge_rect = Rect::from_min_size(pos2(cmd_x, badge_y), vec2(48.0, badge_h));
        painter.rect_filled(badge_rect, 4.0, accent);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            ":CMD",
            FontId::monospace(10.5),
            if theme.is_light() { Color32::WHITE } else { theme.bg },
        );

        let font = FontId::monospace(14.0);
        let cur_clamped = cmd_cur.min(cmd_text.len());
        let mut valid_cur = cur_clamped;
        while !cmd_text.is_char_boundary(valid_cur) && valid_cur > 0 {
            valid_cur -= 1;
        }
        let before_cur = &cmd_text[..valid_cur];
        let cursor_offset_x = painter.layout_no_wrap(before_cur.to_string(), font.clone(), theme.text).size().x;
        let total_text_w = painter.layout_no_wrap(cmd_text.to_string(), font.clone(), theme.text).size().x;

        // Auto-scroll offset so cursor is always within view and long commands push left
        let scroll_x = if total_text_w > cmd_avail_w {
            let max_scroll = (total_text_w - cmd_avail_w + 24.0).max(0.0);
            (cursor_offset_x - (cmd_avail_w - 24.0)).clamp(0.0, max_scroll)
        } else {
            0.0
        };

        let cmd_clip_rect = Rect::from_min_max(
            pos2(cmd_input_left, dock_rect.min.y),
            pos2(max_cmd_x, dock_rect.max.y),
        );
        let cmd_painter = painter.with_clip_rect(cmd_clip_rect);
        let text_origin = pos2(cmd_input_left - scroll_x, cmd_y);

        // Draw selection background if text is selected
        if let Some((start, end)) = cmd_selection {
            let s_min = start.min(end).min(cmd_text.len());
            let s_max = start.max(end).min(cmd_text.len());
            let mut valid_min = s_min;
            while !cmd_text.is_char_boundary(valid_min) && valid_min > 0 {
                valid_min -= 1;
            }
            let mut valid_max = s_max;
            while !cmd_text.is_char_boundary(valid_max) && valid_max > 0 {
                valid_max -= 1;
            }
            let prefix = &cmd_text[..valid_min];
            let selected_part = &cmd_text[valid_min..valid_max];
            let x_off = cmd_painter.layout_no_wrap(prefix.to_string(), font.clone(), theme.text).size().x;
            let sel_w = cmd_painter.layout_no_wrap(selected_part.to_string(), font.clone(), theme.text).size().x;
            cmd_painter.rect_filled(
                Rect::from_min_size(pos2(text_origin.x + x_off, text_origin.y), vec2(sel_w, 18.0)),
                2.0,
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 90),
            );
        }

        // Draw command line text
        let galley = cmd_painter.layout_no_wrap(cmd_text.to_string(), font.clone(), theme.text);
        cmd_painter.galley(text_origin, galley, theme.text);

        // Draw cursor beam at the exact cmd_cur position
        let cursor_x = text_origin.x + cursor_offset_x;
        let blink = ((now * 2.5).sin() > -0.2) as i32 != 0;
        if blink {
            cmd_painter.rect_filled(
                Rect::from_min_size(pos2(cursor_x, text_origin.y), vec2(2.0, 16.0)),
                1.0,
                accent,
            );
        }
    } else if let Some((symbol, query, match_count)) = search_prompt {
        // [SEARCH] badge
        let badge_label = if symbol == "?" { "? SEARCH" } else { "/ SEARCH" };
        let badge_rect = Rect::from_min_size(pos2(cmd_x, badge_y), vec2(68.0, badge_h));
        painter.rect_filled(badge_rect, 4.0, accent);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge_label,
            FontId::monospace(10.0),
            if theme.is_light() { Color32::WHITE } else { theme.bg },
        );

        let search_clip_rect = Rect::from_min_max(
            pos2(cmd_x + 78.0, dock_rect.min.y),
            pos2(max_cmd_x, dock_rect.max.y),
        );
        let search_painter = painter.with_clip_rect(search_clip_rect);
        let query_display = format!("{}_", query);
        search_painter.text(
            pos2(cmd_x + 78.0, cmd_y),
            Align2::LEFT_TOP,
            query_display,
            FontId::monospace(14.0),
            theme.text,
        );

        if !query.is_empty() {
            let count_info = if match_count == 0 {
                "(no matches)".to_string()
            } else {
                format!("({} matches)", match_count)
            };
            let query_w = (query.len() + 2) as f32 * 8.5;
            search_painter.text(
                pos2(cmd_x + 82.0 + query_w, cmd_y + 2.0),
                Align2::LEFT_TOP,
                count_info,
                FontId::monospace(11.0),
                muted,
            );
        }
    } else {
        if let Some(badge) = mode_badge {
            let font = FontId::monospace(10.5);
            let layout = painter.layout_no_wrap(badge.to_string(), font.clone(), theme.highlight);
            let badge_w = (layout.size().x + 20.0).max(64.0);
            let badge_rect = Rect::from_min_size(pos2(cmd_x, badge_y), vec2(badge_w, badge_h));

            let is_insert = badge == "INSERT";
            let is_visual = badge.starts_with("VISUAL");

            let (pill_bg, pill_stroke, text_color) = if is_insert {
                (
                    accent,
                    Stroke::NONE,
                    if theme.is_light() { Color32::WHITE } else { theme.bg },
                )
            } else if is_visual {
                (
                    Color32::from_rgb(224, 108, 117), // coral
                    Stroke::NONE,
                    Color32::WHITE,
                )
            } else if theme.is_light() {
                (
                    theme.surface().lerp_to_gamma(accent, 0.18),
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 120)),
                    accent,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 38),
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 95)),
                    accent,
                )
            };

            painter.rect(
                badge_rect,
                4.0,
                pill_bg,
                pill_stroke,
                egui::StrokeKind::Inside,
            );
            painter.text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                badge,
                font,
                text_color,
            );
            text_x = badge_rect.max.x + 12.0;
        }

        if !status_msg.is_empty() && (now - status_time) < 3.0 {
            let status_clip = Rect::from_min_max(
                pos2(text_x, dock_rect.min.y),
                pos2(max_cmd_x, dock_rect.max.y),
            );
            let status_painter = painter.with_clip_rect(status_clip);
            status_painter.text(
                pos2(text_x, cmd_y),
                Align2::LEFT_TOP,
                status_msg,
                FontId::monospace(12.0),
                muted,
            );
        }
    }

    // Interactive window resize knob in the bottom-right corner
    let knob_size = 28.0;
    let knob_rect = Rect::from_min_max(
        pos2(dock_rect.max.x - knob_size, dock_rect.max.y - knob_size),
        dock_rect.max,
    );
    let is_knob_hovered = ui.rect_contains_pointer(knob_rect);

    if is_knob_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthEast);
    }

    if is_knob_hovered && ui.input(|i| i.pointer.primary_down() || i.pointer.primary_pressed()) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
    }

    let knob_color = if is_knob_hovered {
        accent
    } else {
        theme.muted
    };

    // Tactile diagonal gripper ridges
    for &d in &[5.0, 9.0, 13.0, 17.0] {
        painter.line_segment(
            [
                pos2(dock_rect.max.x - d, dock_rect.max.y - 4.0),
                pos2(dock_rect.max.x - 4.0, dock_rect.max.y - d),
            ],
            Stroke::new(1.4, knob_color),
        );
    }

    // Draw right side stats
    painter.galley(
        pos2(stats_left_x, stats_pos.y),
        stats_galley,
        theme.muted,
    );
}
