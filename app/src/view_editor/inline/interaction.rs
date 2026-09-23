//! Interactive mouse clicks, drag selection, and checkbox toggling for the inline editor.

use super::types::InlineEditorLayout;
use crate::editor::Editor;
use eframe::egui::{Pos2, Rect, Ui};

/// Handles mouse interactions (primary click, drag selection, and checkbox toggle).
pub fn handle_inline_mouse_interaction(
    ui: &Ui,
    editor_rect: Rect,
    ed_origin: Pos2,
    layout: &InlineEditorLayout,
    ed: &mut Editor,
    block_interaction: bool,
    sound: &mut crate::sound::SoundEngine,
    is_dirty: &mut bool,
) -> bool {
    if block_interaction || !ui.rect_contains_pointer(editor_rect) {
        return false;
    }

    let mut action_taken = false;

    // 1. Check for checkbox toggle clicks
    if ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            for line in &layout.lines {
                if let Some(box_rect) = line.checkbox_rect {
                    // Check if clicked inside or near the checkbox
                    let expanded_rect = box_rect.expand(4.0);
                    if expanded_rect.contains(pos) {
                        if let super::types::InlineLineKind::TaskItem { check_char_idx, checked } = line.kind {
                            let target_idx = line.char_start + check_char_idx;
                            if target_idx < ed.buf.len() {
                                ed.save_undo_snapshot();
                                ed.buf[target_idx] = if checked { ' ' } else { 'x' };
                                *is_dirty = true;
                                sound.play();
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Double-click to select word
    if ui.input(|i| i.pointer.button_double_clicked(eframe::egui::PointerButton::Primary)) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let clicked_char = layout.char_at_pos(pos, ed_origin);
            ed.select_word_at(clicked_char);
            action_taken = true;
            return action_taken;
        }
    }

    // 3. Direct click / Shift+Click to move cursor or expand selection
    if ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let clicked_char = layout.char_at_pos(pos, ed_origin);
            let is_shift = ui.input(|i| i.modifiers.shift);
            if is_shift {
                if ed.selection.is_none() {
                    ed.selection = Some(ed.cur);
                }
                ed.cur = clicked_char;
            } else {
                ed.cur = clicked_char;
                ed.selection = None;
            }
            action_taken = true;
        }
    }

    // 4. Mouse drag selection
    if ui.input(|i| i.pointer.is_decidedly_dragging()) && ui.input(|i| i.pointer.primary_down()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let drag_char = layout.char_at_pos(pos, ed_origin);
            if ed.selection.is_none() {
                ed.selection = Some(ed.cur);
            }
            ed.cur = drag_char;
            action_taken = true;
        }
    }

    action_taken
}
