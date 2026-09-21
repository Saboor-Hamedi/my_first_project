//! Interactive modals for fuzzy search and document renaming.

use crate::fuzzy::{SearchItem, SearchResultKind};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub struct SearchModalAction {
    pub selected_item: Option<SearchItem>,
    pub should_close: bool,
}

pub fn render_search_modal(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    query: &mut String,
    results: &[SearchItem],
    selected_idx: &mut usize,
    accent: Color32,
    just_opened: bool,
) -> SearchModalAction {
    // Dimmed background
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(175));

    let modal_w = 580.0;
    let modal_h = 390.0;
    let modal_rect = Rect::from_center_size(bounds.center(), vec2(modal_w, modal_h));

    // Modern obsidian container with soft, subtle 1px charcoal border
    painter.rect(
        modal_rect,
        5.0,
        Color32::from_rgb(16, 17, 21),
        Stroke::new(1.0, Color32::from_rgb(38, 41, 50)),
        egui::StrokeKind::Inside,
    );

    let m_origin = modal_rect.min + vec2(22.0, 20.0);

    // Search Input Bar
    let input_rect = Rect::from_min_size(m_origin, vec2(modal_w - 44.0, 36.0));
    painter.rect(
        input_rect,
        5.0,
        Color32::from_rgb(24, 26, 32),
        Stroke::new(1.0, Color32::from_rgb(46, 50, 62)),
        egui::StrokeKind::Inside,
    );

    // Search icon
    painter.text(
        input_rect.min + vec2(12.0, 10.0),
        Align2::LEFT_TOP,
        "🔍",
        FontId::monospace(13.0),
        Color32::from_gray(140),
    );

    // Interactive TextEdit with auto-focus!
    let edit_rect = Rect::from_min_max(
        input_rect.min + vec2(34.0, 4.0),
        input_rect.max - vec2(10.0, 4.0),
    );
    let response = ui.put(
        edit_rect,
        egui::TextEdit::singleline(query)
            .font(FontId::monospace(14.0))
            .text_color(Color32::WHITE)
            .hint_text("Search notes, flashcards, decisions...")
            .frame(false),
    );

    // Automatically focus the input field as soon as Ctrl+P is hit!
    if just_opened || !response.has_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        response.request_focus();
    }

    let mut action = SearchModalAction {
        selected_item: None,
        should_close: false,
    };

    // Keyboard navigation inside search
    let (up, down, enter, esc) = ui.input(|i| (
        i.key_pressed(egui::Key::ArrowUp),
        i.key_pressed(egui::Key::ArrowDown),
        i.key_pressed(egui::Key::Enter),
        i.key_pressed(egui::Key::Escape),
    ));

    if up && *selected_idx > 0 {
        *selected_idx -= 1;
    }
    if down && !results.is_empty() && *selected_idx + 1 < results.len() {
        *selected_idx += 1;
    }
    if enter {
        if let Some(item) = results.get(*selected_idx) {
            action.selected_item = Some(item.clone());
        }
        action.should_close = true;
    }
    if esc {
        action.should_close = true;
    }

    // Results list
    let results_y = input_rect.max.y + 16.0;
    if results.is_empty() {
        painter.text(
            pos2(m_origin.x + 8.0, results_y + 14.0),
            Align2::LEFT_TOP,
            "No matches found. Type a note title or card prompt.",
            FontId::monospace(13.0),
            Color32::from_gray(110),
        );
    } else {
        for (i, item) in results.iter().take(7).enumerate() {
            let item_rect = Rect::from_min_size(
                pos2(m_origin.x, results_y + i as f32 * 38.0),
                vec2(modal_w - 44.0, 34.0),
            );
            let is_selected = i == *selected_idx;
            let is_hovered = ui.rect_contains_pointer(item_rect);

            if is_selected || is_hovered {
                painter.rect(
                    item_rect,
                    6.0,
                    if is_selected {
                        Color32::from_rgb(28, 33, 44)
                    } else {
                        Color32::from_rgb(22, 24, 30)
                    },
                    Stroke::NONE,
                    egui::StrokeKind::Inside,
                );
                if is_selected {
                    let bar_rect = Rect::from_min_size(
                        item_rect.left_top() + vec2(0.0, 4.0),
                        vec2(3.0, item_rect.height() - 8.0),
                    );
                    painter.rect_filled(bar_rect, 2.0, accent);
                }
            }

            if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                *selected_idx = i;
                action.selected_item = Some(item.clone());
                action.should_close = true;
            }

            // Type badge pill
            let (badge_text, badge_bg, badge_fg) = match item.kind {
                SearchResultKind::Document => ("DOC", Color32::from_rgb(18, 42, 38), Color32::from_rgb(100, 220, 180)),
            };

            let badge_rect = Rect::from_min_size(item_rect.min + vec2(10.0, 7.0), vec2(52.0, 20.0));
            painter.rect_filled(badge_rect, 4.0, badge_bg);
            painter.text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                badge_text,
                FontId::monospace(10.5),
                badge_fg,
            );

            // Title
            let title_display = if item.title.len() > 36 {
                format!("{}...", &item.title[..36])
            } else {
                item.title.clone()
            };
            painter.text(
                item_rect.min + vec2(72.0, 8.0),
                Align2::LEFT_TOP,
                title_display,
                FontId::monospace(13.0),
                if is_selected {
                    Color32::WHITE
                } else {
                    Color32::from_gray(200)
                },
            );

            // Snippet on right
            if !item.snippet.is_empty() {
                let short_snippet = if item.snippet.len() > 20 {
                    format!("{}...", &item.snippet[..20])
                } else {
                    item.snippet.clone()
                };
                painter.text(
                    pos2(item_rect.max.x - 12.0, item_rect.min.y + 9.0),
                    Align2::RIGHT_TOP,
                    short_snippet,
                    FontId::monospace(11.0),
                    Color32::from_gray(100),
                );
            }
        }
    }

    // Modern Footer
    let footer_y = modal_rect.max.y - 28.0;
    painter.line_segment(
        [
            pos2(modal_rect.min.x + 20.0, footer_y - 6.0),
            pos2(modal_rect.max.x - 20.0, footer_y - 6.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(28, 30, 38)),
    );
    painter.text(
        pos2(modal_rect.min.x + 22.0, footer_y),
        Align2::LEFT_TOP,
        "↑/↓ Navigate  ·  Enter Select  ·  Esc Dismiss",
        FontId::monospace(11.0),
        Color32::from_gray(110),
    );

    action
}

