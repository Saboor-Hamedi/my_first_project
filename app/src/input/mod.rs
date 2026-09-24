//! Keyboard shortcut routing, text input, and command/editor event dispatch.

pub mod editor;
pub mod global;

pub use global::window_shortcuts;

use crate::app::App;
use crate::mode::Mode;
use eframe::egui;

/// Processes all keyboard shortcuts and text typing.
/// Returns true if a character or text edit occurred in the editor.
pub fn handle_input(app: &mut App, ctx: &egui::Context, now: f64) -> bool {
    // 1. Check global shortcuts (window close, undo/redo, modal triggers, clipboard, tabs)
    if let Some(typed) = global::handle_global_shortcuts(app, ctx, now) {
        return typed;
    }

    let mut typed = false;

    // 2. Dispatch events for either command bar, focused terminal, or active editor
    ctx.input(|i| {
        for ev in &i.events {
            if app.in_command {
                match ev {
                    egui::Event::Paste(s) => {
                        crate::command::input::handle_command_paste(app, s, now);
                        typed = true;
                    }
                    egui::Event::Text(s) => {
                        crate::command::input::handle_command_text(app, s, now);
                        typed = true;
                    }
                    egui::Event::Key { key, pressed: true, modifiers, .. } => {
                        crate::command::input::handle_command_key(app, *key, *modifiers, now);
                        typed = true;
                    }
                    _ => {}
                }
            } else if app.terminal_open && app.terminal_focused {
                if let egui::Event::Key { key: egui::Key::Escape, pressed: true, modifiers, .. } = ev {
                    if !modifiers.ctrl && !modifiers.shift && !modifiers.alt {
                        app.terminal_focused = false;
                        app.set_status("Editor focused (Ctrl+J to return to terminal)", now);
                        continue;
                    }
                }
                if let Some(ref mut pane) = app.term_pane {
                    pane.feed_event(ev, i.modifiers);
                }
            } else if app.mode == Mode::Normal && (app.show_welcome || app.open_notes.is_empty()) {
                // When on Welcome dashboard, hotkeys are handled directly by the dashboard or modals
                if app.editor_input_mode == crate::app::EditorInputMode::Vim {
                    if let egui::Event::Text(ref s) = ev {
                        if s == ":" {
                            app.in_command = true;
                            app.cmd_ed.clear();
                            app.cmd_selected_idx = 0;
                            app.cmd_navigated = false;
                            app.showcmd.set_command("", now);
                            typed = true;
                        }
                    }
                }
            } else {
                match ev {
                    egui::Event::Paste(s) => {
                        if editor::handle_editor_paste(app, s, now) {
                            typed = true;
                        }
                    }
                    egui::Event::Text(s) => {
                        if editor::handle_editor_text(app, s, now) {
                            typed = true;
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if editor::handle_editor_key(app, *key, *modifiers, now) {
                            typed = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    if typed && !app.open_notes.is_empty() {
        app.show_welcome = false;
    }

    typed
}
