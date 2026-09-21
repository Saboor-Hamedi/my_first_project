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
    let (ctrl_s, ctrl_n, ctrl_r, ctrl_d, ctrl_p, ctrl_comma, ctrl_b, escape) = ctx.input(|i| (
        i.modifiers.ctrl && i.key_pressed(egui::Key::S),
        i.modifiers.ctrl && i.key_pressed(egui::Key::N),
        i.modifiers.ctrl && i.key_pressed(egui::Key::R),
        i.modifiers.ctrl && i.key_pressed(egui::Key::D),
        i.modifiers.ctrl && (i.key_pressed(egui::Key::P) || i.key_pressed(egui::Key::F)),
        i.modifiers.ctrl && i.key_pressed(egui::Key::Comma),
        i.modifiers.ctrl && i.key_pressed(egui::Key::B),
        i.key_pressed(egui::Key::Escape),
    ));

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

    if ctrl_s {
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
        app.is_dirty = false;
        app.scroll_y = 0.0;
        app.set_status("Created new note", now);
        return false;
    }

    if ctrl_r {
        app.rename_open = true;
        app.rename_input = app.active_note_title.clone();
        app.rename_just_opened = true;
        return false;
    }

    if ctrl_d {
        app.delete_confirm_open = true;
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
                        app.last_char_time = now;
                    } else if s == ":" && app.mode == Mode::Normal && app.ed.row_col().1 == 0 {
                        app.in_command = true;
                        app.cmd_ed.clear();
                    } else {
                        for c in s.chars() {
                            if c == '\n' || c == '\r' {
                                continue;
                            }
                            app.ed.insert(c);
                            if app.mode == Mode::Normal {
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
                                execute_command(app, &cmd, now);
                            } else {
                                handle_mode_enter(app, now);
                                if app.mode == Mode::Normal {
                                    app.sound.play();
                                    typed = true;
                                }
                                app.last_char_time = now;
                            }
                        }
                        Backspace if modifiers.ctrl => {
                            if app.in_command {
                                app.cmd_ed.delete_word();
                            } else {
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
                                } else {
                                    app.cmd_ed.backspace();
                                }
                            } else {
                                app.ed.backspace();
                                app.is_dirty = true;
                                if app.mode == Mode::Normal {
                                    app.sound.play();
                                }
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
                        ArrowRight => {
                            if app.in_command {
                                app.cmd_ed.right();
                            } else {
                                app.ed.right();
                                typed = true;
                            }
                        }
                        ArrowUp => {
                            app.ed.up_visual(&app.visual_lines);
                            typed = true;
                        }
                        ArrowDown => {
                            app.ed.down_visual(&app.visual_lines);
                            typed = true;
                        }
                        Home => {
                            app.ed.home_visual(&app.visual_lines);
                            typed = true;
                        }
                        End => {
                            app.ed.end_visual(&app.visual_lines);
                            typed = true;
                        }
                        PageUp => {
                            app.ed.page_up_visual(&app.visual_lines, 10);
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
    use egui::{Key, ViewportCommand};
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
}
