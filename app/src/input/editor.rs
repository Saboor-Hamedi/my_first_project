//! Editor mode (Normal, Vim, Hybrid, Doc) typing, paste, and navigation routing.

use crate::app::{App, EditorInputMode};
use crate::mode::Mode;
use crate::vim::VimSubMode;
use eframe::egui::{Key, Modifiers};

pub fn handle_mode_enter(app: &mut App, _now: f64) {
    if app.mode == Mode::Normal {
        // Auto-indent and auto-continuing lists apply in INSERT mode
        if app.editor_input_mode == EditorInputMode::Vim && app.vim.mode != VimSubMode::Insert {
            return;
        }
        app.ed.handle_enter();
        app.is_dirty = true;
    }
}

pub fn handle_editor_paste(app: &mut App, s: &str, now: f64) -> bool {
    if app.mode == Mode::Normal {
        for c in s.chars() {
            if c == '\r' {
                continue;
            }
            app.ed.insert(c);
        }
        // Play ONE sound for the entire paste — not per character.
        // Pasting 20k chars must not fire the audio engine 20k times.
        if !s.is_empty() {
            app.sound.play();
        }
        app.last_char_time = now;
        app.is_dirty = true;
        return true;
    }
    false
}

pub fn handle_editor_text(app: &mut App, s: &str, now: f64) -> bool {
    if app.editor_input_mode == EditorInputMode::Vim && (app.mode == Mode::Normal || app.mode == Mode::Doc) {
        if s == ":" && app.vim.mode == VimSubMode::Normal {
            app.in_command = true;
            app.cmd_ed.clear();
            app.showcmd.set_command("", now);
            return false;
        } else if app.mode == Mode::Doc {
            let mut typed = false;
            for c in s.chars() {
                if c == '\r' || c == '\n' {
                    continue;
                }
                // Only block editing chars when Vim is in Normal sub-mode.
                // In Visual, Search, or operator-pending modes, chars like 'i', 'a', 'd'
                // are part of text object sequences (vi', va", diw) and must pass through.
                let is_vim_normal = app.vim.mode == VimSubMode::Normal;
                if is_vim_normal && !app.vim.is_searching() {
                    match c {
                        'i' | 'I' | 'a' | 'A' | 'o' | 'O' | 's' | 'S' | 'c' | 'C' | 'r' | 'R' | 'd' | 'D' | 'x' | 'X' | 'p' | 'P' | 'u' => {
                            app.set_status("📖 Documentation is read-only (navigate with j, k, w, b, gg, G, or /)", now);
                            continue;
                        }
                        _ => {}
                    }
                }
                app.vim.pending_keys_time = now;
                if app.vim.handle_char(&mut app.doc_ed, &app.visual_lines, c) {
                    typed = true;
                    app.sound.play();
                    app.last_char_time = now;
                    if app.vim.is_searching() {
                        let sym = if app.vim.search.backward { "?" } else { "/" };
                        app.showcmd.set_search(sym, &app.vim.search.query, now);
                    } else if let Some(action_str) = app.vim.last_completed_action.take() {
                        app.showcmd.record_action(&action_str, now);
                    } else {
                        let pending_after = app.vim.pending_keys();
                        if !pending_after.is_empty() {
                            app.showcmd.set_pending(pending_after, now);
                        }
                    }
                }
                if app.vim.mode == VimSubMode::Insert {
                    app.vim.set_mode(VimSubMode::Normal, &mut app.doc_ed);
                }
            }
            return typed;

        } else {
            let mut typed = false;
            for c in s.chars() {
                if c == '\r' || c == '\n' {
                    continue;
                }
                app.vim.pending_keys_time = now;
                if app.vim.handle_char(&mut app.ed, &app.visual_lines, c) {
                    typed = true;
                    app.sound.play();
                    app.last_char_time = now;
                    app.is_dirty = true;
                    if app.vim.is_searching() {
                        let sym = if app.vim.search.backward { "?" } else { "/" };
                        app.showcmd.set_search(sym, &app.vim.search.query, now);
                    } else if let Some(action_str) = app.vim.last_completed_action.take() {
                        app.showcmd.record_action(&action_str, now);
                    } else {
                        let pending_after = app.vim.pending_keys();
                        if !pending_after.is_empty() {
                            app.showcmd.set_pending(pending_after, now);
                        }
                    }
                } else if app.vim.mode == VimSubMode::Insert {
                    app.ed.insert(c);
                    typed = true;
                    app.sound.play();
                    app.last_char_time = now;
                    app.is_dirty = true;
                }
            }
            return typed;
        }
    } else if s == ":" && (app.mode == Mode::Normal || app.mode == Mode::Doc) && app.ed.row_col().1 == 0 {
        app.in_command = true;
        app.cmd_ed.clear();
        return false;
    } else if app.mode == Mode::Doc {
        app.set_status("📖 Documentation is read-only.", now);
        return false;
    } else {
        for c in s.chars() {
            if c == '\n' || c == '\r' {
                continue;
            }
            if app.editor_input_mode == EditorInputMode::Hybrid && app.hybrid.handle_char(&mut app.ed, c) {
                app.sound.play();
            } else {
                app.ed.insert(c);
                app.sound.play();
            }
        }
        app.last_char_time = now;
        app.is_dirty = true;
        return true;
    }
}

