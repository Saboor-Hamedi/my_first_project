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
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Keyboard-driven navigation reference",
        FontId::monospace(11.5),
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
    let row_h = 21.0;
    let badge_w = 110.0;
    let group_gap = 6.0;
    let header_h = 14.0;
    let item_gap = 2.0;

    let mut cur_y = p_origin.y + 44.0;

    for (g_label, items) in groups.iter() {
        // Group header line
        painter.line_segment(
            [pos2(p_origin.x, cur_y + header_h * 0.5), pos2(p_origin.x + 28.0, cur_y + header_h * 0.5)],
            Stroke::new(1.0, Color32::from_gray(40)),
        );
        painter.text(
            pos2(p_origin.x + 34.0, cur_y + header_h * 0.5),
            Align2::LEFT_CENTER,
            *g_label,
            FontId::monospace(10.0),
            Color32::from_gray(90),
        );
        let label_end_x = p_origin.x + 34.0 + g_label.len() as f32 * 6.2 + 8.0;
        painter.line_segment(
            [pos2(label_end_x, cur_y + header_h * 0.5), pos2(p_origin.x + row_w, cur_y + header_h * 0.5)],
            Stroke::new(1.0, Color32::from_gray(40)),
        );
        cur_y += header_h + 4.0;

        for (key, desc) in items.iter() {
            let row_rect = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(row_w, row_h));
            let hovered = ui.rect_contains_pointer(row_rect);

            // Row background
            painter.rect(
                row_rect,
                4.0,
                if hovered { Color32::from_rgb(20, 22, 30) } else { Color32::from_rgb(13, 14, 18) },
                Stroke::new(1.0, if hovered { Color32::from_rgb(40, 44, 56) } else { Color32::from_rgb(22, 24, 30) }),
                egui::StrokeKind::Inside,
            );

            // Keycap badge — 3D-ish: face + shadow strip at bottom
            let badge_rect = Rect::from_min_size(
                pos2(row_rect.min.x + 5.0, row_rect.min.y + 3.0),
                vec2(badge_w, row_h - 6.0),
            );
            // Shadow strip (bottom 2px darker)
            let shadow_strip = Rect::from_min_size(
                pos2(badge_rect.min.x, badge_rect.max.y - 3.0),
                vec2(badge_rect.width(), 3.0),
            );
            painter.rect_filled(
                badge_rect,
                3.0,
                Color32::from_rgb(26, 28, 38),
            );
            painter.rect_filled(
                shadow_strip,
                egui::CornerRadius { nw: 0, ne: 0, sw: 3, se: 3 },
                Color32::from_rgb(14, 15, 22),
            );
            painter.rect(
                badge_rect,
                3.0,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgb(48, 52, 68)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                badge_rect.center() - vec2(0.0, 1.0),
                Align2::CENTER_CENTER,
                *key,
                FontId::monospace(10.5),
                theme.accent,
            );

            // Description
            painter.text(
                pos2(row_rect.min.x + badge_w + 14.0, row_rect.center().y),
                Align2::LEFT_CENTER,
                *desc,
                FontId::monospace(11.5),
                if hovered { Color32::WHITE } else { Color32::from_gray(180) },
            );

            cur_y += row_h + item_gap;
        }

        cur_y += group_gap;
    }

    // Keymap file info footer (pointing to keymap.json in settings directory)
    let keymap_path = crate::vim::VimKeymap::get_keymap_path();
    let footer_rect = Rect::from_min_size(pos2(p_origin.x, cur_y + 4.0), vec2(row_w, 24.0));
    painter.rect(
        footer_rect,
        4.0,
        Color32::from_rgb(16, 17, 22),
        Stroke::new(1.0, Color32::from_rgb(32, 34, 44)),
        egui::StrokeKind::Inside,
    );
    let path_display = keymap_path.to_string_lossy();
    let short_path = if path_display.len() > 42 {
        format!("...{}", &path_display[path_display.len().saturating_sub(40)..])
    } else {
        path_display.to_string()
    };
    painter.text(
        pos2(footer_rect.min.x + 8.0, footer_rect.center().y),
        Align2::LEFT_CENTER,
        format!("⚙ Config: {}", short_path),
        FontId::monospace(10.5),
        theme.muted,
    );
    let open_btn = Rect::from_min_size(pos2(footer_rect.max.x - 94.0, footer_rect.min.y + 2.0), vec2(90.0, 20.0));
    let open_hover = ui.rect_contains_pointer(open_btn);
    painter.rect_filled(
        open_btn,
        3.0,
        if open_hover { Color32::from_rgb(34, 38, 48) } else { Color32::from_rgb(22, 24, 32) },
    );
    painter.text(
        open_btn.center(),
        Align2::CENTER_CENTER,
        "Open Folder",
        FontId::monospace(10.5),
        if open_hover { theme.accent } else { Color32::from_gray(180) },
    );
    if open_hover && ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(parent) = keymap_path.parent() {
            let _ = std::process::Command::new("explorer").arg(parent).spawn();
        }
    }
}
