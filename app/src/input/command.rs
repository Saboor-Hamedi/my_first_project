//! Command bar (`:`) input handling (typing, navigation, backspace, and execution).

use crate::app::App;
use crate::commands::execute_command;
use eframe::egui::{Key, Modifiers};

pub fn handle_command_paste(app: &mut App, s: &str, now: f64) {
    for c in s.chars() {
        if c != '\n' && c != '\r' {
            app.cmd_ed.insert(c);
        }
    }
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
        Key::Backspace if modifiers.ctrl => {
            app.cmd_ed.delete_word();
            app.showcmd.set_command(&app.cmd_ed.text(), now);
        }
        Key::Backspace => {
            if app.cmd_ed.cur == 0 {
                app.in_command = false;
                app.showcmd.clear();
            } else {
                app.cmd_ed.backspace();
                app.showcmd.set_command(&app.cmd_ed.text(), now);
            }
        }
        Key::Delete => {
            app.cmd_ed.delete();
        }
        Key::ArrowLeft if modifiers.shift => {
            app.cmd_ed.left_select();
        }
        Key::ArrowLeft => {
            app.cmd_ed.left();
        }
        Key::ArrowRight if modifiers.shift => {
            app.cmd_ed.right_select();
        }
        Key::ArrowRight => {
            app.cmd_ed.right();
        }
        _ => {}
    }
}