pub fn handle_editor_key(app: &mut App, key: Key, modifiers: Modifiers, now: f64) -> bool {
    let is_doc = app.mode == Mode::Doc;

    // Mode-specific engines (Vim / Hybrid)
    if app.mode == Mode::Normal || is_doc {
        let target_ed = if is_doc { &mut app.doc_ed } else { &mut app.ed };
        if app.editor_input_mode == EditorInputMode::Vim {
            app.vim.pending_keys_time = now;
            if app.vim.handle_key(target_ed, &app.visual_lines, key, modifiers) {
                app.sound.play();
                app.last_char_time = now;
                if !is_doc {
                    app.is_dirty = true;
                }
                if app.vim.is_searching() {
                    let sym = if app.vim.search.backward { "?" } else { "/" };
                    app.showcmd.set_search(sym, &app.vim.search.query, now);
                } else if let Some(action_str) = app.vim.last_completed_action.take() {
                    app.showcmd.record_action(&action_str, now);
                } else {
                    let pending_after = app.vim.pending_keys();
                    if !pending_after.is_empty() {
                        app.showcmd.set_pending(pending_after, now);
                    } else if key == Key::Escape {
                        app.showcmd.clear();
                    }
                }
                return true;
            }
        } else if app.editor_input_mode == EditorInputMode::Hybrid && app.hybrid.handle_key(target_ed, key, modifiers) {
            app.sound.play();
            app.last_char_time = now;
            if !is_doc {
                app.is_dirty = true;
            }
            return true;
        }
    }

    let target_ed = if is_doc { &mut app.doc_ed } else { &mut app.ed };

    use Key::*;
    match key {
        Enter if modifiers.ctrl => {
            if !is_doc {
                target_ed.insert_line_below();
                app.is_dirty = true;
                app.sound.play();
                app.last_char_time = now;
                return true;
            }
        }
        Enter => {
            if !is_doc {
                handle_mode_enter(app, now);
                app.sound.play();
                app.last_char_time = now;
                return true;
            }
        }
        Backspace if modifiers.ctrl => {
            if !is_doc {
                target_ed.delete_word();
                app.is_dirty = true;
                app.sound.play();
                return true;
            }
        }
        Backspace => {
            if !is_doc {
                target_ed.backspace();
                app.is_dirty = true;
                app.sound.play();
                return true;
            }
        }
        Delete => {
            if !is_doc {
                target_ed.delete();
                app.is_dirty = true;
                app.sound.play();
                return true;
            }
        }
        ArrowLeft if modifiers.shift => {
            target_ed.left_select();
            return true;
        }
        ArrowLeft => {
            target_ed.left();
            return true;
        }
        ArrowRight if modifiers.shift => {
            target_ed.right_select();
            return true;
        }
        ArrowRight => {
            target_ed.right();
            return true;
        }
        ArrowUp if modifiers.shift => {
            target_ed.up_visual_select(&app.visual_lines);
            return true;
        }
        ArrowUp => {
            target_ed.up_visual(&app.visual_lines);
            return true;
        }
        ArrowDown if modifiers.shift => {
            target_ed.down_visual_select(&app.visual_lines);
            return true;
        }
        ArrowDown => {
            target_ed.down_visual(&app.visual_lines);
            return true;
        }
        Home if modifiers.shift => {
            target_ed.home_visual_select(&app.visual_lines);
            return true;
        }
        Home => {
            target_ed.home_visual(&app.visual_lines);
            return true;
        }
        End if modifiers.shift => {
            target_ed.end_visual_select(&app.visual_lines);
            return true;
        }
        End => {
            target_ed.end_visual(&app.visual_lines);
            return true;
        }
        PageUp if modifiers.shift => {
            target_ed.page_up_visual_select(&app.visual_lines, 10);
            return true;
        }
        PageUp => {
            target_ed.page_up_visual(&app.visual_lines, 10);
            return true;
        }
        PageDown if modifiers.shift => {
            target_ed.page_down_visual_select(&app.visual_lines, 10);
            return true;
        }
        PageDown => {
            target_ed.page_down_visual(&app.visual_lines, 10);
            return true;
        }
        _ => {}
    }

    false
}
