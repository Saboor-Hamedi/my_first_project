//! Keyboard shortcut routing, text input, and review/decision state transitions.

use crate::app::App;
use crate::commands::execute_command;
use crate::mode::Mode;
use eframe::egui;

/// Processes all keyboard shortcuts and text typing.
/// Returns true if a character or text edit occurred in the editor.
pub fn handle_input(app: &mut App, ctx: &egui::Context, now: f64) -> bool {
    let mut typed = false;

    // Global Keyboard Shortcuts
    let (ctrl_s, ctrl_n, ctrl_r, ctrl_p, ctrl_comma, ctrl_b, escape) = ctx.input(|i| (
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::S),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::N),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::R),
        i.modifiers.ctrl && !i.modifiers.shift && (i.key_pressed(egui::Key::P) || i.key_pressed(egui::Key::F)),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Comma),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::B),
        i.key_pressed(egui::Key::Escape),
    ));

    // Delete Note (Ctrl+Shift+D or Ctrl+Shift+Delete)
    let ctrl_shift_d = ctx.input(|i| {
        (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::D))
            || (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Delete))
    });

    let ctrl_close = ctx.input(|i| {
        (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::W))
            || (i.modifiers.ctrl && i.key_pressed(egui::Key::Q))
    });
    if ctrl_close {
        if app.is_dirty && app.mode == Mode::Normal {
            app.quick_save_active_note(now);
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        return false;
    }

    // Undo / Redo
    let ctrl_z = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Z));
    let ctrl_redo = ctx.input(|i| {
        (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z))
            || (i.modifiers.ctrl && i.key_pressed(egui::Key::Y))
    });
    if ctrl_z {
        if app.in_command {
            app.cmd_ed.undo();
        } else {
            if app.ed.undo() {
                app.is_dirty = true;
                app.sound.play();
                app.set_status("Undo", now);
                return true;
            }
        }
        return false;
    }
    if ctrl_redo {
        if app.in_command {
            app.cmd_ed.redo();
        } else {
            if app.ed.redo() {
                app.is_dirty = true;
                app.sound.play();
                app.set_status("Redo", now);
                return true;
            }
        }
        return false;
    }

    // Move text left / right (Ctrl + [ to dedent left, Ctrl + ] to indent right)
    let ctrl_indent = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::CloseBracket));
    let ctrl_dedent = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::OpenBracket));
    if ctrl_indent {
        if !app.in_command {
            app.ed.indent_line();
            app.is_dirty = true;
            app.sound.play();
            app.set_status("Indented line (Ctrl + ])", now);
            return true;
        }
    }
    if ctrl_dedent {
        if !app.in_command {
            app.ed.dedent_line();
            app.is_dirty = true;
            app.sound.play();
            app.set_status("Dedented line (Ctrl + [)", now);
            return true;
        }
    }

    // Tab / Shift+Tab — polled directly so egui focus-cycling can never intercept it.
    let (tab_pressed, shift_tab_pressed) = ctx.input(|i| {
        let tab = !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::Tab);
        (tab && !i.modifiers.shift, tab && i.modifiers.shift)
    });
    if (tab_pressed || shift_tab_pressed) && !app.in_command && (app.mode == Mode::Normal || app.mode == Mode::Doc) {
        use crate::app::EditorInputMode;
        let modifiers = if shift_tab_pressed {
            let mut m = egui::Modifiers::default();
            m.shift = true;
            m
        } else {
            egui::Modifiers::default()
        };
        if app.editor_input_mode == EditorInputMode::Vim {
            app.vim.handle_key(&mut app.ed, &app.visual_lines, egui::Key::Tab, modifiers);
        } else if app.editor_input_mode == EditorInputMode::Hybrid {
            app.hybrid.handle_key(&mut app.ed, egui::Key::Tab, modifiers);
        } else {
            if shift_tab_pressed { app.ed.dedent(); } else { app.ed.indent(); }
        }
        if app.mode == Mode::Normal {
            app.is_dirty = true;
        }
        app.sound.play();
        app.last_char_time = now;
        return true;
    }

    // Select All (Ctrl+A)
    let ctrl_a = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::A));
    if ctrl_a {
        if app.in_command {
            app.cmd_ed.select_all();
        } else {
            app.ed.select_all();
            return true;
        }
        return false;
    }

    // Clipboard Copy (Ctrl+C)
    let ctrl_c = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C));
    if ctrl_c {
        let text = if app.in_command {
            app.cmd_ed.selected_text()
        } else {
            app.ed.selected_text()
        };
        if let Some(t) = text {
            ctx.copy_text(t);
            app.set_status("Copied selection", now);
        }
        return false;
    }

    // Clipboard Cut (Ctrl+X)
    let ctrl_x = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::X));
    if ctrl_x {
        let text = if app.in_command {
            let t = app.cmd_ed.selected_text();
            app.cmd_ed.delete_selection();
            t
        } else {
            let t = app.ed.selected_text();
            if app.ed.delete_selection() {
                app.is_dirty = true;
                app.sound.play();
                app.set_status("Cut selection", now);
            }
            t
        };
        if let Some(t) = text {
            ctx.copy_text(t);
            return true;
        }
        return false;
    }

    if ctrl_s {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files are read-only (changes not saved).", now);
            return false;
        }
        app.quick_save_active_note(now);
        return false;
    }

    if ctrl_n {
        if app.is_dirty && app.mode == Mode::Normal {
            app.quick_save_active_note(now);
        }
        app.active_note_id = None;
        app.active_note_title = "Untitled Note".to_string();
        app.ed.clear();
        app.mode = Mode::Normal;
        app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
        app.is_dirty = false;
        app.scroll_y = 0.0;
        app.set_status("Created new note", now);
        return false;
    }

    if ctrl_r {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files are read-only and cannot be renamed.", now);
            return false;
        }
        app.rename_open = true;
        app.rename_input = app.active_note_title.clone();
        app.rename_just_opened = true;
        return false;
    }

    if ctrl_shift_d {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files cannot be deleted.", now);
            return false;
        }
        app.delete_confirm_open = true;
        app.delete_just_opened = true;
        return false;
    }

    if ctrl_p {
        app.search_open = true;
        app.search_query.clear();
        app.search_selected = 0;
        app.search_just_opened = true;
        app.update_search_results();
        return false;
    }

    if ctrl_comma {
        app.settings_open = !app.settings_open;
        return false;
    }

    if ctrl_b {
        app.sidebar_open = !app.sidebar_open;
        return false;
    }

    if app.delete_confirm_open {
        if escape {
            app.delete_confirm_open = false;
        }
        return false;
    }

    if app.search_open {
        if escape {
            app.search_open = false;
        }
        return false;
    }

    if app.rename_open {
        if escape {
            app.rename_open = false;
        }
        return false;
    }

    if app.settings_open {
        if escape {
            app.settings_open = false;
        }
        return false;
    }

    if escape {
        app.showcmd.clear();
        if app.editor_input_mode == crate::app::EditorInputMode::Vim && app.vim.mode != crate::vim::VimSubMode::Normal {
            if app.vim.is_searching() {
                app.vim.search.cancel(&mut app.ed);
            }
            app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
            return true;
        }
        if app.ed.has_selection() {
            app.ed.clear_selection();
            return true;
        }
        if app.sidebar_open {
            app.sidebar_open = false;
            return false;
        }
        if app.in_command {
            app.in_command = false;
            app.cmd_ed.clear();
            return false;
        }
        if app.mode != Mode::Normal {
            app.mode = Mode::Normal;
            return false;
        }
    }

    // Suppress egui tab focus navigation so Tab key reaches the editor
    ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
    });

    // Editor and Command input processing
    ctx.input(|i| {
        for ev in &i.events {
            match ev {
                egui::Event::Paste(s) => {
                    if app.in_command {
                        for c in s.chars() {
                            if c != '\n' && c != '\r' {
                                app.cmd_ed.insert(c);
                            }
                        }
                        app.last_char_time = now;
                    } else if app.mode == Mode::Normal {
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
                        typed = true;
                        app.last_char_time = now;
                        app.is_dirty = true;
                    }
                }
                egui::Event::Text(s) => {
                    if app.in_command {
                        for c in s.chars() {
                            app.cmd_ed.insert(c);
                        }
                        app.showcmd.set_command(&app.cmd_ed.text(), now);
                        app.last_char_time = now;
                    } else if app.editor_input_mode == crate::app::EditorInputMode::Vim && (app.mode == Mode::Normal || app.mode == Mode::Doc) {
                        if s == ":" && app.vim.mode == crate::vim::VimSubMode::Normal {
                            app.in_command = true;
                            app.cmd_ed.clear();
                            app.showcmd.set_command("", now);
                        } else if app.mode == Mode::Doc {
                            for c in s.chars() {
                                if c == '\r' || c == '\n' {
                                    continue;
                                }
                                match c {
                                    'i' | 'I' | 'a' | 'A' | 'o' | 'O' | 's' | 'S' | 'c' | 'C' | 'r' | 'R' | 'd' | 'D' | 'x' | 'X' | 'p' | 'P' | 'u' => {
                                        app.set_status("📖 Documentation is read-only (navigate with j, k, w, b, gg, G, or /)", now);
                                        continue;
                                    }
                                    _ => {}
                                }
                                app.vim.pending_keys_time = now;
                                if app.vim.handle_char(&mut app.ed, &app.visual_lines, c) {
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
                                if app.vim.mode == crate::vim::VimSubMode::Insert {
                                    app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
                                }
                            }
                        } else {
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
                                } else if app.vim.mode == crate::vim::VimSubMode::Insert {
                                    app.ed.insert(c);
                                    typed = true;
                                    app.sound.play();
                                    app.last_char_time = now;
                                    app.is_dirty = true;
                                }
                            }
                        }
                    } else if s == ":" && (app.mode == Mode::Normal || app.mode == Mode::Doc) && app.ed.row_col().1 == 0 {
                        app.in_command = true;
                        app.cmd_ed.clear();
                    } else if app.mode == Mode::Doc {
                        app.set_status("📖 Documentation is read-only.", now);
                    } else {
                        for c in s.chars() {
                            if c == '\n' || c == '\r' {
                                continue;
                            }
                            if app.editor_input_mode == crate::app::EditorInputMode::Hybrid && app.hybrid.handle_char(&mut app.ed, c) {
                                app.sound.play();
                            } else {
                                app.ed.insert(c);
                                app.sound.play();
                            }
                        }
                        typed = true;
                        app.last_char_time = now;
                        app.is_dirty = true;
                    }
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if !app.in_command && (app.mode == Mode::Normal || app.mode == Mode::Doc) {
                        if app.editor_input_mode == crate::app::EditorInputMode::Vim {
                            app.vim.pending_keys_time = now;
                            if app.vim.handle_key(&mut app.ed, &app.visual_lines, *key, *modifiers) {
                                typed = true;
                                app.sound.play();
                                app.last_char_time = now;
                                if app.mode == Mode::Normal {
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
                                    } else if *key == egui::Key::Escape {
                                        app.showcmd.clear();
                                    }
                                }
                                continue;
                            }
                        } else if app.editor_input_mode == crate::app::EditorInputMode::Hybrid {
                            if app.hybrid.handle_key(&mut app.ed, *key, *modifiers) {
                                typed = true;
                                app.sound.play();
                                app.last_char_time = now;
                                if app.mode == Mode::Normal {
                                    app.is_dirty = true;
                                }
                                continue;
                            }
                        }
                    }

                    use egui::Key::*;
                    match key {
                        Enter if modifiers.ctrl => {
                            if !app.in_command && app.mode == Mode::Normal {
                                app.ed.insert_line_below();
                                app.is_dirty = true;
                                app.sound.play();
                                typed = true;
                                app.last_char_time = now;
                            }
                        }
                        Enter => {
                            if app.in_command {
                                let cmd = app.cmd_ed.text();
                                app.in_command = false;
                                app.cmd_ed.clear();
                                app.showcmd.record_action(&format!(":{}", cmd), now);
                                execute_command(app, &cmd, now);
                            } else if app.mode == Mode::Normal {
                                handle_mode_enter(app, now);
                                app.sound.play();
                                typed = true;
                                app.last_char_time = now;
                            }
                        }
                        Backspace if modifiers.ctrl => {
                            if app.in_command {
                                app.cmd_ed.delete_word();
                                app.showcmd.set_command(&app.cmd_ed.text(), now);
                            } else if app.mode != Mode::Doc {
                                app.ed.delete_word();
                                app.is_dirty = true;
                                if app.mode == Mode::Normal {
                                    app.sound.play();
                                }
                                typed = true;
                            }
                        }
                        Backspace => {
                            if app.in_command {
                                if app.cmd_ed.cur == 0 {
                                    app.in_command = false;
                                    app.showcmd.clear();
                                } else {
                                    app.cmd_ed.backspace();
                                    app.showcmd.set_command(&app.cmd_ed.text(), now);
                                }
                            } else if app.mode != Mode::Doc {
                                app.ed.backspace();
                                app.is_dirty = true;
                                if app.mode == Mode::Normal {
                                    app.sound.play();
                                }
                                typed = true;
                            }
                        }
                        Delete => {
                            if app.in_command {
                                app.cmd_ed.delete();
                            } else if app.mode != Mode::Doc {
                                app.ed.delete();
                                app.is_dirty = true;
                                if app.mode == Mode::Normal {
                                    app.sound.play();
                                }
                                typed = true;
                            }
                        }
                        ArrowLeft if modifiers.shift => {
                            if app.in_command {
                                app.cmd_ed.left_select();
                            } else {
                                app.ed.left_select();
                                typed = true;
                            }
                        }
                        ArrowLeft => {
                            if app.in_command {
                                app.cmd_ed.left();
                            } else {
                                app.ed.left();
                                typed = true;
                            }
                        }
                        ArrowRight if modifiers.shift => {
                            if app.in_command {
                                app.cmd_ed.right_select();
                            } else {
                                app.ed.right_select();
                                typed = true;
                            }
                        }
                        ArrowRight => {
                            if app.in_command {
                                app.cmd_ed.right();
                            } else {
                                app.ed.right();
                                typed = true;
                            }
                        }
                        ArrowUp if modifiers.shift => {
                            app.ed.up_visual_select(&app.visual_lines);
                            typed = true;
                        }
                        ArrowUp => {
                            app.ed.up_visual(&app.visual_lines);
                            typed = true;
                        }
                        ArrowDown if modifiers.shift => {
                            app.ed.down_visual_select(&app.visual_lines);
                            typed = true;
                        }
                        ArrowDown => {
                            app.ed.down_visual(&app.visual_lines);
                            typed = true;
                        }
                        Home if modifiers.shift => {
                            app.ed.home_visual_select(&app.visual_lines);
                            typed = true;
                        }
                        Home => {
                            app.ed.home_visual(&app.visual_lines);
                            typed = true;
                        }
                        End if modifiers.shift => {
                            app.ed.end_visual_select(&app.visual_lines);
                            typed = true;
                        }
                        End => {
                            app.ed.end_visual(&app.visual_lines);
                            typed = true;
                        }
                        PageUp if modifiers.shift => {
                            app.ed.page_up_visual_select(&app.visual_lines, 10);
                            typed = true;
                        }
                        PageUp => {
                            app.ed.page_up_visual(&app.visual_lines, 10);
                            typed = true;
                        }
                        PageDown if modifiers.shift => {
                            app.ed.page_down_visual_select(&app.visual_lines, 10);
                            typed = true;
                        }
                        PageDown => {
                            app.ed.page_down_visual(&app.visual_lines, 10);
                            typed = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    typed
}

pub fn handle_mode_enter(app: &mut App, _now: f64) {
    if app.mode == Mode::Normal {
        app.ed.insert('\n');
        app.is_dirty = true;
    }
}

pub fn window_shortcuts(ctx: &egui::Context) {
    use egui::{CursorIcon, Key, ResizeDirection, ViewportCommand};
    let (drag, f11, quit, is_fs) = ctx.input(|i| (
        i.modifiers.alt && i.pointer.primary_pressed(),
        i.key_pressed(Key::F11),
        (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(Key::W))
            || (i.modifiers.ctrl && i.key_pressed(Key::Q)),
        i.viewport().fullscreen.unwrap_or(false),
    ));
    if drag {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if f11 {
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!is_fs));
    }
    if quit {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    // Borderless window edge & corner resize grips (6px borders)
    if !is_fs {
        let screen = ctx.screen_rect();
        if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
            let margin = 6.0;
            let on_right = pos.x >= screen.max.x - margin && pos.x <= screen.max.x + 2.0;
            let on_bottom = pos.y >= screen.max.y - margin && pos.y <= screen.max.y + 2.0;
            let on_left = pos.x <= screen.min.x + margin && pos.x >= screen.min.x - 2.0;

            let resize_dir = if on_right && on_bottom {
                Some((ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast))
            } else if on_left && on_bottom {
                Some((ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest))
            } else if on_right {
                Some((ResizeDirection::East, CursorIcon::ResizeEast))
            } else if on_bottom {
                Some((ResizeDirection::South, CursorIcon::ResizeSouth))
            } else if on_left {
                Some((ResizeDirection::West, CursorIcon::ResizeWest))
            } else {
                None
            };

            if let Some((dir, cursor)) = resize_dir {
                ctx.set_cursor_icon(cursor);
                if ctx.input(|i| i.pointer.primary_pressed()) {
                    ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
                }
            }
        }
    }
}
