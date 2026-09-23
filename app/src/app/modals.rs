//! Modal dialogs and overlay menus (Preferences, Search, Rename, Delete confirmation, Accent picker).

use super::App;
use crate::db_worker::DbMsg;
use crate::modals::{render_delete_confirm_modal, render_rename_modal, render_search_modal};
use crate::settings::{render_setting_panel, render_setting_tabs, SettingPanelAction};
use eframe::egui::{self, pos2, Color32, Rect, Stroke, Ui};

impl App {
    /// Renders open modals (Preferences, Search, Rename, Delete, Accent Dropdown).
    pub fn render_modals(
        &mut self,
        ui: &mut Ui,
        painter: &egui::Painter,
        bounds: Rect,
        accent_anchor_rect: Rect,
        now: f64,
    ) {
        // 1. Two-Column Preferences Modal (Ctrl+,)
        if self.settings_open {
            let backdrop_alpha = if self.theme.is_light() { 90 } else { 160 };
            painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(backdrop_alpha));

            let modal_w = (bounds.width() - 80.0).clamp(640.0, 880.0);
            let modal_h = (bounds.height() - 80.0).clamp(480.0, 680.0);
            let modal_rect = Rect::from_center_size(bounds.center(), eframe::egui::vec2(modal_w, modal_h));

            // Surface container
            painter.rect(
                modal_rect,
                5.0,
                self.theme.surface(),
                Stroke::new(1.0, self.theme.border()),
                egui::StrokeKind::Inside,
            );

            // Left tab strip
            let tab_w = 180.0;
            let tabs_rect = Rect::from_min_max(
                modal_rect.min,
                pos2(modal_rect.min.x + tab_w, modal_rect.max.y),
            );
            let panel_rect = Rect::from_min_max(
                pos2(modal_rect.min.x + tab_w, modal_rect.min.y),
                modal_rect.max,
            );

            render_setting_tabs(
                ui,
                painter,
                tabs_rect,
                &mut self.active_setting_tab,
                &self.theme,
            );

            let db_tx_clone = self.db_tx.clone();
            let mut on_save = |key: &str, val: &str| {
                let _ = db_tx_clone.send(DbMsg::SaveSetting {
                    key: key.to_string(),
                    val: val.to_string(),
                });
            };

            let p_action = render_setting_panel(
                ui,
                painter,
                panel_rect,
                self.active_setting_tab,
                &mut self.editor_input_mode,
                &mut self.caret,
                &mut self.sound,
                &mut self.theme,
                &mut self.backup_dir,
                self.last_backup_status.as_deref(),
                &self.updater,
                &mut self.agent_state.deepseek_api_key_enc,
                &mut self.agent_state.deepseek_model,
                &mut on_save,
            );

            if let Some(act) = p_action {
                match act {
                    SettingPanelAction::TriggerBackup => {
                        self.trigger_backup(now);
                    }
                    SettingPanelAction::CheckUpdates => {
                        self.updater.check_for_updates(env!("CARGO_PKG_VERSION"));
                    }
                    SettingPanelAction::DownloadUpdate => {
                        self.updater.start_download();
                    }
                    SettingPanelAction::RestartToApply => {
                        let _ = self.updater.restart_and_apply();
                    }
                }
            }

            // Close preferences on Escape key or outside click
            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
            let outside_click = ui.input(|i| i.pointer.primary_clicked()) && !ui.rect_contains_pointer(modal_rect);
            if escape || outside_click {
                self.settings_open = false;
                self.set_status("Preferences closed", now);
            }
        }

        // 2. Fuzzy Search Modal (Ctrl+P)
        if self.search_open {
            let act = render_search_modal(
                ui,
                painter,
                bounds,
                &mut self.search_query,
                &self.search_results,
                &mut self.search_selected,
                &self.theme,
                self.search_just_opened,
            );
            self.search_just_opened = false;
            if let Some(item) = act.selected_item {
                self.search_open = false;
                if let Some(ref db) = self.db {
                    if let Ok(Some(n)) = db.get_note(item.id) {
                        self.load_note(n.id, n.topic, n.body, now);
                    }
                }
            }
            if act.should_close {
                self.search_open = false;
            }
        }

        // 3. Rename Document Modal (Ctrl+R)
        if self.rename_open {
            let act = render_rename_modal(
                ui,
                painter,
                bounds,
                &mut self.rename_input,
                &self.theme,
                self.rename_just_opened,
            );
            self.rename_just_opened = false;
            if let Some(new_title) = act.confirmed_title {
                self.rename_open = false;
                self.rename_active_note(&new_title, now);
            }
            if act.should_close {
                self.rename_open = false;
            }
        }

        // 4. Delete Confirmation Modal
        if self.delete_confirm_open {
            let target_title = if let Some(del_id) = self.pending_delete_note_id {
                self.notes_list.iter().find(|n| n.id == del_id).map(|n| n.topic.as_str()).unwrap_or("this note")
            } else {
                &self.active_note_title
            };
            let act = render_delete_confirm_modal(
                ui,
                painter,
                bounds,
                target_title,
                &self.theme,
                self.delete_just_opened,
            );
            self.delete_just_opened = false;
            if act.confirmed {
                self.delete_confirm_open = false;
                if let Some(del_id) = self.pending_delete_note_id.take() {
                    let _ = self.db_tx.send(DbMsg::DeleteNote { id: del_id });
                    self.notes_list.retain(|n| n.id != del_id);
                    if self.active_note_id == Some(del_id) {
                        self.delete_active_note(now);
                    } else {
                        self.open_notes.retain(|t| t.id != del_id);
                        if self.active_tab >= self.open_notes.len() && !self.open_notes.is_empty() {
                            self.active_tab = self.open_notes.len() - 1;
                        }
                        self.save_open_tabs();
                    }
                    self.set_status("Note deleted", now);
                } else {
                    self.delete_active_note(now);
                }
            }
            if act.should_close {
                self.delete_confirm_open = false;
                self.pending_delete_note_id = None;
            }
        }

        // 5. Accent Picker Dropdown
        if self.accent_dropdown_open {
            let default_theme = crate::theme::Theme::from_kind(self.theme.kind);
            let action = crate::accent::render_accent_dropdown(
                ui,
                painter,
                accent_anchor_rect,
                &mut self.accent_overrides,
                &self.theme,
                &default_theme,
            );
            if let Some(act) = action {
                match act {
                    crate::accent::AccentAction::Changed => {
                        self.accent_overrides.apply(&mut self.theme);
                        if let Some(ref db) = self.db {
                            let _ = self.accent_overrides.save_to_db(db);
                        }
                    }
                    crate::accent::AccentAction::ResetAll => {
                        self.accent_overrides.clear();
                        self.theme = crate::theme::Theme::from_kind(self.theme.kind);
                        if let Some(ref db) = self.db {
                            let _ = self.accent_overrides.save_to_db(db);
                        }
                    }
                    crate::accent::AccentAction::Close => {
                        self.accent_dropdown_open = false;
                    }
                }
            }
        }
    }
}
