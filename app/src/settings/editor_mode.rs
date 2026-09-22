//! Editor Mode (Hybrid vs Vim) settings tab.

use crate::app::EditorInputMode;
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

pub fn render_editor_mode_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    editor_input_mode: &mut EditorInputMode,
    theme: &Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "TYPING & EDITING ENGINE",
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Select your preferred editing cockpit (or toggle anytime via :vim)",
        FontId::monospace(11.0),
        theme.muted,
    );

    let card_w = panel_rect.width() - 56.0;
    let mut cur_y = p_origin.y + 52.0;

    // ── Card 1: Hybrid Mode ──────────────────────────────────────────
    let is_hybrid = *editor_input_mode == EditorInputMode::Hybrid;
    let hybrid_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 128.0));
    let hybrid_hover = ui.rect_contains_pointer(hybrid_card);

    if hybrid_hover && ui.input(|i| i.pointer.primary_clicked()) {
        *editor_input_mode = EditorInputMode::Hybrid;
        on_save_setting("editor_mode", "hybrid");
    }

    let hybrid_bg = if is_hybrid {
        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 30)
    } else if hybrid_hover {
        Color32::from_rgb(20, 22, 28)
    } else {
        Color32::from_rgb(14, 15, 20)
    };
    painter.rect(
        hybrid_card,
        5.0,
        hybrid_bg,
        Stroke::new(1.0, if is_hybrid { theme.accent } else { Color32::from_rgb(32, 34, 44) }),
        egui::StrokeKind::Inside,
    );

    painter.text(
        hybrid_card.min + vec2(14.0, 11.0),
        Align2::LEFT_TOP,
        "⚡ Hybrid Mode (Modern Power IDE)",
        FontId::monospace(12.5),
        if is_hybrid { theme.accent } else { theme.highlight },
    );
    if is_hybrid {
        painter.text(
            pos2(hybrid_card.max.x - 14.0, hybrid_card.min.y + 11.0),
            Align2::RIGHT_TOP,
            "● ACTIVE",
            FontId::monospace(10.5),
            theme.accent,
        );
    }

    let hybrid_features = [
        ("• Word Jump", "Ctrl+Left/Right to jump; Shift to select"),
        ("• Line Move", "Alt+Up/Down swaps lines in place smoothly"),
        ("• Duplicate", "Ctrl+D duplicates current line or selection"),
        ("• Auto-Pair", "Closes \"\", (), [], {} and wraps selected text"),
        ("• Smart Tab", "Tab indents 4 spaces; Shift+Tab dedents"),
    ];
    for (idx, (label, desc)) in hybrid_features.iter().enumerate() {
        let y = hybrid_card.min.y + 34.0 + idx as f32 * 17.5;
        painter.text(pos2(hybrid_card.min.x + 14.0, y), Align2::LEFT_TOP, *label, FontId::monospace(10.0), theme.accent);
        painter.text(pos2(hybrid_card.min.x + 110.0, y), Align2::LEFT_TOP, *desc, FontId::monospace(9.5), Color32::from_gray(175));
    }

    cur_y += 138.0;

    // ── Card 2: Vim Mode ─────────────────────────────────────────────
    let is_vim = *editor_input_mode == EditorInputMode::Vim;
    let vim_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 128.0));
    let vim_hover = ui.rect_contains_pointer(vim_card);

    if vim_hover && ui.input(|i| i.pointer.primary_clicked()) {
        *editor_input_mode = EditorInputMode::Vim;
        on_save_setting("editor_mode", "vim");
    }

    let vim_bg = if is_vim {
        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 30)
    } else if vim_hover {
        Color32::from_rgb(20, 22, 28)
    } else {
        Color32::from_rgb(14, 15, 20)
    };
    painter.rect(
        vim_card,
        5.0,
        vim_bg,
        Stroke::new(1.0, if is_vim { theme.accent } else { Color32::from_rgb(32, 34, 44) }),
        egui::StrokeKind::Inside,
    );

    painter.text(
        vim_card.min + vec2(14.0, 11.0),
        Align2::LEFT_TOP,
        "⚔ Vim Mode (Modal Keyboard Engine)",
        FontId::monospace(12.5),
        if is_vim { theme.accent } else { theme.highlight },
    );
    if is_vim {
        painter.text(
            pos2(vim_card.max.x - 14.0, vim_card.min.y + 11.0),
            Align2::RIGHT_TOP,
            "● ACTIVE",
            FontId::monospace(10.5),
            theme.accent,
        );
    }

    let vim_features = [
        ("• Motions", "h, j, k, l, w, b, 0, $, gg, G & counts (3j, 5w)"),
        ("• Operators", "dd, yy, cc, dw, x, u, Ctrl+R & text objects"),
        ("• Search", "/ and ? in-buffer search with live n/N repeat"),
        ("• Modes", "Normal, Insert (i, a, o, A, I), Visual (v, V)"),
        ("• Caret", "Dynamic Block in Normal and Beam in Insert"),
    ];
    for (idx, (label, desc)) in vim_features.iter().enumerate() {
        let y = vim_card.min.y + 34.0 + idx as f32 * 17.5;
        painter.text(pos2(vim_card.min.x + 14.0, y), Align2::LEFT_TOP, *label, FontId::monospace(10.0), theme.accent);
        painter.text(pos2(vim_card.min.x + 110.0, y), Align2::LEFT_TOP, *desc, FontId::monospace(9.5), Color32::from_gray(175));
    }
}