pub struct RenameModalAction {
    pub confirmed_title: Option<String>,
    pub should_close: bool,
}

pub fn render_rename_modal(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    input_text: &mut String,
    accent: Color32,
    just_opened: bool,
) -> RenameModalAction {
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(175));

    let modal_w = 460.0;
    let modal_h = 160.0;
    let modal_rect = Rect::from_center_size(bounds.center(), vec2(modal_w, modal_h));

    painter.rect(
        modal_rect,
        5.0,
        Color32::from_rgb(16, 17, 21),
        Stroke::new(1.0, Color32::from_rgb(38, 41, 50)),
        egui::StrokeKind::Inside,
    );

    let m_origin = modal_rect.min + vec2(24.0, 20.0);
    painter.text(
        m_origin,
        Align2::LEFT_TOP,
        "Rename Document",
        FontId::monospace(15.0),
        Color32::WHITE,
    );
    painter.text(
        m_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Enter a new title for this document",
        FontId::monospace(12.0),
        Color32::from_gray(140),
    );

    let input_rect = Rect::from_min_size(m_origin + vec2(0.0, 48.0), vec2(modal_w - 48.0, 34.0));
    painter.rect(
        input_rect,
        5.0,
        Color32::from_rgb(24, 26, 32),
        Stroke::new(1.0, Color32::from_rgb(46, 50, 62)),
        egui::StrokeKind::Inside,
    );

    let edit_rect = input_rect.shrink2(vec2(10.0, 6.0));
    let response = ui.put(
        edit_rect,
        egui::TextEdit::singleline(input_text)
            .font(FontId::monospace(14.0))
            .text_color(Color32::WHITE)
            .frame(false),
    );

    if just_opened || !response.has_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        response.request_focus();
    }

    let mut action = RenameModalAction {
        confirmed_title: None,
        should_close: false,
    };

    let (enter, esc) = ui.input(|i| (
        i.key_pressed(egui::Key::Enter),
        i.key_pressed(egui::Key::Escape),
    ));

    if enter {
        let trimmed = input_text.trim();
        if !trimmed.is_empty() {
            action.confirmed_title = Some(trimmed.to_string());
        }
        action.should_close = true;
    } else if esc {
        action.should_close = true;
    }

    // Bottom subtle hint line
    painter.text(
        pos2(m_origin.x, modal_rect.max.y - 22.0),
        Align2::LEFT_TOP,
        "Enter to rename  ·  Esc to cancel",
        FontId::monospace(11.0),
        accent,
    );

    action
}

