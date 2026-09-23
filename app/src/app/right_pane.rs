//! Right pane split tab coordinator: Markdown live preview and DeepSeek AI Assistant.

use super::{App, RightPaneTab};
use crate::view_editor::render_markdown_preview;
use eframe::egui::{pos2, Painter, Rect, Ui};

impl App {
    /// Renders the right split pane tabs: Markdown Preview or DeepSeek AI Assistant.
    pub fn render_right_pane_tabs(
        &mut self,
        ui: &mut Ui,
        painter: &Painter,
        preview_rect_opt: Option<Rect>,
        any_modal_open: bool,
    ) {
        if let Some(p_rect) = preview_rect_opt {
            let r_header_h = crate::view_editor::TAB_ROW_H;
            let r_header_rect = Rect::from_min_max(
                p_rect.min,
                pos2(p_rect.max.x, p_rect.min.y + r_header_h),
            );
            let r_content_rect = Rect::from_min_max(
                pos2(p_rect.min.x, p_rect.min.y + r_header_h),
                p_rect.max,
            );

            let header_action = crate::view_editor::preview::render_right_pane_header(
                ui,
                painter,
                r_header_rect,
                self.right_pane_tab,
                &self.theme,
            );
            if let Some(action) = header_action {
                match action {
                    crate::view_editor::preview::RightPaneAction::SelectTab(tab) => {
                        self.right_pane_tab = tab;
                        if tab == RightPaneTab::AiAgent {
                            self.ai_focus_requested = true;
                            self.agent_state.is_open = true;
                        }
                    }
                    crate::view_editor::preview::RightPaneAction::Close => {
                        self.preview_open = false;
                        self.agent_state.is_open = false;
                        let _ = self.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                            key: "preview".into(),
                            val: "false".into(),
                        });
                    }
                }
            }

            match self.right_pane_tab {
                RightPaneTab::Preview => {
                    let note_text = self.ed.text();
                    render_markdown_preview(
                        ui,
                        painter,
                        r_content_rect,
                        &note_text,
                        &mut self.preview_scroll_y,
                        &self.theme,
                        self.font_size,
                        any_modal_open
                            || self.is_dragging_splitter
                            || self.is_dragging_sidebar_splitter,
                    );
                }
                RightPaneTab::AiAgent => {
                    self.agent_state.is_open = true;
                    let cur_text = self.ed.text();
                    let active_note_info = if let Some(n) = self.notes_list.iter().find(|n| Some(n.id) == self.active_note_id) {
                        Some((n.topic.as_str(), cur_text.as_str()))
                    } else {
                        None
                    };
                    let req_focus = self.ai_focus_requested;
                    self.ai_focus_requested = false;
                    crate::agent::deepseek_ui::render_ai_pane(
                        ui,
                        painter,
                        r_content_rect,
                        &mut self.agent_state,
                        &self.notes_list,
                        active_note_info,
                        &self.theme,
                        self.font_size,
                        req_focus,
                        any_modal_open
                            || self.is_dragging_splitter
                            || self.is_dragging_sidebar_splitter,
                    );
                }
            }
        }
    }
}
