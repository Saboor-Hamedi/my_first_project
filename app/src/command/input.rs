//! Command bar (`:`) input handling (typing, navigation, backspace, and execution).

use crate::app::App;
use crate::command::dispatch::execute_command;
use eframe::egui::{Key, Modifiers};

pub fn handle_command_paste(app: &mut App, s: &str, now: f64) {
    for c in s.chars() {
        if c != '\n' && c != '\r' {
            app.cmd_ed.insert(c);
        }
    }
    app.showcmd.set_command(&app.cmd_ed.text(), now);
    app.last_char_time = now;
}

pub fn handle_command_text(app: &mut App, s: &str, now: f64) {
    for c in s.chars() {
        app.cmd_ed.insert(c);
    }
    app.showcmd.set_command(&app.cmd_ed.text(), now);
    app.last_char_time = now;
}

pub fn handle_command_key(app: &mut App, key: Key, modifiers: Modifiers, now: f64) {
    match key {
        Key::Enter => {
            let cmd = app.cmd_ed.text();
            app.in_command = false;
            app.cmd_ed.clear();
            app.showcmd.record_action(&format!(":{}", cmd), now);
            execute_command(app, &cmd, now);
        }
        Key::Escape => {
            app.in_command = false;
            app.cmd_ed.clear();
            app.showcmd.clear();
        }
        Key::Backspace if modifiers.ctrl => {
            app.cmd_ed.delete_word();
            app.showcmd.set_command(&app.cmd_ed.text(), now);
        }
        Key::Backspace => {
            if app.cmd_ed.cur == 0 && !app.cmd_ed.has_selection() {
                app.in_command = false;
                app.showcmd.clear();
            } else {
                app.cmd_ed.backspace();
                app.showcmd.set_command(&app.cmd_ed.text(), now);
            }
        }
        Key::Delete if modifiers.ctrl => {
            app.cmd_ed.delete_word_forward();
            app.showcmd.set_command(&app.cmd_ed.text(), now);
        }
        Key::Delete => {
            app.cmd_ed.delete();
            app.showcmd.set_command(&app.cmd_ed.text(), now);
        }
        Key::ArrowLeft if modifiers.ctrl && modifiers.shift => {
            app.cmd_ed.word_left_select();
        }
        Key::ArrowLeft if modifiers.ctrl => {
            app.cmd_ed.word_left();
        }
        Key::ArrowLeft if modifiers.shift => {
            app.cmd_ed.left_select();
        }
        Key::ArrowLeft => {
            app.cmd_ed.left();
        }
        Key::ArrowRight if modifiers.ctrl && modifiers.shift => {
            app.cmd_ed.word_right_select();
        }
        Key::ArrowRight if modifiers.ctrl => {
            app.cmd_ed.word_right();
        }
        Key::ArrowRight if modifiers.shift => {
            app.cmd_ed.right_select();
        }
        Key::ArrowRight => {
            app.cmd_ed.right();
        }
        Key::Home => {
            app.cmd_ed.home();
        }
        Key::End => {
            app.cmd_ed.end();
        }
        Key::A if modifiers.ctrl => {
            app.cmd_ed.select_all();
        }
        Key::C if modifiers.ctrl => {
            if let Some(t) = app.cmd_ed.selected_text() {
                app.clipboard_text = Some(t.clone());
                app.vim.register = t.clone();
                app.vim.register_is_line = false;
                crate::input::global::set_win32_clipboard(&t);
            }
        }
        Key::V if modifiers.ctrl => {
            if let Some(text) = crate::input::global::get_clipboard_text(app) {
                handle_command_paste(app, &text, now);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_text_and_backspace() {
        let mut app = App::new();
        app.in_command = true;
        handle_command_text(&mut app, "set nu", 0.0);
        assert_eq!(app.cmd_ed.text(), "set nu");
        assert_eq!(app.showcmd.text, ":set nu");

        handle_command_key(&mut app, Key::Backspace, Modifiers::NONE, 0.0);
        assert_eq!(app.cmd_ed.text(), "set n");

        handle_command_key(&mut app, Key::Escape, Modifiers::NONE, 0.0);
        assert_eq!(app.in_command, false);
        assert_eq!(app.cmd_ed.text(), "");
    }

    #[test]
    fn test_command_colon_input_no_freeze() {
        let mut app = App::new();
        app.in_command = true;
        handle_command_text(&mut app, ":", 0.0);
        assert_eq!(app.cmd_ed.text(), ":");
        handle_command_text(&mut app, "w", 0.0);
        assert_eq!(app.cmd_ed.text(), ":w");
    }
}
