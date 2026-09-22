//! Keyboard shortcut routing, text input, and command/editor event dispatch.

pub mod command;
pub mod editor;
pub mod global;

pub use global::window_shortcuts;

use crate::app::App;
use eframe::egui;

/// Processes all keyboard shortcuts and text typing.
/// Returns true if a character or text edit occurred in the editor.
pub fn handle_input(app: &mut App, ctx: &egui::Context, now: f64) -> bool {
    // 1. Check global shortcuts (window close, undo/redo, modal triggers, clipboard, tabs)
    if let Some(typed) = global::handle_global_shortcuts(app, ctx, now) {
        return typed;
    }

    let mut typed = false;

    // 2. Dispatch events for either command bar or active editor
    ctx.input(|i| {
        for ev in &i.events {
            match ev {
                egui::Event::Paste(s) => {
                    if app.in_command {
                        command::handle_command_paste(app, s, now);
                    } else if editor::handle_editor_paste(app, s, now) {
                        typed = true;
                    }
                }
                egui::Event::Text(s) => {
                    if app.in_command {
                        command::handle_command_text(app, s, now);
                    } else if editor::handle_editor_text(app, s, now) {
                        typed = true;
                    }
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if app.in_command {
                        command::handle_command_key(app, *key, *modifiers, now);
                    } else if editor::handle_editor_key(app, *key, *modifiers, now) {
                        typed = true;
                    }
                }
                _ => {}
            }
        }
    });

    typed
}
