//! Dedicated bottom command & status bar dock.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub fn render_bottom_dock(
    ui: &egui::Ui,
    painter: &egui::Painter,
    dock_rect: Rect,
    _content_left_margin: f32,
    in_command: bool,
    cmd_text: &str,
    status_msg: &str,
    status_time: f64,
    now: f64,
    cursor_row: usize,
    cursor_col: usize,
    total_words: usize,
    mode_badge: Option<&str>,
    search_prompt: Option<(&str, &str, usize)>,
    accent: Color32,
    muted: Color32,
) {
    // Solid background dock with 5px rounded bottom corners matching window
    painter.rect(
        dock_rect,
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: 5,
            se: 5,
        },
        Color32::from_rgb(10, 10, 13),
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [dock_rect.left_top(), dock_rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(26, 26, 32)),
    );

    let cmd_y = dock_rect.min.y + 9.0;
    let cmd_x = dock_rect.min.x + 24.0;
    let mut text_x = cmd_x;

    if in_command {
        // [:CMD] badge
        let badge_rect = Rect::from_min_size(pos2(cmd_x, cmd_y - 2.0), vec2(44.0, 20.0));
        painter.rect_filled(badge_rect, 3.0, accent);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            ":CMD",
            FontId::monospace(11.0),
            Color32::BLACK,
        );

        painter.text(
            pos2(cmd_x + 52.0, cmd_y),
            Align2::LEFT_TOP,
            format!("{}_", cmd_text),
            FontId::monospace(14.0),
            Color32::WHITE,
        );
    } else if let Some((symbol, query, match_count)) = search_prompt {
        // [SEARCH] badge
        let badge_label = if symbol == "?" { "? SEARCH" } else { "/ SEARCH" };
        let badge_rect = Rect::from_min_size(pos2(cmd_x, cmd_y - 2.0), vec2(64.0, 20.0));
        painter.rect_filled(badge_rect, 3.0, accent);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge_label,
            FontId::monospace(10.0),
            Color32::BLACK,
        );

        let query_display = format!("{}_", query);
        painter.text(
            pos2(cmd_x + 72.0, cmd_y),
            Align2::LEFT_TOP,
            query_display,
            FontId::monospace(14.0),
            Color32::WHITE,
        );

        if !query.is_empty() {
            let count_info = if match_count == 0 {
                "(no matches)".to_string()
            } else {
                format!("({} matches)", match_count)
            };
            let query_w = (query.len() + 2) as f32 * 8.5;
            painter.text(
                pos2(cmd_x + 75.0 + query_w, cmd_y + 2.0),
                Align2::LEFT_TOP,
                count_info,
                FontId::monospace(11.0),
                muted,
            );
        }
    } else {
        if let Some(badge) = mode_badge {
            let badge_w = 10.0 + badge.len() as f32 * 6.5;
            let badge_rect = Rect::from_min_size(pos2(cmd_x, cmd_y - 0.5), vec2(badge_w, 16.0));
            let pill_bg = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 18);
            let pill_border = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 45);
            painter.rect(
                badge_rect,
                3.0,
                pill_bg,
                Stroke::new(1.0, pill_border),
                egui::StrokeKind::Inside,
            );
            painter.text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                badge,
                FontId::monospace(9.5),
                Color32::from_gray(190),
            );
            text_x += badge_w + 10.0;
        }

        if !status_msg.is_empty() && (now - status_time) < 3.0 {
            painter.text(
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
        Color32::from_gray(75)
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

    // Right side stats: Line, Col, word count (padded before resize knob)
    let stats = if dock_rect.width() > 620.0 {
        format!("Ln {}, Col {}  ·  {} words", cursor_row + 1, cursor_col + 1, total_words)
    } else {
        format!("Ln {}, Col {}", cursor_row + 1, cursor_col + 1)
    };

    let stats_galley = painter.layout_no_wrap(stats, FontId::monospace(12.0), Color32::from_gray(120));
    let stats_pos = pos2(dock_rect.max.x - 34.0, cmd_y);
    painter.galley(
        pos2(stats_pos.x - stats_galley.size().x, stats_pos.y),
        stats_galley,
        Color32::from_gray(120),
    );
}
