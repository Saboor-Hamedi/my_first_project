//! Bottom-docked embedded terminal drawer and resize splitter.

use super::App;
use eframe::egui::{self, pos2, vec2, Color32, Painter, Rect, Stroke, Ui};

impl App {
    /// Renders the bottom-docked terminal drawer, handling interactive splitter resizing.
    pub fn render_terminal_drawer(
        &mut self,
        ui: &mut Ui,
        painter: &Painter,
        editor_panel_rect: Rect,
        bottom_terminal_rect: Option<Rect>,
        term_splitter_rect_opt: Option<Rect>,
        now: f64,
    ) {
        if let (Some(term_rect), Some(divider_rect)) = (bottom_terminal_rect, term_splitter_rect_opt) {
            let divider_h = 10.0;
            let available_h = (editor_panel_rect.height() - divider_h).max(140.0);
            let divider_hit_rect = divider_rect.expand2(vec2(0.0, 8.0));
            let is_divider_hovered = ui.rect_contains_pointer(divider_hit_rect);
            let primary_down = ui.input(|i| i.pointer.primary_down());
            let primary_pressed = ui.input(|i| i.pointer.primary_clicked() || i.pointer.button_pressed(egui::PointerButton::Primary));

            if is_divider_hovered && primary_pressed {
                self.is_dragging_terminal_splitter = true;
            }

            if self.is_dragging_terminal_splitter {
                if primary_down {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeRow);
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos())) {
                        let term_pixel_h = editor_panel_rect.max.y - pos.y;
                        let raw_ratio = term_pixel_h / available_h;
                        self.terminal_split_ratio = raw_ratio.clamp(0.12, 0.85);
                        ui.ctx().request_repaint();
                    }
                } else {
                    self.is_dragging_terminal_splitter = false;
                }
            } else if is_divider_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeRow);
            }

            let is_active = is_divider_hovered || self.is_dragging_terminal_splitter;
            let mid_y = divider_rect.center().y;
            let panel_left = divider_rect.min.x;
            let panel_right = divider_rect.max.x;
            if is_active {
                let line_rect = Rect::from_center_size(
                    pos2((panel_left + panel_right) * 0.5, mid_y),
                    vec2((panel_right - panel_left).max(0.0), 5.0),
                );
                painter.rect_filled(line_rect, 2.5, self.theme.accent);
            }
            let knob_w = 42.0;
            let knob_h = if is_active { 7.0 } else { 5.0 };
            let knob_rect = Rect::from_center_size(pos2((panel_left + panel_right) * 0.5, mid_y), vec2(knob_w, knob_h));
            painter.rect_filled(
                knob_rect,
                3.0,
                if is_active {
                    self.theme.accent
                } else {
                    Color32::from_rgba_unmultiplied(self.theme.muted.r(), self.theme.muted.g(), self.theme.muted.b(), 100)
                },
            );

            let knob_mid = knob_rect.center();
            let grip_color = self.theme.bg;
            for dx in [-6.0, 0.0, 6.0] {
                painter.line_segment(
                    [pos2(knob_mid.x + dx, knob_mid.y - 1.5), pos2(knob_mid.x + dx, knob_mid.y + 1.5)],
                    Stroke::new(1.0, grip_color),
                );
            }

            if self.term_pane.is_none() {
                self.term_pane = crate::terminal_pane::TerminalPane::spawn(ui.ctx(), &self.theme).ok();
            }
            if let Some(ref mut pane) = self.term_pane {
                let action = pane.ui(ui, term_rect, &self.theme, self.font_size, self.terminal_focused);
                match action {
                    crate::terminal_pane::TerminalAction::Close => {
                        self.terminal_open = false;
                        self.terminal_focused = false;
                        self.set_status("Terminal closed", now);
                        ui.ctx().request_repaint();
                    }
                    crate::terminal_pane::TerminalAction::RequestFocus => {
                        self.terminal_focused = true;
                    }
                    crate::terminal_pane::TerminalAction::None => {}
                }
            }
        } else {
            self.is_dragging_terminal_splitter = false;
        }
    }
}