pub struct DeleteModalAction {
    pub confirmed: bool,
    pub should_close: bool,
}

pub fn render_delete_confirm_modal(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    doc_title: &str,
) -> DeleteModalAction {
    // Dimmed background
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(175));

    let modal_w = 420.0;
    let modal_h = 160.0;
    let modal_rect = Rect::from_center_size(bounds.center(), vec2(modal_w, modal_h));

    // Obsidian container with subtle warm charcoal border
    painter.rect(
        modal_rect,
        5.0,
        Color32::from_rgb(17, 18, 22),
        Stroke::new(1.0, Color32::from_rgb(44, 46, 56)),
        egui::StrokeKind::Inside,
    );

    let m_origin = modal_rect.min + vec2(24.0, 20.0);

    // Red warning pill badge
    let badge_rect = Rect::from_min_size(m_origin, vec2(58.0, 20.0));
    painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(48, 20, 24));
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "DELETE",
        FontId::monospace(10.5),
        Color32::from_rgb(255, 100, 110),
    );

    // Modal Title
    painter.text(
        m_origin + vec2(68.0, 1.0),
        Align2::LEFT_TOP,
        "Delete Document?",
        FontId::monospace(15.0),
        Color32::WHITE,
    );

    // Truncate document title cleanly if long to prevent any offset/overflow
    let safe_title = if doc_title.len() > 30 {
        format!("{}...", &doc_title[..30])
    } else if doc_title.trim().is_empty() {
        "Untitled".to_string()
    } else {
        doc_title.to_string()
    };

    painter.text(
        m_origin + vec2(0.0, 32.0),
        Align2::LEFT_TOP,
        format!("Permanently delete \"{}\"?", safe_title),
        FontId::monospace(12.5),
        Color32::from_gray(210),
    );

    painter.text(
        m_origin + vec2(0.0, 52.0),
        Align2::LEFT_TOP,
        "This action cannot be undone.",
        FontId::monospace(11.5),
        Color32::from_gray(130),
    );

    // Buttons: Cancel (Esc) & Delete (Enter)
    let btn_h = 32.0;
    let btn_y = modal_rect.max.y - btn_h - 18.0;

    let cancel_w = 110.0;
    let cancel_rect = Rect::from_min_size(pos2(modal_rect.max.x - 24.0 - cancel_w - 12.0 - 130.0, btn_y), vec2(cancel_w, btn_h));
    let cancel_hover = ui.rect_contains_pointer(cancel_rect);

    let delete_w = 130.0;
    let delete_rect = Rect::from_min_size(pos2(modal_rect.max.x - 24.0 - delete_w, btn_y), vec2(delete_w, btn_h));
    let delete_hover = ui.rect_contains_pointer(delete_rect);

    let (enter, esc) = ui.input(|i| (
        i.key_pressed(egui::Key::Enter),
        i.key_pressed(egui::Key::Escape),
    ));

    let mut action = DeleteModalAction {
        confirmed: false,
        should_close: false,
    };

    // Cancel button
    painter.rect(
        cancel_rect,
        5.0,
        if cancel_hover { Color32::from_rgb(28, 30, 38) } else { Color32::from_rgb(22, 23, 28) },
        Stroke::new(1.0, if cancel_hover { Color32::from_gray(80) } else { Color32::from_gray(50) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        cancel_rect.center(),
        Align2::CENTER_CENTER,
        "Cancel (Esc)",
        FontId::monospace(11.5),
        if cancel_hover { Color32::WHITE } else { Color32::from_gray(180) },
    );

    // Delete button (Destructive red)
    painter.rect(
        delete_rect,
        5.0,
        if delete_hover { Color32::from_rgb(75, 22, 28) } else { Color32::from_rgb(52, 16, 20) },
        Stroke::new(1.0, if delete_hover { Color32::from_rgb(220, 60, 70) } else { Color32::from_rgb(160, 45, 55) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        delete_rect.center(),
        Align2::CENTER_CENTER,
        "Delete (Enter)",
        FontId::monospace(11.5),
        Color32::from_rgb(255, 140, 150),
    );

    if (delete_hover && ui.input(|i| i.pointer.primary_clicked())) || enter {
        action.confirmed = true;
        action.should_close = true;
    } else if (cancel_hover && ui.input(|i| i.pointer.primary_clicked())) || esc {
        action.should_close = true;
    }

    action
}
