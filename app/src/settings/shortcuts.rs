//! Keyboard shortcuts reference table tab.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

pub fn render_shortcuts_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    theme: &Theme,
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "KEYBOARD SHORTCUTS",
        FontId::proportional(15.0),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Keyboard-driven navigation reference",
        FontId::proportional(12.0),
        theme.muted,
    );

    // Groups: (header_label, &[(key, desc)])
    let groups: &[(&str, &[(&str, &str)])] = &[
        ("DOCUMENT", &[
            ("Ctrl + N",      "Create new note"),
            ("Ctrl + S",      "Save active document"),
            ("Ctrl + R",      "Rename document"),
            ("Ctrl + [ / ]",  "Move text left / right (dedent/indent)"),
            ("Ctrl + D",      "Duplicate line below"),
        ]),
        ("NAVIGATION", &[
            ("Ctrl + P",   "Fuzzy search across notes"),
            ("Ctrl + B",   "Toggle notes sidebar"),
            ("Ctrl + ,",   "Open preferences modal"),
            ("Esc",        "Dismiss modal / return to Normal mode"),
        ]),
        ("VIM & SHOWCMD HUD", &[
            ("h / j / k / l", "Home-row cursor navigation"),
            ("ci\" / da(",    "Text object editing (tracked in HUD)"),
            ("/pattern",      "Live buffer search (FIND HUD badge)"),
            (":w / :doc",     "Command palette (CMD HUD badge)"),
        ]),
        ("SYSTEM", &[
            ("Ctrl+Shift+W", "Close window"),
            (":help",        "Command bar reference"),
        ]),
    ];

    let row_w = panel_rect.width() - 56.0;
    let row_h = 28.0;
    let group_gap = 8.0;
    let header_h = 16.0;
    let item_gap = 4.0;

    let content_top = p_origin.y + 44.0;
    let total_items: usize = groups.iter().map(|(_, items)| items.len()).sum();
    let content_h = groups.len() as f32 * (header_h + 4.0 + group_gap)
        + total_items as f32 * (row_h + item_gap)
        + 36.0 // footer
        + 28.0; // bottom margin
    let visible_h = (panel_rect.max.y - content_top).max(0.0);
    let max_scroll = (content_h - visible_h).max(0.0);

    let scroll_id = egui::Id::new("shortcuts_tab_scroll");
    let mut scroll = ui.ctx().data_mut(|d| d.get_temp::<f32>(scroll_id)).unwrap_or(0.0);

    let pointer_over_panel = ui.rect_contains_pointer(Rect::from_min_max(
        pos2(panel_rect.min.x, content_top),
        panel_rect.max,
    ));
    if pointer_over_panel && max_scroll > 0.0 {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        scroll = (scroll - wheel).clamp(0.0, max_scroll);
    }
    ui.ctx().data_mut(|d| d.insert_temp(scroll_id, scroll));

    let clip_rect = Rect::from_min_max(pos2(panel_rect.min.x, content_top), panel_rect.max);
    let list_painter = painter.with_clip_rect(clip_rect);
    let hover_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

    let mut cur_y = content_top - scroll;

    for (g_label, items) in groups.iter() {
        // Group header line
        if cur_y + header_h >= clip_rect.min.y && cur_y <= clip_rect.max.y {
            list_painter.line_segment(
                [pos2(p_origin.x, cur_y + header_h * 0.5), pos2(p_origin.x + 28.0, cur_y + header_h * 0.5)],
                Stroke::new(1.0, theme.border()),
            );
            list_painter.text(
                pos2(p_origin.x + 34.0, cur_y + header_h * 0.5),
                Align2::LEFT_CENTER,
                *g_label,
                FontId::proportional(11.0),
                theme.muted,
            );
            let label_end_x = p_origin.x + 34.0 + g_label.len() as f32 * 6.8 + 8.0;
            list_painter.line_segment(
                [pos2(label_end_x, cur_y + header_h * 0.5), pos2(p_origin.x + row_w, cur_y + header_h * 0.5)],
                Stroke::new(1.0, theme.border()),
            );
        }
        cur_y += header_h + 4.0;

        for (key, desc) in items.iter() {
            let row_rect = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(row_w, row_h));
            let in_view = cur_y + row_h >= clip_rect.min.y && cur_y <= clip_rect.max.y;
            let hovered = in_view && clip_rect.contains(hover_pos) && ui.rect_contains_pointer(row_rect);

            if in_view {
                // Row background
                let row_bg = if hovered {
                    theme.surface().lerp_to_gamma(theme.accent, 0.08)
                } else {
                    theme.surface()
                };
                let row_stroke = Stroke::new(
                    1.0,
                    if hovered { theme.accent } else { theme.border() },
                );
                list_painter.rect(
                    row_rect,
                    6.0,
                    row_bg,
                    row_stroke,
                    egui::StrokeKind::Inside,
                );

                // Explanation on the Left
                list_painter.text(
                    pos2(row_rect.min.x + 16.0, row_rect.center().y),
                    Align2::LEFT_CENTER,
                    *desc,
                    FontId::proportional(13.0),
                    if hovered { theme.text } else { theme.muted },
                );

                // Keycap badge on the Right
                let font_key = FontId::monospace(11.0);
                let text_layout = list_painter.layout_no_wrap(key.to_string(), font_key.clone(), theme.accent);
                let badge_w = (text_layout.size().x + 20.0).max(75.0);
                let badge_h = 22.0;
                let badge_rect = Rect::from_min_size(
                    pos2(row_rect.max.x - badge_w - 8.0, row_rect.center().y - badge_h * 0.5),
                    vec2(badge_w, badge_h),
                );

                let badge_bg = if hovered {
                    if theme.is_light() {
                        Color32::from_rgb(255, 255, 255)
                    } else {
                        theme.surface().lerp_to_gamma(theme.accent, 0.16)
                    }
                } else {
                    if theme.is_light() {
                        Color32::from_rgb(255, 255, 255)
                    } else {
                        theme.bg
                    }
                };
                list_painter.rect(
                    badge_rect,
                    4.0,
                    badge_bg,
                    Stroke::new(1.0, if hovered { theme.accent } else { theme.border() }),
                    egui::StrokeKind::Inside,
                );
                list_painter.text(
                    badge_rect.center() - vec2(0.0, 0.5),
                    Align2::CENTER_CENTER,
                    *key,
                    font_key,
                    theme.accent,
                );
            }

            cur_y += row_h + item_gap;
        }

        cur_y += group_gap;
    }

    // Keymap file info footer (pointing to keymap.json in settings directory)
    let keymap_path = crate::vim::VimKeymap::get_keymap_path();
    let footer_rect = Rect::from_min_size(pos2(p_origin.x, cur_y + 4.0), vec2(row_w, 28.0));
    let footer_in_view = footer_rect.max.y >= clip_rect.min.y && footer_rect.min.y <= clip_rect.max.y;

    if footer_in_view {
        list_painter.rect(
            footer_rect,
            5.0,
            theme.surface(),
            Stroke::new(1.0, theme.border()),
            egui::StrokeKind::Inside,
        );
        let path_display = keymap_path.to_string_lossy();
        let short_path = if path_display.len() > 42 {
            format!("...{}", &path_display[path_display.len().saturating_sub(40)..])
        } else {
            path_display.to_string()
        };
        list_painter.text(
            pos2(footer_rect.min.x + 10.0, footer_rect.center().y),
            Align2::LEFT_CENTER,
            format!("⚙ Config: {}", short_path),
            FontId::proportional(11.5),
            theme.muted,
        );
        let open_btn = Rect::from_min_size(pos2(footer_rect.max.x - 96.0, footer_rect.min.y + 4.0), vec2(90.0, 20.0));
        let open_hover = clip_rect.contains(hover_pos) && ui.rect_contains_pointer(open_btn);
        let btn_bg = if open_hover {
            theme.surface().lerp_to_gamma(theme.accent, 0.15)
        } else {
            theme.bg
        };
        list_painter.rect(
            open_btn,
            4.0,
            btn_bg,
            Stroke::new(1.0, if open_hover { theme.accent } else { theme.border() }),
            egui::StrokeKind::Inside,
        );
        list_painter.text(
            open_btn.center(),
            Align2::CENTER_CENTER,
            "Open Folder",
            FontId::proportional(11.0),
            if open_hover { theme.accent } else { theme.text },
        );
        if open_hover && ui.input(|i| i.pointer.primary_clicked()) {
            if let Some(parent) = keymap_path.parent() {
                let _ = std::process::Command::new("explorer").arg(parent).spawn();
            }
        }
    }

    // Custom scrollbar
    if max_scroll > 0.0 {
        let track_x = panel_rect.max.x - 8.0;
        let track = Rect::from_min_max(pos2(track_x, content_top), pos2(track_x + 3.0, panel_rect.max.y - 8.0));
        let track_color = if theme.is_light() {
            Color32::from_rgba_unmultiplied(0, 0, 0, 15)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 15)
        };
        painter.rect_filled(track, 1.5, track_color);

        let thumb_h = (visible_h * visible_h / content_h).clamp(24.0, visible_h);
        let thumb_y = content_top + (scroll / max_scroll) * (visible_h - thumb_h - 8.0);
        let thumb = Rect::from_min_size(pos2(track_x, thumb_y), vec2(3.0, thumb_h));
        let thumb_color = if theme.is_light() {
            Color32::from_rgba_unmultiplied(0, 0, 0, 60)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 60)
        };
        painter.rect_filled(thumb, 1.5, thumb_color);
    }
}
