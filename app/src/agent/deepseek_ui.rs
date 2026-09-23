//! Lightweight floating, resizable DeepSeek AI Agent dropdown interface.

use super::{AgentState, MessageRole};
use crate::theme::Theme;
use crate::view_editor::preview::MdBlock;
use core::Note;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Key, Rect, Stroke};

pub fn render_ai_dropdown(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    state: &mut AgentState,
    notes: &[Note],
    active_note: Option<(&str, &str)>,
    theme: &Theme,
) {
    if !state.is_open {
        return;
    }

    // Seamless geometry: anchored directly above bottom status dock, aligned to 5px grid
    let gap = 5.0f32;
    let right_x = bounds.max.x - gap;
    let bottom_y = bounds.max.y - gap - 32.0 - gap; // Exactly 5px above the 32px bottom dock
    let top_limit = bounds.min.y + gap + 32.0 + gap; // Below titlebar

    let min_w = 380.0f32;
    let min_h = 360.0f32;
    let max_w = (bounds.width() - gap * 2.0).max(min_w);
    let max_h = (bottom_y - top_limit).max(min_h);

    let default_w = 460.0f32.clamp(min_w, max_w);
    let default_h = 500.0f32.clamp(min_h, max_h);

    let window_rect = state.window_rect.get_or_insert_with(|| {
        Rect::from_min_max(
            pos2((right_x - default_w).max(bounds.min.x + gap), (bottom_y - default_h).max(top_limit)),
            pos2(right_x, bottom_y),
        )
    });

    // Ensure bottom-right remains anchored to the 5px status bar grid
    let cur_w = window_rect.width().clamp(min_w, max_w);
    let cur_h = window_rect.height().clamp(min_h, max_h);
    *window_rect = Rect::from_min_max(pos2(right_x - cur_w, bottom_y - cur_h), pos2(right_x, bottom_y));

    let rect = *window_rect;

    // Handle Top-Left resizing knob / edges
    let resize_knob_rect = Rect::from_min_size(rect.min, vec2(18.0, 18.0));
    let is_resizing_hover = ui.rect_contains_pointer(resize_knob_rect);
    if is_resizing_hover || state.is_resizing {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNorthWest);
    }

    if ui.input(|i| i.pointer.primary_down()) {
        if is_resizing_hover {
            state.is_resizing = true;
        }
    } else {
        state.is_resizing = false;
    }

    if state.is_resizing {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let new_min_x = pos.x.clamp(bounds.min.x + gap, right_x - min_w);
            let new_min_y = pos.y.clamp(top_limit, bottom_y - min_h);
            *window_rect = Rect::from_min_max(pos2(new_min_x, new_min_y), pos2(right_x, bottom_y));
        }
    }

    // ── Dropdown Window Frame with ambient elevation shadow ────────────────────
    let shadow_color = if theme.is_light() {
        Color32::from_rgba_unmultiplied(0, 0, 0, 18)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 55)
    };
    painter.rect_filled(rect.translate(vec2(0.0, 3.0)).expand(2.0), 8.0, shadow_color);

    let bg_color = if theme.is_light() {
        theme.surface()
    } else {
        theme.surface().lerp_to_gamma(theme.bg, 0.20)
    };

    painter.rect(
        rect,
        6.0,
        bg_color,
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    // Diagonal grip lines on the top-left resize handle
    let knob_color = if is_resizing_hover || state.is_resizing { theme.accent } else { theme.muted };
    for &offset in &[4.0, 8.0, 12.0] {
        painter.line_segment(
            [pos2(rect.min.x + offset, rect.min.y + 2.0), pos2(rect.min.x + 2.0, rect.min.y + offset)],
            Stroke::new(1.2, knob_color),
        );
    }

    // ── Header Bar ────────────────────────────────────────────────────────────
    let header_h = 38.0;
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), header_h));

    let title_font = FontId::proportional(13.0);
    painter.text(
        pos2(header_rect.min.x + 16.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        "AI Assistant",
        title_font,
        theme.highlight,
    );

    // Header buttons on right: Clear, Delete, Close
    let btn_y = header_rect.center().y - 11.0;
    let btn_h = 22.0;

    // 1. Close button (×)
    let close_rect = Rect::from_min_size(pos2(header_rect.max.x - 26.0, btn_y), vec2(20.0, btn_h));
    if crate::ui_components::render_close_button_rect(ui, painter, close_rect, theme, "deepseek_close_btn") {
        state.is_open = false;
    }

    // 2. "Delete" button (delete conversation)
    let del_rect = Rect::from_min_size(pos2(close_rect.min.x - 4.0 - 44.0, btn_y), vec2(44.0, btn_h));
    let del_hover = ui.rect_contains_pointer(del_rect);
    if del_hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        painter.rect_filled(del_rect, 4.0, Color32::from_rgba_unmultiplied(220, 60, 60, 26));
        if ui.input(|i| i.pointer.primary_clicked()) {
            state.chat_history.clear();
            state.input_text.clear();
            state.error_msg = None;
        }
    }
    painter.text(
        del_rect.center(),
        Align2::CENTER_CENTER,
        "Delete",
        FontId::proportional(11.0),
        if del_hover { Color32::from_rgb(230, 80, 80) } else { theme.muted },
    );

    // 3. "Clear" button
    let clear_rect = Rect::from_min_size(pos2(del_rect.min.x - 4.0 - 42.0, btn_y), vec2(42.0, btn_h));
    let clear_hover = ui.rect_contains_pointer(clear_rect);
    if clear_hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        painter.rect_filled(clear_rect, 4.0, theme.bg);
        if ui.input(|i| i.pointer.primary_clicked()) {
            state.clear_chat();
        }
    }
    painter.text(
        clear_rect.center(),
        Align2::CENTER_CENTER,
        "Clear",
        FontId::proportional(11.0),
        if clear_hover { theme.text } else { theme.muted },
    );

    // Header divider line
    painter.line_segment(
        [pos2(rect.min.x, header_rect.max.y), pos2(rect.max.x, header_rect.max.y)],
        Stroke::new(1.0, theme.border()),
    );

    // ── Chat Scroll Area ──────────────────────────────────────────────────────
    let input_area_h = 58.0;
    let chat_rect = Rect::from_min_max(
        pos2(rect.min.x + 8.0, header_rect.max.y + 6.0),
        pos2(rect.max.x - 8.0, rect.max.y - input_area_h - 4.0),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(chat_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("ai_agent_chat_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(4.0);

                if state.chat_history.is_empty() {
                    // Empty state with interactive starter prompts
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Ask anything about your notes")
                                .font(FontId::proportional(13.0))
                                .color(theme.muted),
                        );
                        ui.add_space(10.0);

                        let starters = [
                            "When did I add my last note?",
                            "What topics are in my database?",
                            "Summarize what I have written so far",
                        ];

                        for prompt in &starters {
                            let (p_rect, p_resp) = ui.allocate_exact_size(
                                vec2(ui.available_width() - 24.0, 28.0),
                                egui::Sense::click(),
                            );
                            let h = p_resp.hovered();
                            let bg = if h {
                                Color32::from_rgba_unmultiplied(
                                    theme.accent.r(),
                                    theme.accent.g(),
                                    theme.accent.b(),
                                    if theme.is_light() { 24 } else { 35 },
                                )
                            } else {
                                theme.bg
                            };
                            ui.painter().rect(
                                p_rect,
                                4.0,
                                bg,
                                Stroke::new(1.0, if h { theme.accent } else { theme.border() }),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                p_rect.center(),
                                Align2::CENTER_CENTER,
                                format!("\"{}\"", prompt),
                                FontId::proportional(11.5),
                                if h { theme.accent } else { theme.text },
                            );

                            if p_resp.clicked() {
                                state.input_text = prompt.to_string();
                                state.send_message(notes, active_note);
                            }
                        }
                    });
                } else {
                    for (msg_idx, msg) in state.chat_history.iter().enumerate() {
                        let is_user = msg.role == MessageRole::User;

                        if is_user {
                            // User Message: sleek right-aligned rounded bubble with accent tint
                            let avail_w = ui.available_width();
                            let max_bubble_w = (avail_w * 0.82).max(180.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                    let bubble_bg = if theme.is_light() {
                                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 26)
                                    } else {
                                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 36)
                                    };
                                    let stroke = Stroke::new(
                                        1.0,
                                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 85),
                                    );
                                    egui::Frame::new()
                                        .fill(bubble_bg)
                                        .corner_radius(7.0)
                                        .stroke(stroke)
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.set_max_width(max_bubble_w);
                                            ui.label(
                                                egui::RichText::new(&msg.content)
                                                    .font(FontId::proportional(12.0))
                                                    .color(theme.text),
                                            );
                                        });
                                });
                            });
                            ui.add_space(8.0);
                        } else {
                            // Assistant Message: clean vector AI badge and header, then formatted markdown
                            ui.horizontal(|ui| {
                                let (badge_rect, _) = ui.allocate_exact_size(vec2(22.0, 15.0), egui::Sense::hover());
                                ui.painter().rect(
                                    badge_rect,
                                    3.0,
                                    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 28),
                                    Stroke::new(1.0, theme.accent),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    badge_rect.center(),
                                    Align2::CENTER_CENTER,
                                    "AI",
                                    FontId::monospace(9.5),
                                    theme.accent,
                                );
                                ui.label(
                                    egui::RichText::new("Assistant")
                                        .font(FontId::proportional(12.0))
                                        .strong()
                                        .color(theme.highlight),
                                );
                                ui.label(
                                    egui::RichText::new(&msg.timestamp)
                                        .font(FontId::monospace(9.0))
                                        .color(theme.muted),
                                );
                            });

                            ui.add_space(3.0);
                            render_chat_markdown(ui, &msg.content, theme, msg_idx);
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);
                        }
                    }

                    if state.is_thinking {
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(120));
                        ui.horizontal(|ui| {
                            let t = ui.input(|i| i.time);
                            let dots = match ((t * 3.5) as u32) % 4 {
                                1 => ".  ",
                                2 => ".. ",
                                3 => "...",
                                _ => "   ",
                            };
                            ui.label(
                                egui::RichText::new(format!("Thinking{}", dots))
                                    .font(FontId::monospace(11.5))
                                    .color(theme.accent),
                            );
                        });
                    }
                }

                if state.scroll_to_bottom {
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                    state.scroll_to_bottom = false;
                }
            });
    });

    // Divider above input bar
    let input_top_y = rect.max.y - input_area_h;
    painter.line_segment(
        [pos2(rect.min.x, input_top_y), pos2(rect.max.x, input_top_y)],
        Stroke::new(1.0, theme.border()),
    );

    // ── Bottom Input Row ──────────────────────────────────────────────────────
    let send_btn_w = 34.0;
    let send_btn_h = 34.0;
    let send_btn_y = input_top_y + (input_area_h - send_btn_h) * 0.5;
    let send_btn_rect = Rect::from_min_size(
        pos2(rect.max.x - send_btn_w - 10.0, send_btn_y),
        vec2(send_btn_w, send_btn_h),
    );

    let input_rect = Rect::from_min_max(
        pos2(rect.min.x + 10.0, input_top_y + 9.0),
        pos2(send_btn_rect.min.x - 8.0, rect.max.y - 9.0),
    );

    // Custom themed container for TextEdit
    let edit_id = egui::Id::new("deepseek_prompt_input");
    let is_focused = ui.memory(|m| m.has_focus(edit_id));
    let is_input_hover = ui.rect_contains_pointer(input_rect);

    let input_bg = if theme.is_light() {
        theme.bg
    } else {
        theme.surface()
    };
    let border_stroke = if is_focused {
        Stroke::new(1.2, theme.accent)
    } else if is_input_hover {
        Stroke::new(1.0, theme.border().lerp_to_gamma(theme.accent, 0.4))
    } else {
        Stroke::new(1.0, theme.border())
    };

    painter.rect(
        input_rect,
        6.0,
        input_bg,
        border_stroke,
        egui::StrokeKind::Inside,
    );

    let edit_resp = ui.put(
        input_rect.shrink2(vec2(8.0, 7.0)),
        egui::TextEdit::multiline(&mut state.input_text)
            .id(edit_id)
            .font(FontId::proportional(12.5))
            .text_color(theme.text)
            .hint_text(
                egui::WidgetText::from("Ask DeepSeek about your notes... (Enter to send)")
                    .color(theme.muted),
            )
            .desired_rows(1)
            .frame(false),
    );

    // Send on Enter (without Shift)
    let enter_pressed = edit_resp.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift);
    let is_send_hover = ui.rect_contains_pointer(send_btn_rect);
    let send_clicked = is_send_hover && ui.input(|i| i.pointer.primary_clicked());

    if (enter_pressed || send_clicked) && !state.is_thinking {
        let clean = state.input_text.trim().to_string();
        if !clean.is_empty() {
            state.input_text = clean;
            state.send_message(notes, active_note);
        } else {
            state.input_text.clear();
        }
        edit_resp.request_focus();
    }

    // ── Vector Send Button ────────────────────────────────────────────────────
    let has_content = !state.input_text.trim().is_empty();
    if is_send_hover && !state.is_thinking && has_content {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let (send_bg, send_border, arrow_color) = if state.is_thinking {
        (theme.bg, Stroke::new(1.0, theme.border()), theme.muted)
    } else if has_content {
        let bg = if is_send_hover {
            theme.accent.lerp_to_gamma(Color32::WHITE, 0.15)
        } else {
            theme.accent
        };
        let arrow = if theme.is_light() {
            Color32::WHITE
        } else {
            theme.bg
        };
        (bg, Stroke::NONE, arrow)
    } else {
        let bg = if is_send_hover {
            theme.surface().lerp_to_gamma(theme.border(), 0.25)
        } else {
            input_bg
        };
        let stroke = Stroke::new(1.0, if is_send_hover { theme.accent } else { theme.border() });
        let arrow = if is_send_hover { theme.accent } else { theme.muted };
        (bg, stroke, arrow)
    };

    painter.rect(
        send_btn_rect,
        6.0,
        send_bg,
        send_border,
        egui::StrokeKind::Inside,
    );

    let c = send_btn_rect.center();
    if state.is_thinking {
        let t = ui.input(|i| i.time);
        let pulse_r = 3.0 + (t * 5.0).sin().abs() as f32 * 2.0;
        painter.circle_filled(c, pulse_r, theme.accent);
    } else {
        let arrow_stroke = Stroke::new(1.8, arrow_color);
        painter.line_segment([pos2(c.x - 5.5, c.y), pos2(c.x + 4.5, c.y)], arrow_stroke);
        painter.line_segment([pos2(c.x + 0.5, c.y - 4.5), pos2(c.x + 4.5, c.y)], arrow_stroke);
        painter.line_segment([pos2(c.x + 0.5, c.y + 4.5), pos2(c.x + 4.5, c.y)], arrow_stroke);
    }
}

