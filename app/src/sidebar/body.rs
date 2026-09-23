//! Sidebar body component: Documents explorer and SQLite notes library.

use core::Note;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect};
use crate::sidebar::SidebarAction;

/// Renders the scrollable documents list, active selection indicators, and note management controls.
pub fn render_sidebar_body(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    sb_rect: Rect,
    sb_origin: egui::Pos2,
    active_note_id: Option<i64>,
    notes: &[Note],
    notes_limit: usize,
    total_notes_count: usize,
    is_dirty: bool,
    accent: Color32,
    muted_color: Color32,
    sidebar_selected_idx: usize,
    sidebar_focused: bool,
) -> Option<SidebarAction> {
    let mut action = None;

    // Documents Header
    let docs_y = sb_origin.y + 80.0;
    let header_label = if total_notes_count > notes.len() {
        format!("DOCUMENTS ({}/{})", notes.len(), total_notes_count)
    } else {
        format!("DOCUMENTS ({})", notes.len())
    };

    painter.text(
        pos2(sb_origin.x, docs_y),
        Align2::LEFT_TOP,
        header_label,
        FontId::monospace(11.5),
        muted_color,
    );

    // "+" New Note button
    let new_btn = Rect::from_min_size(
        pos2(sb_rect.max.x - 38.0, docs_y - 2.0),
        vec2(22.0, 20.0),
    );
    if ui.rect_contains_pointer(new_btn) {
        painter.rect_filled(new_btn, 3.0, Color32::from_rgb(32, 32, 38));
        if ui.input(|inp| inp.pointer.primary_clicked()) {
            action = Some(SidebarAction::NewNote);
        }
    }
    painter.text(new_btn.center(), Align2::CENTER_CENTER, "+", FontId::monospace(14.0), accent);

    // Scrollable Documents Area (User SQLite notes)
    let docs_list_rect = Rect::from_min_max(
        pos2(sb_origin.x, docs_y + 24.0),
        pos2(sb_rect.max.x - 12.0, sb_rect.max.y - 48.0),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(docs_list_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("sidebar_docs_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (idx, note) in notes.iter().enumerate() {
                    let is_active = active_note_id == Some(note.id);
                    let is_selected = idx == sidebar_selected_idx;
                    let row_w = ui.available_width();
                    let (rect, resp) = ui.allocate_exact_size(vec2(row_w, 24.0), egui::Sense::click());
                    let hovered = resp.hovered();

                    if is_selected && sidebar_focused {
                        resp.scroll_to_me(Some(egui::Align::Center));
                    }

                    let del_w = 18.0;
                    let del_rect = Rect::from_min_size(pos2(rect.max.x - del_w - 2.0, rect.min.y + 2.0), vec2(del_w, 20.0));
                    let del_hover = del_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()));

                    if is_active || (is_selected && sidebar_focused) || hovered {
                        let bg_color = if is_selected && sidebar_focused && is_active {
                            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 50)
                        } else if is_selected && sidebar_focused {
                            Color32::from_rgb(26, 28, 38)
                        } else if is_active {
                            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35)
                        } else {
                            Color32::from_rgb(22, 22, 26)
                        };
                        ui.painter().rect_filled(rect, 4.0, bg_color);

                        // Left accent bar on active or focused/selected note
                        if is_active || (is_selected && sidebar_focused) {
                            let bar_w = if is_selected && sidebar_focused { 3.0 } else { 2.5 };
                            let bar = Rect::from_min_size(
                                pos2(rect.min.x, rect.min.y + 3.0),
                                vec2(bar_w, rect.height() - 6.0),
                            );
                            ui.painter().rect_filled(bar, 1.5, accent);
                        }
                    }

                    if resp.clicked() && !del_hover {
                        action = Some(SidebarAction::LoadNote {
                            id: note.id,
                            topic: note.topic.clone(),
                            body: note.body.clone(),
                            index: idx,
                        });
                    }

                    let display_title = if note.topic.len() > 15 {
                        format!("{}...", &note.topic[..15])
                    } else {
                        note.topic.clone()
                    };

                    let title_color = if is_active {
                        accent
                    } else if is_selected && sidebar_focused {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(180)
                    };

                    ui.painter().text(
                        rect.min + vec2(7.0, 4.0),
                        Align2::LEFT_TOP,
                        display_title,
                        FontId::monospace(12.0),
                        title_color,
                    );

                    // GitHub-style unsaved document indicator ●
                    if is_active && is_dirty {
                        let dot_color = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 180);
                        let dot_x = if hovered { rect.max.x - del_w - 10.0 } else { rect.max.x - 14.0 };
                        ui.painter().text(
                            pos2(dot_x, rect.min.y + 4.0),
                            Align2::CENTER_TOP,
                            "●",
                            FontId::monospace(11.0),
                            dot_color,
                        );
                    }

                    if hovered {
                        if del_hover {
                            ui.painter().rect_filled(del_rect, 3.0, Color32::from_rgb(55, 25, 25));
                            if ui.input(|i| i.pointer.primary_clicked()) {
                                action = Some(SidebarAction::DeleteNote(note.id));
                            }
                        }
                        ui.painter().text(
                            del_rect.center(),
                            Align2::CENTER_CENTER,
                            "×",
                            FontId::monospace(13.0),
                            if del_hover { Color32::from_rgb(250, 100, 100) } else { Color32::from_gray(130) },
                        );
                    }
                }

                // "See more" dropdown button: toggles between 50 and 100 max
                if notes_limit < 100 && total_notes_count > notes.len() {
                    ui.add_space(4.0);
                    let row_w = ui.available_width();
                    let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(row_w, 24.0), egui::Sense::click());
                    let b_hover = btn_resp.hovered();
                    ui.painter().rect_filled(
                        btn_rect,
                        4.0,
                        if b_hover { Color32::from_rgb(28, 32, 38) } else { Color32::from_rgb(18, 19, 24) },
                    );
                    ui.painter().text(
                        btn_rect.center(),
                        Align2::CENTER_CENTER,
                        "▼ See more (100 max)",
                        FontId::monospace(11.0),
                        accent,
                    );
                    if btn_resp.clicked() {
                        action = Some(SidebarAction::ToggleNotesLimit);
                    }
                } else if notes_limit >= 100 {
                    ui.add_space(4.0);
                    let row_w = ui.available_width();
                    let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(row_w, 24.0), egui::Sense::click());
                    let b_hover = btn_resp.hovered();
                    ui.painter().rect_filled(
                        btn_rect,
                        4.0,
                        if b_hover { Color32::from_rgb(28, 32, 38) } else { Color32::from_rgb(18, 19, 24) },
                    );
                    ui.painter().text(
                        btn_rect.center(),
                        Align2::CENTER_CENTER,
                        "▲ Show less (50)",
                        FontId::monospace(11.0),
                        muted_color,
                    );
                    if btn_resp.clicked() {
                        action = Some(SidebarAction::ToggleNotesLimit);
                    }
                }
            });
    });

    action
}
