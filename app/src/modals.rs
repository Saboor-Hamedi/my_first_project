use crate::fuzzy::SearchItem;
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
    let mut action = SearchModalAction {
        selected_item: None,
        should_close: false,
    };

    // Dimmed translucent backdrop
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(150));

    // Spotlight layout: positioned towards the top (~18% from window top)
    let modal_w = 600.0f32.min(bounds.width() - 32.0);
    let modal_top = bounds.min.y + (bounds.height() * 0.18).clamp(65.0, 140.0);
    let modal_x = bounds.min.x + (bounds.width() - modal_w) * 0.5;

    let is_querying = !query.trim().is_empty();
    let bar_h = 54.0;
    let visible_items = results.len().min(6);

    let modal_h = if !is_querying {
        bar_h
    } else if results.is_empty() {
        bar_h + 1.0 + 52.0 + 32.0 // bar + divider + empty state + footer
    } else {
        bar_h + 1.0 + (visible_items as f32 * 40.0) + 12.0 + 34.0 // bar + divider + items + gap + footer
    };

    let modal_rect = Rect::from_min_size(pos2(modal_x, modal_top), vec2(modal_w, modal_h));

    // Click outside dismisses modal (Mac Spotlight behavior)
    if ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !modal_rect.contains(pos) {
                action.should_close = true;
            }
        }
    }

    // Modern macOS Spotlight container: frosted dark surface with smooth rounded corners
    let glass_bg = Color32::from_rgb(18, 20, 26);
    let glass_border = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 28));
    painter.rect(modal_rect, 10.0, glass_bg, glass_border, egui::StrokeKind::Inside);

    // ── Search Bar Input Row ────────────────────────────────────────────────
    let bar_center_y = modal_rect.min.y + bar_h * 0.5;

    // Search icon (macOS style magnifying glass) vertically centered
    painter.text(
        pos2(modal_rect.min.x + 20.0, bar_center_y),
        Align2::LEFT_CENTER,
        "🔍",
        FontId::monospace(14.0),
        Color32::from_gray(140),
    );

    // Escape shortcut pill keycap on far right of search bar, vertically centered
    let esc_h = 22.0;
    let esc_pill = Rect::from_min_size(
        pos2(modal_rect.max.x - 48.0, bar_center_y - esc_h * 0.5),
        vec2(34.0, esc_h),
    );
    painter.rect(
        esc_pill,
        4.0,
        Color32::from_rgb(26, 28, 36),
        Stroke::new(1.0, Color32::from_rgb(44, 48, 62)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        esc_pill.center(),
        Align2::CENTER_CENTER,
        "esc",
        FontId::monospace(10.5),
        Color32::from_gray(140),
    );

    // Single-line text input vertically aligned with the search icon
    let input_h = 24.0;
    let edit_rect = Rect::from_min_size(
        pos2(modal_rect.min.x + 48.0, bar_center_y - input_h * 0.5),
        vec2(modal_w - 48.0 - 54.0, input_h),
    );
    let response = ui.put(
        edit_rect,
        egui::TextEdit::singleline(query)
            .font(FontId::monospace(14.0))
            .text_color(Color32::WHITE)
            .hint_text("Search notes...")
            .margin(vec2(0.0, 2.0))
            .frame(false),
    );

    // Auto-focus immediately when modal opens
    if just_opened || (!response.has_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape))) {
        response.request_focus();
    }

    // Keyboard navigation
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

    // ── Expanded Results (Only when user types) ──────────────────────────────
    if is_querying {
        // Subtle divider separating search input from results
        let div_y = modal_rect.min.y + bar_h;
        painter.line_segment(
            [pos2(modal_rect.min.x, div_y), pos2(modal_rect.max.x, div_y)],
            Stroke::new(1.0, Color32::from_rgb(32, 35, 45)),
        );

        let results_y = div_y + 8.0;

        if results.is_empty() {
            painter.text(
                pos2(modal_rect.center().x, results_y + 20.0),
                Align2::CENTER_CENTER,
                format!("No matching notes found for \"{}\"", query.trim()),
                FontId::monospace(12.5),
                Color32::from_gray(125),
            );
        } else {
            for (i, item) in results.iter().take(visible_items).enumerate() {
                let item_rect = Rect::from_min_size(
                    pos2(modal_rect.min.x + 8.0, results_y + i as f32 * 40.0),
                    vec2(modal_w - 16.0, 36.0),
                );
                let is_selected = i == *selected_idx;
                let is_hovered = ui.rect_contains_pointer(item_rect);

                if is_selected || is_hovered {
                    let sel_bg = if is_selected {
                        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 32)
                    } else {
                        Color32::from_rgb(24, 27, 35)
                    };
                    painter.rect_filled(item_rect, 6.0, sel_bg);

                    if is_selected {
                        let bar_rect = Rect::from_min_size(
                            item_rect.left_top() + vec2(0.0, 6.0),
                            vec2(3.0, item_rect.height() - 12.0),
                        );
                        painter.rect_filled(bar_rect, 1.5, accent);
                    }
                }

                if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                    *selected_idx = i;
                    action.selected_item = Some(item.clone());
                    action.should_close = true;
                }

                // Type badge pill
                let badge_text = "NOTE";
                let badge_w = 46.0;
                let badge_bg = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35);
                let badge_rect = Rect::from_min_size(item_rect.min + vec2(10.0, 8.0), vec2(badge_w, 20.0));
                painter.rect_filled(badge_rect, 4.0, badge_bg);
                painter.text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    badge_text,
                    FontId::monospace(10.0),
                    accent,
                );

                // Note Title
                let title_display = if item.title.len() > 38 {
                    format!("{}...", &item.title[..38])
                } else {
                    item.title.clone()
                };
                painter.text(
                    item_rect.min + vec2(66.0, 9.0),
                    Align2::LEFT_TOP,
                    title_display,
                    FontId::monospace(13.0),
                    if is_selected { Color32::WHITE } else { Color32::from_gray(210) },
                );

                // Right side: "↵ Open" if selected, snippet if not
                if is_selected {
                    let open_pill = Rect::from_min_size(pos2(item_rect.max.x - 64.0, item_rect.min.y + 8.0), vec2(54.0, 20.0));
                    painter.rect(
                        open_pill,
                        3.0,
                        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 25),
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 80)),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        open_pill.center(),
                        Align2::CENTER_CENTER,
                        "↵ Open",
                        FontId::monospace(10.5),
                        accent,
                    );
                } else if !item.snippet.is_empty() {
                    let short_snippet = if item.snippet.len() > 22 {
                        format!("{}...", &item.snippet[..22])
                    } else {
                        item.snippet.clone()
                    };
                    painter.text(
                        pos2(item_rect.max.x - 14.0, item_rect.min.y + 10.0),
                        Align2::RIGHT_TOP,
                        short_snippet,
                        FontId::monospace(11.0),
                        Color32::from_gray(105),
                    );
                }
            }
        }

        // Minimalist footer bar
        let footer_y = modal_rect.max.y - 28.0;
        painter.line_segment(
            [pos2(modal_rect.min.x + 16.0, footer_y - 4.0), pos2(modal_rect.max.x - 16.0, footer_y - 4.0)],
            Stroke::new(1.0, Color32::from_rgb(28, 30, 40)),
        );
        painter.text(
            pos2(modal_rect.min.x + 18.0, footer_y + 1.0),
            Align2::LEFT_TOP,
            "↑↓ Navigate  ·  ↵ Select  ·  esc Close",
            FontId::monospace(10.5),
            Color32::from_gray(115),
        );
    }

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

    if just_opened {
        response.request_focus();
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), response.id).unwrap_or_default();
        let char_count = input_text.chars().count();
        state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(char_count),
        )));
        state.store(ui.ctx(), response.id);
    } else if !response.has_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
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
    just_opened: bool,
) -> DeleteModalAction {
    // Dimmed background overlay
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(175));

    let modal_w = 460.0;
    let modal_h = 175.0;
    let modal_rect = Rect::from_center_size(bounds.center(), vec2(modal_w, modal_h));

    // Obsidian container with subtle warm charcoal border
    painter.rect(
        modal_rect,
        5.0,
        Color32::from_rgb(17, 18, 22),
        Stroke::new(1.0, Color32::from_rgb(44, 46, 56)),
        egui::StrokeKind::Inside,
    );

    let m_origin = modal_rect.min + vec2(24.0, 22.0);

    // Red warning pill badge
    let badge_rect = Rect::from_min_size(m_origin, vec2(54.0, 20.0));
    painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(48, 20, 24));
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "DELETE",
        FontId::monospace(10.0),
        Color32::from_rgb(255, 100, 110),
    );

    // Modal Title
    painter.text(
        m_origin + vec2(64.0, 1.0),
        Align2::LEFT_TOP,
        "Delete Note",
        FontId::monospace(14.5),
        Color32::WHITE,
    );

    // Truncate note title cleanly if long so it never overflows the container
    let safe_title = if doc_title.trim().is_empty() {
        "Untitled Note".to_string()
    } else if doc_title.chars().count() > 36 {
        let truncated: String = doc_title.chars().take(36).collect();
        format!("{}...", truncated)
    } else {
        doc_title.to_string()
    };

    // Body text - cleanly spaced across dedicated rows
    painter.text(
        m_origin + vec2(0.0, 32.0),
        Align2::LEFT_TOP,
        "Permanently delete this document?",
        FontId::monospace(12.5),
        Color32::from_gray(215),
    );

    painter.text(
        m_origin + vec2(0.0, 52.0),
        Align2::LEFT_TOP,
        format!("\"{}\"", safe_title),
        FontId::monospace(12.0),
        Color32::from_rgb(255, 130, 140),
    );

    painter.text(
        m_origin + vec2(0.0, 72.0),
        Align2::LEFT_TOP,
        "This action cannot be undone.",
        FontId::monospace(11.0),
        Color32::from_gray(120),
    );

    // Buttons: Cancel (Esc) & Delete (Enter)
    let btn_h = 32.0;
    let btn_y = modal_rect.max.y - btn_h - 18.0;

    let delete_w = 125.0;
    let cancel_w = 110.0;
    let delete_rect = Rect::from_min_size(pos2(modal_rect.max.x - 24.0 - delete_w, btn_y), vec2(delete_w, btn_h));
    let cancel_rect = Rect::from_min_size(pos2(modal_rect.max.x - 24.0 - delete_w - 12.0 - cancel_w, btn_y), vec2(cancel_w, btn_h));

    let cancel_hover = ui.rect_contains_pointer(cancel_rect);
    let delete_hover = ui.rect_contains_pointer(delete_rect);

    let (enter, esc) = if just_opened {
        (false, false)
    } else {
        ui.input(|i| (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
        ))
    };

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
