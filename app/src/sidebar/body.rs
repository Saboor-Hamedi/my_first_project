//! Sidebar body component: Documents explorer and SQLite notes library.

use core::Note;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect};
use crate::sidebar::SidebarAction;

use crate::theme::Theme;

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
    theme: &Theme,
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
        FontId::proportional(11.5),
        theme.muted,
    );

    // "+" New Note button
    let new_btn = Rect::from_min_size(
        pos2(sb_rect.max.x - 38.0, docs_y - 2.0),
        vec2(22.0, 20.0),
    );
    if ui.rect_contains_pointer(new_btn) {
        painter.rect_filled(new_btn, 4.0, theme.surface().lerp_to_gamma(theme.accent, 0.15));
        if ui.input(|inp| inp.pointer.primary_clicked()) {
            action = Some(SidebarAction::NewNote);
        }
    }
    painter.text(new_btn.center(), Align2::CENTER_CENTER, "+", FontId::proportional(14.0), theme.accent);

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
                    let row_h = 30.0;
                    let (rect, resp) = ui.allocate_exact_size(vec2(row_w, row_h), egui::Sense::click());
                    let hovered = resp.hovered();

                    if is_selected && sidebar_focused {
                        resp.scroll_to_me(Some(egui::Align::Center));
                    }

                    let del_w = 20.0;
                    let del_rect = Rect::from_center_size(
                        pos2(rect.max.x - 14.0, rect.center().y),
                        vec2(del_w, 20.0),
                    );
                    let del_hover = del_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()));

                    let pill_rect = Rect::from_min_max(
                        pos2(rect.min.x + 4.0, rect.min.y + 1.5),
                        pos2(rect.max.x - 4.0, rect.max.y - 1.5),
                    );

                    if is_active || (is_selected && sidebar_focused) || hovered {
                        let bg_color = if is_selected && sidebar_focused && is_active {
                            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 40)
                        } else if is_selected && sidebar_focused {
                            if theme.is_light() {
                                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 28)
                            } else {
                                theme.surface().lerp_to_gamma(theme.accent, 0.20)
                            }
                        } else if is_active {
                            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 24)
                        } else {
                            // Soft, transparent hover background: never harsh black on light theme
                            if theme.is_light() {
                                Color32::from_rgba_unmultiplied(0, 0, 0, 13)
                            } else {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 14)
                            }
                        };
                        ui.painter().rect_filled(pill_rect, 6.0, bg_color);

                        // Left accent bar on active or focused/selected note
                        if is_active || (is_selected && sidebar_focused) {
                            let bar = Rect::from_min_size(
                                pos2(pill_rect.min.x + 1.0, pill_rect.min.y + 5.0),
                                vec2(3.0, pill_rect.height() - 10.0),
                            );
                            ui.painter().rect_filled(bar, 1.5, theme.accent);
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

                    let text_x = pill_rect.min.x + 12.0;
                    let text_right_limit = (rect.max.x - del_w - 6.0).min(sb_rect.max.x - 20.0);
                    let avail_w = (text_right_limit - text_x).max(10.0);

                    let font_id = FontId::proportional(12.5);
                    let mut display_title = note.topic.clone();
                    let full_w = ui.fonts(|f| f.layout_no_wrap(display_title.clone(), font_id.clone(), Color32::WHITE).size().x);
                    if full_w > avail_w {
                        let mut truncated = String::new();
                        for ch in note.topic.chars() {
                            let candidate = format!("{}...", truncated);
                            let w = ui.fonts(|f| f.layout_no_wrap(candidate.clone(), font_id.clone(), Color32::WHITE).size().x);
                            if w > avail_w {
                                break;
                            }
                            truncated.push(ch);
                        }
                        display_title = if truncated.is_empty() {
                            "…".to_string()
                        } else {
                            format!("{}…", truncated)
                        };
                    }

                    let title_color = if is_active {
                        theme.accent
                    } else if is_selected && sidebar_focused {
                        theme.highlight
                    } else if hovered {
                        theme.text
                    } else {
                        theme.text.lerp_to_gamma(theme.muted, 0.35)
                    };

                    let row_clip = Rect::from_min_max(
                        pos2(rect.min.x, rect.min.y),
                        pos2(text_right_limit, rect.max.y),
                    ).intersect(sb_rect);
                    let row_painter = ui.painter().with_clip_rect(row_clip);

                    row_painter.text(
                        pos2(text_x, pill_rect.center().y),
                        Align2::LEFT_CENTER,
                        display_title,
                        font_id,
                        title_color,
                    );

                    // GitHub-style unsaved document indicator ●
                    if is_active && is_dirty {
                        let dot_color = Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 180);
                        let dot_x = if hovered { rect.max.x - del_w - 12.0 } else { rect.max.x - 14.0 };
                        ui.painter().text(
                            pos2(dot_x, rect.center().y),
                            Align2::CENTER_CENTER,
                            "●",
                            FontId::proportional(11.0),
                            dot_color,
                        );
                    }

                    if hovered {
                        if del_hover {
                            ui.painter().rect_filled(del_rect, 4.0, Color32::from_rgba_unmultiplied(220, 60, 60, 35));
                            if ui.input(|i| i.pointer.primary_clicked()) {
                                action = Some(SidebarAction::DeleteNote(note.id));
                            }
                        }
                        ui.painter().text(
                            del_rect.center(),
                            Align2::CENTER_CENTER,
                            "×",
                            FontId::proportional(13.0),
                            if del_hover { Color32::from_rgb(220, 70, 70) } else { theme.muted },
                        );
                    }
                }

                // "See more" dropdown button: toggles between 50 and 100 max
                if notes_limit < 100 && total_notes_count > notes.len() {
                    ui.add_space(6.0);
                    let row_w = ui.available_width();
                    let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(row_w, 28.0), egui::Sense::click());
                    let b_hover = btn_resp.hovered();
                    let b_bg = if b_hover {
                        theme.surface().lerp_to_gamma(theme.accent, 0.12)
                    } else {
                        theme.surface()
                    };
                    ui.painter().rect(
                        btn_rect,
                        5.0,
                        b_bg,
                        egui::Stroke::new(1.0, theme.border()),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        btn_rect.center(),
                        Align2::CENTER_CENTER,
                        "▼ See more (100 max)",
                        FontId::proportional(11.5),
                        theme.accent,
                    );
                    if btn_resp.clicked() {
                        action = Some(SidebarAction::ToggleNotesLimit);
                    }
                } else if notes_limit >= 100 {
                    ui.add_space(6.0);
                    let row_w = ui.available_width();
                    let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(row_w, 28.0), egui::Sense::click());
                    let b_hover = btn_resp.hovered();
                    let b_bg = if b_hover {
                        theme.surface().lerp_to_gamma(theme.accent, 0.12)
                    } else {
                        theme.surface()
                    };
                    ui.painter().rect(
                        btn_rect,
                        5.0,
                        b_bg,
                        egui::Stroke::new(1.0, theme.border()),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        btn_rect.center(),
                        Align2::CENTER_CENTER,
                        "▲ Show less (50)",
                        FontId::proportional(11.5),
                        theme.muted,
                    );
                    if btn_resp.clicked() {
                        action = Some(SidebarAction::ToggleNotesLimit);
                    }
                }
            });
    });

    action
}