fn render_chat_markdown(ui: &mut egui::Ui, text: &str, theme: &Theme, block_seed: usize) {
    let blocks = crate::view_editor::preview::parse_markdown(text);
    let max_text_w = (ui.available_width() - 8.0).max(100.0);
    let font_size = 12.0f32;

    for (b_idx, block) in blocks.into_iter().enumerate() {
        match block {
            MdBlock::Heading1(text) => {
                ui.add_space(8.0);
                let font = FontId::proportional(font_size * 1.48);
                let galley = ui.painter().layout(text, font, theme.highlight, max_text_w);
                let text_h = galley.size().y;
                let (h_rect, _) = ui.allocate_exact_size(vec2(max_text_w, text_h + 4.0), egui::Sense::hover());
                ui.painter().galley(h_rect.min, galley, theme.highlight);
                ui.painter().line_segment(
                    [pos2(h_rect.min.x, h_rect.max.y), pos2(h_rect.max.x, h_rect.max.y)],
                    Stroke::new(1.0, theme.border()),
                );
                ui.add_space(6.0);
            }
            MdBlock::Heading2(text) => {
                ui.add_space(6.0);
                let font = FontId::proportional(font_size * 1.25);
                let galley = ui.painter().layout(text, font, theme.accent, max_text_w);
                let text_h = galley.size().y;
                let (h_rect, _) = ui.allocate_exact_size(vec2(max_text_w, text_h), egui::Sense::hover());
                ui.painter().galley(h_rect.min, galley, theme.accent);
                ui.add_space(4.0);
            }
            MdBlock::Heading3(text) => {
                ui.add_space(4.0);
                let font = FontId::proportional(font_size * 1.10);
                let galley = ui.painter().layout(text, font, theme.text, max_text_w);
                let text_h = galley.size().y;
                let (h_rect, _) = ui.allocate_exact_size(vec2(max_text_w, text_h), egui::Sense::hover());
                ui.painter().galley(h_rect.min, galley, theme.text);
                ui.add_space(3.0);
            }
            MdBlock::Heading4(text) => {
                ui.add_space(3.0);
                let font = FontId::proportional(font_size * 1.00);
                let galley = ui.painter().layout(text, font, theme.muted, max_text_w);
                let text_h = galley.size().y;
                let (h_rect, _) = ui.allocate_exact_size(vec2(max_text_w, text_h), egui::Sense::hover());
                ui.painter().galley(h_rect.min, galley, theme.muted);
                ui.add_space(2.0);
            }
            MdBlock::Paragraph(text) => {
                for sub_line in text.lines() {
                    let job = crate::view_editor::preview::build_inline_job(
                        sub_line,
                        font_size,
                        theme.text,
                        theme,
                        max_text_w,
                    );
                    let galley = ui.painter().layout_job(job);
                    let text_h = galley.size().y;
                    let (p_rect, _) = ui.allocate_exact_size(vec2(max_text_w, text_h), egui::Sense::hover());
                    ui.painter().galley(p_rect.min, galley, theme.text);
                }
                ui.add_space(5.0);
            }
            MdBlock::Quote(text) => {
                let inner_w = (max_text_w - 24.0).max(40.0);
                let job = crate::view_editor::preview::build_inline_job(
                    &text,
                    font_size * 0.96,
                    theme.muted,
                    theme,
                    inner_w,
                );
                let galley = ui.painter().layout_job(job);
                let text_h = galley.size().y;
                let box_h = text_h + 12.0;
                let (q_rect, _) = ui.allocate_exact_size(vec2(max_text_w, box_h), egui::Sense::hover());

                // Left quote vertical accent border
                let bar_rect = Rect::from_min_size(q_rect.min, vec2(3.5, box_h));
                ui.painter().rect_filled(bar_rect, 1.5, theme.accent);

                // Subtle quote background box
                let bg_rect = Rect::from_min_size(pos2(q_rect.min.x + 4.0, q_rect.min.y), vec2(max_text_w - 4.0, box_h));
                ui.painter().rect_filled(
                    bg_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 14),
                );
                ui.painter().rect_stroke(
                    bg_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 22)),
                    egui::StrokeKind::Inside,
                );
                ui.painter().galley(pos2(q_rect.min.x + 14.0, q_rect.min.y + 6.0), galley, theme.text);
                ui.add_space(6.0);
            }
            MdBlock::ListItem { bullet, text, checked, indent_level } => {
                let indent_offset = (indent_level as f32) * 16.0;
                let available_w = (max_text_w - indent_offset).max(40.0);

                let job = crate::view_editor::preview::build_inline_job(
                    &text,
                    font_size,
                    theme.text,
                    theme,
                    available_w - 28.0,
                );
                let galley = ui.painter().layout_job(job);
                let text_h = galley.size().y;
                let row_h = text_h.max(18.0);
                let (row_rect, _) = ui.allocate_exact_size(vec2(max_text_w, row_h), egui::Sense::hover());

                let item_x = row_rect.min.x + indent_offset;
                let item_y = row_rect.min.y;

                if let Some(is_checked) = checked {
                    let cb_size = 14.0;
                    let cb_y = item_y + 1.5;
                    let cb_rect = Rect::from_min_size(pos2(item_x, cb_y), vec2(cb_size, cb_size));

                    if is_checked {
                        ui.painter().rect_filled(cb_rect, 3.0, theme.accent);
                        let p1 = pos2(cb_rect.min.x + 3.0, cb_rect.min.y + 7.0);
                        let p2 = pos2(cb_rect.min.x + 5.5, cb_rect.min.y + 10.0);
                        let p3 = pos2(cb_rect.min.x + 11.0, cb_rect.min.y + 4.0);
                        ui.painter().line_segment([p1, p2], Stroke::new(1.6, theme.bg));
                        ui.painter().line_segment([p2, p3], Stroke::new(1.6, theme.bg));

                        let text_start = pos2(item_x + 22.0, item_y);
                        ui.painter().galley(text_start, galley, Color32::from_rgba_unmultiplied(255, 255, 255, 140));
                        let strike_y = item_y + text_h * 0.52;
                        ui.painter().line_segment(
                            [pos2(text_start.x, strike_y), pos2(text_start.x + available_w - 28.0, strike_y)],
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 120)),
                        );
                    } else {
                        ui.painter().rect_filled(
                            cb_rect,
                            3.0,
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 18),
                        );
                        ui.painter().rect_stroke(
                            cb_rect,
                            3.0,
                            Stroke::new(1.5, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 140)),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().galley(pos2(item_x + 22.0, item_y), galley, theme.text);
                    }
                } else {
                    let bullet_font = FontId::monospace(font_size * 0.92);
                    let bullet_w = if bullet.ends_with('.') {
                        (bullet.len() as f32 * font_size * 0.58).max(18.0)
                    } else {
                        14.0
                    };
                    ui.painter().text(pos2(item_x, item_y), Align2::LEFT_TOP, &bullet, bullet_font, theme.accent);
                    ui.painter().galley(pos2(item_x + bullet_w + 4.0, item_y), galley, theme.text);
                }
                ui.add_space(4.0);
            }
            MdBlock::CodeBlock { lang, code } => {
                let lines: Vec<&str> = code.lines().collect();
                let line_count = lines.len().max(1);
                let line_h = (font_size * 1.45).round();
                let pad_x = 12.0;
                let pad_top = 28.0; // Header area for language label and copy button
                let avail_w = (max_text_w - pad_x * 2.0).max(10.0);

                let line_galleys: Vec<std::sync::Arc<egui::Galley>> = lines
                    .iter()
                    .map(|line_str| ui.painter().layout_job(crate::view_editor::preview::highlight_code_line(line_str, &lang, font_size, theme)))
                    .collect();
                let max_line_w = line_galleys.iter().map(|g| g.size().x).fold(0.0f32, f32::max);
                let max_scroll_x = (max_line_w - avail_w).max(0.0);
                let needs_h_scroll = max_scroll_x > 0.0;
                let pad_bottom = if needs_h_scroll { 16.0 } else { 10.0 };
                let block_h = pad_top + (line_count as f32 * line_h) + pad_bottom;

                // Exactly sized container that NEVER overflows right
                let (code_rect, _) = ui.allocate_exact_size(vec2(max_text_w, block_h), egui::Sense::hover());

                // Single cohesive code wrapper card matching theme surface and border
                ui.painter().rect(
                    code_rect,
                    5.0,
                    theme.surface(),
                    Stroke::new(1.0, theme.border()),
                    egui::StrokeKind::Inside,
                );

                // Horizontal scroll state & input
                let scroll_id = egui::Id::new("ai_code_block_scroll_x").with(block_seed).with(b_idx);
                let mut scroll_x: f32 = ui.data(|d| d.get_temp(scroll_id).unwrap_or(0.0));
                if ui.rect_contains_pointer(code_rect) && needs_h_scroll {
                    let h_delta = ui.input(|i| {
                        if i.modifiers.shift {
                            if i.smooth_scroll_delta.y.abs() > 0.001 {
                                i.smooth_scroll_delta.y
                            } else {
                                i.raw_scroll_delta.y * 0.5
                            }
                        } else if i.smooth_scroll_delta.x.abs() > 0.001 {
                            i.smooth_scroll_delta.x
                        } else {
                            i.raw_scroll_delta.x * 0.5
                        }
                    });
                    if h_delta != 0.0 {
                        scroll_x = (scroll_x - h_delta).clamp(0.0, max_scroll_x);
                        ui.data_mut(|d| d.insert_temp(scroll_id, scroll_x));
                        ui.ctx().request_repaint();
                    }
                }
                scroll_x = scroll_x.clamp(0.0, max_scroll_x);

                // Top-right copy button logic
                let copy_id = egui::Id::new("ai_code_block_copy").with(block_seed).with(b_idx);
                let current_time = ui.input(|i| i.time);
                let last_copied: Option<f64> = ui.data(|d| d.get_temp(copy_id));
                let is_copied = last_copied.map_or(false, |t| current_time - t < 1.8);

                let btn_w = 58.0;
                let btn_h = 18.0;
                let btn_rect = Rect::from_min_size(
                    pos2(code_rect.max.x - btn_w - 10.0, code_rect.min.y + 5.0),
                    vec2(btn_w, btn_h),
                );
                let is_btn_hovered = ui.rect_contains_pointer(btn_rect);
                if is_btn_hovered {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if is_btn_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                    crate::input::global::set_win32_clipboard(&code);
                    ui.data_mut(|d| d.insert_temp(copy_id, current_time));
                    ui.ctx().request_repaint();
                }
                if is_copied {
                    let elapsed = current_time - last_copied.unwrap();
                    let remaining = 1.8 - elapsed;
                    if remaining > 0.0 {
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis((remaining * 1000.0) as u64 + 20));
                    }
                }

                // Subtle language label on the left (integrated into card)
                let tag = if lang.is_empty() { "CODE" } else { &lang.to_uppercase() };
                ui.painter().text(
                    pos2(code_rect.min.x + pad_x, code_rect.min.y + 14.0),
                    Align2::LEFT_CENTER,
                    tag,
                    FontId::monospace(10.0),
                    theme.accent,
                );

                // Render copy button in top-right
                let (btn_bg, btn_text, btn_color) = if is_copied {
                    (
                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 45),
                        "✓ Copied",
                        theme.accent,
                    )
                } else if is_btn_hovered {
                    (
                        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 45),
                        "Copy",
                        theme.text,
                    )
                } else {
                    (
                        Color32::TRANSPARENT,
                        "Copy",
                        theme.muted,
                    )
                };

                ui.painter().rect(
                    btn_rect,
                    3.0,
                    btn_bg,
                    Stroke::new(1.0, if is_copied { theme.accent } else if is_btn_hovered { theme.accent } else { theme.border() }),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    btn_rect.center(),
                    Align2::CENTER_CENTER,
                    btn_text,
                    FontId::proportional(10.5),
                    btn_color,
                );

                // Code lines clipped to padding area
                let code_clip_rect = Rect::from_min_max(
                    pos2(code_rect.min.x + pad_x, code_rect.min.y + pad_top),
                    pos2(code_rect.max.x - pad_x, code_rect.max.y - pad_bottom),
                );
                let line_painter = ui.painter().with_clip_rect(code_clip_rect.intersect(ui.clip_rect()));

                let mut line_y = code_rect.min.y + pad_top;
                for galley in line_galleys {
                    line_painter.galley(
                        pos2(code_rect.min.x + pad_x - scroll_x, line_y),
                        galley,
                        theme.text,
                    );
                    line_y += line_h;
                }

                // Horizontal scroll bar when needed
                if needs_h_scroll {
                    let track_y = code_rect.max.y - 7.0;
                    let track_rect = Rect::from_min_max(
                        pos2(code_rect.min.x + pad_x, track_y),
                        pos2(code_rect.max.x - pad_x, track_y + 3.5),
                    );
                    let ratio = scroll_x / max_scroll_x;
                    let thumb_w = (avail_w * (avail_w / max_line_w)).clamp(20.0, avail_w);
                    let thumb_x = code_rect.min.x + pad_x + ratio * (avail_w - thumb_w);
                    let thumb_rect = Rect::from_min_size(pos2(thumb_x, track_y), vec2(thumb_w, 3.5));

                    let hit_rect = Rect::from_min_max(
                        pos2(code_rect.min.x + pad_x, track_y - 3.0),
                        pos2(code_rect.max.x - pad_x, track_y + 6.5),
                    );
                    let is_hit = ui.rect_contains_pointer(hit_rect);
                    if is_hit && ui.input(|i| i.pointer.primary_down()) {
                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                            let r = ((pos.x - (code_rect.min.x + pad_x) - thumb_w * 0.5) / (avail_w - thumb_w).max(1.0)).clamp(0.0, 1.0);
                            scroll_x = r * max_scroll_x;
                            ui.data_mut(|d| d.insert_temp(scroll_id, scroll_x));
                            ui.ctx().request_repaint();
                        }
                    }

                    ui.painter().rect_filled(
                        track_rect,
                        1.75,
                        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 30),
                    );
                    let thumb_color = if is_hit {
                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 180)
                    } else {
                        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 110)
                    };
                    ui.painter().rect_filled(thumb_rect, 1.75, thumb_color);
                }

                ui.add_space(8.0);
            }
            MdBlock::Table { headers, rows } => {
                let col_count = headers.len().max(1);
                let col_w = (max_text_w / col_count as f32).max(50.0);
                let cell_pad_x = 8.0;
                let cell_pad_y = 5.0;
                let row_h = font_size * 1.4 + cell_pad_y * 2.0;
                let table_h = row_h * (1 + rows.len()) as f32;

                let (table_rect, _) = ui.allocate_exact_size(vec2(max_text_w, table_h), egui::Sense::hover());

                // Table outer rounded container
                ui.painter().rect_stroke(
                    table_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 40)),
                    egui::StrokeKind::Inside,
                );

                // Header row background
                let header_rect = Rect::from_min_size(table_rect.min, vec2(max_text_w, row_h));
                ui.painter().rect_filled(
                    header_rect,
                    egui::CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 },
                    Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 24),
                );
                ui.painter().line_segment(
                    [header_rect.left_bottom(), header_rect.right_bottom()],
                    Stroke::new(1.5, theme.accent),
                );

                // Header cells
                for (c_idx, h_text) in headers.iter().enumerate() {
                    let c_x = table_rect.min.x + c_idx as f32 * col_w + cell_pad_x;
                    let job = crate::view_editor::preview::build_inline_job(h_text, font_size * 0.95, theme.highlight, theme, col_w - cell_pad_x * 2.0);
                    let galley = ui.painter().layout_job(job);
                    ui.painter().galley(pos2(c_x, table_rect.min.y + cell_pad_y), galley, theme.highlight);
                }

                // Rows
                for (r_idx, row) in rows.iter().enumerate() {
                    let r_y = table_rect.min.y + (r_idx + 1) as f32 * row_h;
                    if r_idx % 2 == 1 {
                        let r_rect = Rect::from_min_size(pos2(table_rect.min.x, r_y), vec2(max_text_w, row_h));
                        let corner = if r_idx == rows.len() - 1 {
                            egui::CornerRadius { nw: 0, ne: 0, sw: 4, se: 4 }
                        } else {
                            egui::CornerRadius::ZERO
                        };
                        ui.painter().rect_filled(
                            r_rect,
                            corner,
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 12),
                        );
                    }
                    for (c_idx, cell_text) in row.iter().enumerate() {
                        let c_x = table_rect.min.x + c_idx as f32 * col_w + cell_pad_x;
                        let job = crate::view_editor::preview::build_inline_job(cell_text, font_size * 0.92, theme.text, theme, col_w - cell_pad_x * 2.0);
                        let galley = ui.painter().layout_job(job);
                        ui.painter().galley(pos2(c_x, r_y + cell_pad_y), galley, theme.text);
                    }
                }
                ui.add_space(6.0);
            }
            MdBlock::Rule => {
                let (rule_rect, _) = ui.allocate_exact_size(vec2(max_text_w, 9.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [pos2(rule_rect.min.x, rule_rect.center().y), pos2(rule_rect.max.x, rule_rect.center().y)],
                    Stroke::new(1.0, theme.border()),
                );
                ui.add_space(3.0);
            }
        }
    }
}
