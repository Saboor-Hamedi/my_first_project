//! Application shell: titlebar, layout frame, splitters, bottom dock, and sidebar interactions.

use super::{App, EditorInputMode, OpenNote, RightPaneTab};
use crate::mode::Mode;
use crate::sidebar::{render_sidebar, SidebarAction};
use crate::statusbar::render_bottom_dock;
use eframe::egui::{self, pos2, vec2, Color32, Rect, Stroke, Ui};

impl App {
    /// Renders the entire application frame: window frame, titlebar, splitters, editor panes, sidebar, and bottom dock.
    pub fn draw(&mut self, ui: &mut Ui, dt: f32, now: f64, typed: bool) {
        let (_cw, _lh) = self.cell_size(ui.ctx());
        let painter = ui.painter().clone();
        let bounds = ui.max_rect();

        // 5px Rounded window background frame
        let win_bg = Color32::from_rgba_unmultiplied(
            self.theme.bg.r(),
            self.theme.bg.g(),
            self.theme.bg.b(),
            (self.opacity * 255.0) as u8,
        );
        painter.rect(
            bounds,
            5.0,
            win_bg,
            Stroke::new(1.0, Color32::from_rgb(32, 34, 40)),
            egui::StrokeKind::Inside,
        );

        // Apply Acrylic / Mica backdrop blur on first frame
        if self.first_frame {
            crate::blur::apply_window_blur(self.blur_effect);
        }

        // Full Control Window Dragging:
        // 1. Alt + Left-Click Drag anywhere on the canvas (Linux / Blender / Neovim GUI convention)
        if ui.input(|i| i.modifiers.alt && i.pointer.primary_down()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let is_titlebar_visible = self.show_titlebar;

        // 2. Invisible top-edge grab strip (top 7px) whenever titlebar is hidden
        if !is_titlebar_visible {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos())) {
                if pos.y <= bounds.min.y + 7.0 && ui.input(|i| i.pointer.primary_down()) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            }
        }
        let sidebar_visible = self.sidebar_open;
        let layout = crate::layout::compute_modular_layout(
            bounds,
            sidebar_visible,
            self.sidebar_width,
            is_titlebar_visible,
            true,
        );
        let titlebar_rect = layout.titlebar_rect;
        let cmd_bar_rect = layout.cmd_bar_rect;
        let editor_panel_rect = layout.editor_panel_rect;

        // Full-width modern Titlebar (hidden if show_titlebar is false or in Zen mode)
        let (titlebar_action, accent_anchor_rect) = if is_titlebar_visible {
            let (header_title, header_dirty) = match self.mode {
                Mode::Doc => {
                    let doc_title = crate::docs::get_docs()
                        .get(self.active_doc_idx)
                        .map(|d| d.title)
                        .unwrap_or("Documentation");
                    (format!("📖 {}", doc_title), false)
                }
                Mode::Normal => (self.active_note_title.clone(), self.is_dirty),
                Mode::Help => ("✦ Quick Start Guide".to_string(), false),
                Mode::Stats => ("📊 Daily Story & Statistics".to_string(), false),
                Mode::ScanReport | Mode::ScanHistory => ("🌐 Security Scanner".to_string(), false),
                Mode::Terminal => ("💻 Embedded Terminal".to_string(), false),
            };

            let (action, anchor) = crate::view_editor::render_full_titlebar(
                ui,
                &painter,
                titlebar_rect,
                &header_title,
                header_dirty,
                &self.theme,
                self.accent_dropdown_open,
            );
            (action, anchor)
        } else {
            (None, Rect::NOTHING)
        };

        if let Some(crate::view_editor::TitlebarAction::ToggleAccentDropdown) = titlebar_action {
            self.accent_dropdown_open = !self.accent_dropdown_open;
        }

