//! Keyboard shortcuts reference table tab.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, FontId, Pos2, Rect, Stroke};

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
    let row_h = 24.0;
    let badge_w = 110.0;
    let group_gap = 6.0;
    let header_h = 14.0;
    let item_gap = 3.0;

    let mut cur_y = p_origin.y + 44.0;

    for (g_label, items) in groups.iter() {
        // Group header line
        painter.line_segment(
            [pos2(p_origin.x, cur_y + header_h * 0.5), pos2(p_origin.x + 28.0, cur_y + header_h * 0.5)],
            Stroke::new(1.0, theme.border()),
        );
        painter.text(
            pos2(p_origin.x + 34.0, cur_y + header_h * 0.5),
            Align2::LEFT_CENTER,
            *g_label,
            FontId::proportional(11.0),
            theme.muted,
        );
        let label_end_x = p_origin.x + 34.0 + g_label.len() as f32 * 6.8 + 8.0;
        painter.line_segment(
            [pos2(label_end_x, cur_y + header_h * 0.5), pos2(p_origin.x + row_w, cur_y + header_h * 0.5)],
            Stroke::new(1.0, theme.border()),
        );
        cur_y += header_h + 4.0;

        for (key, desc) in items.iter() {
            let row_rect = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(row_w, row_h));
            let hovered = ui.rect_contains_pointer(row_rect);

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
            painter.rect(
                row_rect,
                5.0,
                row_bg,
                row_stroke,
                egui::StrokeKind::Inside,
            );

            // Keycap badge — crisp rounded tag
            let badge_rect = Rect::from_min_size(
                pos2(row_rect.min.x + 5.0, row_rect.min.y + 3.0),
                vec2(badge_w, row_h - 6.0),
            );
            let badge_bg = if hovered {
                theme.surface().lerp_to_gamma(theme.accent, 0.16)
            } else {
                theme.bg
            };
            painter.rect(
                badge_rect,
                4.0,
                badge_bg,
                Stroke::new(1.0, if hovered { theme.accent } else { theme.border() }),
                egui::StrokeKind::Inside,
            );
            painter.text(
                badge_rect.center() - vec2(0.0, 0.5),
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
                FontId::proportional(12.5),
                if hovered { theme.text } else { theme.muted },
            );

            cur_y += row_h + item_gap;
        }

        cur_y += group_gap;
    }

    // Keymap file info footer (pointing to keymap.json in settings directory)
    let keymap_path = crate::vim::VimKeymap::get_keymap_path();
    let footer_rect = Rect::from_min_size(pos2(p_origin.x, cur_y + 4.0), vec2(row_w, 28.0));
    painter.rect(
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
    painter.text(
        pos2(footer_rect.min.x + 10.0, footer_rect.center().y),
        Align2::LEFT_CENTER,
        format!("⚙ Config: {}", short_path),
        FontId::proportional(11.5),
        theme.muted,
    );
    let open_btn = Rect::from_min_size(pos2(footer_rect.max.x - 96.0, footer_rect.min.y + 4.0), vec2(90.0, 20.0));
    let open_hover = ui.rect_contains_pointer(open_btn);
    let btn_bg = if open_hover {
        theme.surface().lerp_to_gamma(theme.accent, 0.15)
    } else {
        theme.bg
    };
    painter.rect(
        open_btn,
        4.0,
        btn_bg,
        Stroke::new(1.0, if open_hover { theme.accent } else { theme.border() }),
        egui::StrokeKind::Inside,
    );
    painter.text(
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
