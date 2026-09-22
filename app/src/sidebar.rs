//! Sleek floating sidebar with navigation and user notes library.

use core::Note;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub enum SidebarAction {
    SwitchMode(usize),
    LoadNote { id: i64, topic: String, body: String },
    DeleteNote(i64),
    NewNote,
    OpenSettings,
    ToggleNotesLimit,
}

pub fn render_sidebar(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    active_mode_idx: usize,
    active_note_id: Option<i64>,
    notes: &[Note],
    notes_limit: usize,
    total_notes_count: usize,
    is_dirty: bool,
    accent: Color32,
    text_color: Color32,
    muted_color: Color32,
) -> Option<SidebarAction> {
    let gap = 14.0;
    let sidebar_w = 230.0;
    let sb_top = bounds.min.y + 12.0;
    let sb_bottom = bounds.max.y - 36.0 - 8.0;
    let sb_rect = Rect::from_min_max(
        pos2(bounds.min.x + gap, sb_top),
        pos2(bounds.min.x + gap + sidebar_w, sb_bottom),
    );

    // Floating panel background with subtle border (5px round)
    painter.rect(
        sb_rect,
        5.0,
        Color32::from_rgb(12, 12, 14),
        Stroke::new(1.0, Color32::from_rgb(34, 34, 40)),
        egui::StrokeKind::Inside,
    );

    let sb_origin = sb_rect.min + vec2(16.0, 18.0);
    painter.text(
        sb_origin,
        Align2::LEFT_TOP,
        "MINDFORGE",
        FontId::monospace(15.0),
        accent,
    );
    painter.text(
        sb_origin + vec2(0.0, 20.0),
        Align2::LEFT_TOP,
        "Ctrl+B to toggle",
        FontId::monospace(11.0),
        Color32::from_gray(100),
    );

    let mut action = None;

    // Navigation items: Strictly app modes (Notes & Stats)
    let nav_items = [
        ("📝 Notes", 0),
        ("📊 Stats", 1),
    ];

    for (i, (label, mode_idx)) in nav_items.iter().enumerate() {
        let btn_rect = Rect::from_min_size(
            sb_origin + vec2(0.0, 48.0 + i as f32 * 26.0),
            vec2(sidebar_w - 32.0, 22.0),
        );
        let is_sel = active_mode_idx == *mode_idx;
        let is_hovered = ui.rect_contains_pointer(btn_rect);

        if is_hovered || is_sel {
            painter.rect_filled(
                btn_rect,
                4.0,
                if is_sel {
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35)
                } else {
                    Color32::from_rgb(20, 20, 24)
                },
            );
            if is_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
                action = Some(SidebarAction::SwitchMode(*mode_idx));
            }
        }

        painter.text(
            btn_rect.min + vec2(6.0, 3.0),
            Align2::LEFT_TOP,
            *label,
            FontId::monospace(12.0),
            if is_sel { accent } else { text_color },
        );
    }

    // Documents Header (Only user SQLite notes)
    let docs_y = sb_origin.y + 110.0;
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
        pos2(sb_rect.max.x - 12.0, sb_rect.max.y - 42.0),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(docs_list_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("sidebar_docs_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for note in notes {
                    let is_active = active_note_id == Some(note.id);
                    let row_w = ui.available_width();
                    let (rect, resp) = ui.allocate_exact_size(vec2(row_w, 24.0), egui::Sense::click());
                    let hovered = resp.hovered();

                    let del_w = 18.0;
                    let del_rect = Rect::from_min_size(pos2(rect.max.x - del_w - 2.0, rect.min.y + 2.0), vec2(del_w, 20.0));
                    let del_hover = del_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()));

                    if is_active || hovered {
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            if is_active {
                                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35)
                            } else {
                                Color32::from_rgb(22, 22, 26)
                            },
                        );
                        // Left accent border on active note
                        if is_active {
                            let bar = Rect::from_min_size(
                                pos2(rect.min.x, rect.min.y + 4.0),
                                vec2(2.5, rect.height() - 8.0),
                            );
                            ui.painter().rect_filled(bar, 1.5, accent);
                        }
                    }

                    if resp.clicked() && !del_hover {
                        action = Some(SidebarAction::LoadNote {
                            id: note.id,
                            topic: note.topic.clone(),
                            body: note.body.clone(),
                        });
                    }

                    let display_title = if note.topic.len() > 15 {
                        format!("{}...", &note.topic[..15])
                    } else {
                        note.topic.clone()
                    };

                    ui.painter().text(
                        rect.min + vec2(6.0, 4.0),
                        Align2::LEFT_TOP,
                        display_title,
                        FontId::monospace(12.0),
                        if is_active { accent } else { Color32::from_gray(180) },
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

    // Bottom Settings entry in sidebar
    let sb_settings_btn = Rect::from_min_size(
        pos2(sb_rect.min.x + 12.0, sb_rect.max.y - 36.0),
        vec2(sidebar_w - 24.0, 24.0),
    );
    if ui.rect_contains_pointer(sb_settings_btn) {
        painter.rect_filled(sb_settings_btn, 4.0, Color32::from_rgb(25, 25, 30));
        if ui.input(|inp| inp.pointer.primary_clicked()) {
            action = Some(SidebarAction::OpenSettings);
        }
    }
    painter.text(
        sb_settings_btn.min + vec2(6.0, 4.0),
        Align2::LEFT_TOP,
        "⚙ Settings (Ctrl+,)",
        FontId::monospace(11.5),
        muted_color,
    );

    action
}