        let pointer_pos = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()));
        let is_pointer_over_ai = self.agent_state.is_open && self.agent_state.window_rect.map_or(false, |r| {
            pointer_pos.map_or(false, |p| r.contains(p))
        });

        // Sidebar Splitter Divider & Knob
        let any_modal_open = self.settings_open
            || self.search_open
            || self.help_open
            || self.rename_open
            || self.delete_confirm_open
            || self.accent_dropdown_open
            || is_pointer_over_ai;

        if let Some(center_x) = layout.splitter_center_x {
            let panel_top = editor_panel_rect.min.y;
            let panel_bottom = editor_panel_rect.max.y;
            let knob_mid = pos2(center_x, (panel_top + panel_bottom) * 0.5);
            let is_dragging = self.is_dragging_sidebar_splitter;
            let knob_w = if is_dragging { 6.0 } else { 4.0 };
            let knob_h = 36.0;
            let knob_rect = Rect::from_center_size(knob_mid, vec2(knob_w, knob_h));
            // Generous interactive hit area specifically on the knob
            let knob_hit_rect = Rect::from_center_size(knob_mid, vec2(16.0, 44.0));

            let is_knob_hovered = !any_modal_open && ui.rect_contains_pointer(knob_hit_rect);
            let primary_down = ui.input(|i| i.pointer.primary_down());
            let primary_pressed = ui.input(|i| i.pointer.primary_clicked() || i.pointer.button_pressed(egui::PointerButton::Primary));

            if is_knob_hovered && primary_pressed {
                self.is_dragging_sidebar_splitter = true;
            }

            if self.is_dragging_sidebar_splitter {
                if primary_down {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos())) {
                        let drag_x = pos.x - bounds.min.x - crate::layout::GAP;
                        if drag_x < 70.0 {
                            self.sidebar_open = false;
                            self.is_dragging_sidebar_splitter = false;
                        } else {
                            let max_sb = (bounds.width() - 200.0).clamp(crate::layout::MIN_SIDEBAR_W, crate::layout::MAX_SIDEBAR_W);
                            let new_w = drag_x.clamp(crate::layout::MIN_SIDEBAR_W, max_sb);
                            self.sidebar_width = new_w;
                        }
                    }
                } else {
                    self.is_dragging_sidebar_splitter = false;
                    let _ = self.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "sidebar_w".into(),
                        val: self.sidebar_width.to_string(),
                    });
                }
            } else if is_knob_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
            }

            let is_active = is_knob_hovered || self.is_dragging_sidebar_splitter;

            // Render tactile knob only — no harsh full-height line
            let active_knob_rect = if is_active {
                Rect::from_center_size(knob_mid, vec2(6.0, 38.0))
            } else {
                knob_rect
            };
            painter.rect_filled(
                active_knob_rect,
                2.5,
                if is_active {
                    self.theme.accent
                } else {
                    Color32::from_rgba_unmultiplied(self.theme.muted.r(), self.theme.muted.g(), self.theme.muted.b(), 100)
                },
            );

            let grip_color = self.theme.bg;
            for dy in [-5.0, 0.0, 5.0] {
                painter.line_segment(
                    [pos2(knob_mid.x - 1.2, knob_mid.y + dy), pos2(knob_mid.x + 1.2, knob_mid.y + dy)],
                    Stroke::new(1.0, grip_color),
                );
            }
        }

        // Render Doc Sidebar if in Documentation mode
        if self.mode == Mode::Doc && self.sidebar_open {
            if let Some(sb_rect) = layout.sidebar_rect {
                let doc_action = crate::docs::render_doc_sidebar(
                    ui,
                    &painter,
                    sb_rect,
                    self.active_doc_idx,
                    self.doc_selected_idx,
                    self.doc_sidebar_focused,
                    &self.theme,
                );
                if let Some(action) = doc_action {
                    match action {
                        crate::docs::DocSidebarAction::SelectDoc(idx) => {
                            self.load_doc_by_index(idx, now);
                            self.doc_sidebar_focused = true;
                        }
                        crate::docs::DocSidebarAction::ToggleSidebar => {
                            self.sidebar_open = !self.sidebar_open;
                            self.doc_sidebar_focused = self.sidebar_open;
                            let msg = if self.sidebar_open {
                                "Documentation sidebar opened"
                            } else {
                                "Documentation sidebar collapsed into full-width reader (Ctrl+B to reopen)"
                            };
                            self.set_status(msg, now);
                        }
                        crate::docs::DocSidebarAction::BackToEditor => {
                            self.mode = Mode::Normal;
                            self.set_status("Switched to Notes Editor", now);
                        }
                        crate::docs::DocSidebarAction::OpenSettings => {
                            self.settings_open = true;
                            self.settings_just_opened = true;
                        }
                    }
                }
            }
        }

        // Render Editor & Split Panes (Delegated to panes.rs)
        self.render_editor_panes(
            ui,
            &painter,
            bounds,
            editor_panel_rect,
            accent_anchor_rect,
            any_modal_open,
            dt,
            now,
            typed,
        );

        // Update Vim keystroke HUD and ShowCmd card timeout
        if self.editor_input_mode == EditorInputMode::Vim {
            self.vim.update_hud(now);
            self.showcmd.update(now);
        }

        // Check for search status feedback from Vim engine
        if let Some(msg) = self.vim.status_feedback.take() {
            self.set_status(msg, now);
        }

        // Bottom Dock (active editing mode badge, status feedback, word stats)
        let (row, col) = if self.mode == Mode::Doc {
            self.doc_ed.visual_row_col(&self.visual_lines)
        } else {
            self.ed.visual_row_col(&self.visual_lines)
        };
        let word_count = if self.mode == Mode::Doc {
            self.doc_ed.text().split_whitespace().count()
        } else {
            self.ed.text().split_whitespace().count()
        };
        let mode_badge_str = match self.editor_input_mode {
            EditorInputMode::Vim => self.vim.compact_label(),
            EditorInputMode::Hybrid => "HYBRID".to_string(),
        };

        let search_prompt = if self.editor_input_mode == EditorInputMode::Vim && self.vim.is_searching() {
            let symbol = if self.vim.search.backward { "?" } else { "/" };
            Some((symbol, self.vim.search.query.as_str(), self.vim.search.match_indices.len()))
        } else {
            None
        };

        let is_ai_active = self.preview_open && self.right_pane_tab == RightPaneTab::AiAgent;
        let toggle_ai = render_bottom_dock(
            ui,
            &painter,
            cmd_bar_rect,
            0.0,
            self.in_command,
            &self.cmd_ed.text(),
            self.cmd_ed.cur,
            self.cmd_ed.selected_range(),
            &self.status_msg,
            self.status_time,
            now,
            row + 1,
            col + 1,
            word_count,
            Some(mode_badge_str.as_str()),
            search_prompt,
            &self.theme,
            is_ai_active,
        );
        if toggle_ai {
            if self.preview_open && self.right_pane_tab == RightPaneTab::AiAgent {
                self.preview_open = false;
                self.agent_state.is_open = false;
                ui.memory_mut(|m| m.surrender_focus(egui::Id::new("deepseek_prompt_input")));
                self.set_status("AI Agent closed", now);
            } else {
                self.preview_open = true;
                self.right_pane_tab = RightPaneTab::AiAgent;
                self.ai_focus_requested = true;
                self.agent_state.is_open = true;
                ui.memory_mut(|m| m.request_focus(egui::Id::new("deepseek_prompt_input")));
                self.set_status("AI Assistant opened (Ctrl+Shift+I to toggle)", now);
            }
        }


        // Sleek Sidebar (Ctrl+B)
        if self.sidebar_open && self.mode != Mode::Doc {
            if let Some(sb_rect) = layout.sidebar_rect {
                let active_mode_idx = match self.mode {
                    Mode::Normal => 0,
                    Mode::Stats => 1,
                    Mode::Doc | Mode::Help | Mode::ScanReport | Mode::ScanHistory | Mode::Terminal => 0,
                };
                let action = render_sidebar(
                    ui,
                    &painter,
                    sb_rect,
                    active_mode_idx,
                    self.active_note_id,
                    &self.notes_list,
                    self.sidebar_notes_limit,
                    self.total_notes_count,
                    self.is_dirty,
                    &self.theme,
                    self.sidebar_selected_idx,
                    self.sidebar_focused,
                );

                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if sb_rect.contains(pos) {
                            self.sidebar_focused = true;
                        } else if editor_panel_rect.contains(pos) {
                            self.sidebar_focused = false;
                        }
                    }
                }

                if let Some(act) = action {
                    match act {
                        SidebarAction::SwitchMode(idx) => {
                            match idx {
                                0 => self.mode = Mode::Normal,
                                1 => {
                                    self.reload_db_state();
                                    self.mode = Mode::Stats;
                                }
                                _ => {}
                            }
                        }
                        SidebarAction::LoadNote { id, topic, body, index } => {
                            self.load_note(id, topic, body, now);
                            self.sidebar_selected_idx = index;
                            self.sidebar_focused = true;
                        }
                        SidebarAction::NewNote => {
                            if let Some(cur) = self.open_notes.get_mut(self.active_tab) {
                                cur.editor = self.ed.clone();
                                cur.title = self.active_note_title.clone();
                                cur.scroll_y = self.scroll_y;
                                cur.is_dirty = self.is_dirty;
                            }
                            self.active_note_id = None;
                            self.save_active_note_id();
                            self.active_note_title = "Untitled Note".to_string();
                            self.ed.clear();
                            self.mode = Mode::Normal;
                            self.vim.set_mode(crate::vim::VimSubMode::Normal, &mut self.ed);
                            self.is_dirty = false;
                            self.scroll_y = 0.0;
                            self.open_notes.push(OpenNote {
                                id: 0,
                                title: "Untitled Note".to_string(),
                                editor: self.ed.clone(),
                                scroll_y: 0.0,
                                is_dirty: false,
                            });
                            self.active_tab = self.open_notes.len() - 1;
                            self.save_open_tabs();
                            self.set_status("Created new note", now);
                        }
                        SidebarAction::DeleteNote(id) => {
                            self.pending_delete_note_id = Some(id);
                            self.delete_confirm_open = true;
                            self.delete_just_opened = true;
                        }
                        SidebarAction::ToggleNotesLimit => {
                            self.sidebar_notes_limit = if self.sidebar_notes_limit >= 100 { 50 } else { 100 };
                            self.reload_db_state();
                        }
                        SidebarAction::OpenSettings => {
                            self.settings_open = true;
                            self.settings_just_opened = true;
                        }
                    }
                }
            }
        }


        // Command Autocomplete Popup (Pops above bottom dock, rendered on top of sidebar)
        crate::command::command_suggestion::render_command_suggestions_overlay(
            self,
            ui,
            &painter,
            cmd_bar_rect,
            now,
        );

        // Render Modal dialogs (Preferences, Search, Rename, Delete, Accent dropdown)
        self.render_modals(ui, &painter, bounds, accent_anchor_rect, now);

        // Poll DeepSeek background worker for any completed responses
        self.agent_state.poll_response();
    }
}
