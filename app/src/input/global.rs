//! Global keyboard shortcuts, clipboard actions, and window resize/drag handlers.

use crate::app::App;
use crate::mode::Mode;
use eframe::egui;

/// Processes global shortcuts (saving, note creation, modals, clipboard, undo/redo).
/// Returns `Some(typed)` if a global shortcut fully handled the frame, or `None` to continue to typing.
pub fn handle_global_shortcuts(app: &mut App, ctx: &egui::Context, now: f64) -> Option<bool> {
    // When Vim search is active, bypass ALL global shortcuts so every keystroke
    // flows through as a Text event into the search buffer (fixes missing chars).
    if app.editor_input_mode == crate::app::EditorInputMode::Vim && app.vim.is_searching() {
        return None;
    }

    // Global Keyboard Shortcuts
    let (ctrl_s, ctrl_n, ctrl_r, ctrl_p, ctrl_comma, ctrl_b, escape, ctrl_backslash) = ctx.input(|i| (
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::S),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::N),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::R),
        i.modifiers.ctrl && !i.modifiers.shift && (i.key_pressed(egui::Key::P) || i.key_pressed(egui::Key::F)),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Comma),
        i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::B),
        i.key_pressed(egui::Key::Escape),
        (i.modifiers.ctrl && (i.key_pressed(egui::Key::Backslash) || i.key_pressed(egui::Key::Pipe)))
            || (i.modifiers.alt && i.key_pressed(egui::Key::Backslash)),
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
        return Some(false);
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
        } else if app.ed.undo() {
            app.is_dirty = true;
            app.sound.play();
            app.set_status("Undo", now);
            return Some(true);
        }
        return Some(false);
    }
    if ctrl_redo {
        if app.in_command {
            app.cmd_ed.redo();
        } else if app.ed.redo() {
            app.is_dirty = true;
            app.sound.play();
            app.set_status("Redo", now);
            return Some(true);
        }
        return Some(false);
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
            return Some(true);
        }
    }
    if ctrl_dedent {
        if !app.in_command {
            app.ed.dedent_line();
            app.is_dirty = true;
            app.sound.play();
            app.set_status("Dedented line (Ctrl + [)", now);
            return Some(true);
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
        } else if shift_tab_pressed {
            app.ed.dedent();
        } else {
            app.ed.indent();
        }
        if app.mode == Mode::Normal {
            app.is_dirty = true;
        }
        app.sound.play();
        app.last_char_time = now;
        return Some(true);
    }

    // Select All (Ctrl+A)
    let ctrl_a = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::A));
    if ctrl_a {
        if app.in_command {
            app.cmd_ed.select_all();
        } else {
            app.ed.select_all();
            return Some(true);
        }
        return Some(false);
    }

    // Clipboard Copy (Ctrl+C)
    let ctrl_c = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::C));
    if ctrl_c {
        let text = if app.in_command {
            app.cmd_ed.selected_text()
        } else if app.search_open {
            Some(app.search_query.clone())
        } else if app.rename_open {
            Some(app.rename_input.clone())
        } else if app.mode == Mode::Doc {
            app.doc_ed.selected_text()
        } else {
            app.ed.selected_text()
        };
        if let Some(t) = text {
            app.clipboard_text = Some(t.clone());
            app.vim.register = t.clone();
            app.vim.register_is_line = false;
            set_win32_clipboard(&t);
            ctx.copy_text(t);
            app.set_status("Copied selection", now);
        }
        return Some(false);
    }

    // Clipboard Cut (Ctrl+X)
    let ctrl_x = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::X));
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
            app.clipboard_text = Some(t.clone());
            app.vim.register = t.clone();
            app.vim.register_is_line = false;
            set_win32_clipboard(&t);
            ctx.copy_text(t);
            return Some(true);
        }
        return Some(false);
    }

    // Clipboard Paste (Ctrl+V)
    let ctrl_v = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::V));
    if ctrl_v {
        if let Some(text) = get_clipboard_text(app) {
            if app.in_command {
                crate::input::command::handle_command_paste(app, &text, now);
                return Some(true);
            } else if app.search_open {
                app.search_query.push_str(&text);
                app.search_selected = 0;
                app.update_search_results();
                return Some(true);
            } else if app.rename_open {
                app.rename_input.push_str(&text);
                return Some(true);
            } else if app.mode == Mode::Normal {
                if crate::input::editor::handle_editor_paste(app, &text, now) {
                    return Some(true);
                }
            }
        }
        return Some(false);
    }

    if ctrl_s {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files are read-only (changes not saved).", now);
            return Some(false);
        }
        app.quick_save_active_note(now);
        return Some(false);
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
        return Some(false);
    }

    if ctrl_r {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files are read-only and cannot be renamed.", now);
            return Some(false);
        }
        app.rename_open = true;
        app.rename_input = app.active_note_title.clone();
        app.rename_just_opened = true;
        return Some(false);
    }

    if ctrl_shift_d {
        if app.mode == Mode::Doc {
            app.set_status("Documentation files cannot be deleted.", now);
            return Some(false);
        }
        app.delete_confirm_open = true;
        app.delete_just_opened = true;
        return Some(false);
    }

    if ctrl_p {
        app.search_open = true;
        app.search_query.clear();
        app.search_selected = 0;
        app.search_just_opened = true;
        app.update_search_results();
        return Some(false);
    }

    if ctrl_comma {
        app.settings_open = !app.settings_open;
        return Some(false);
    }

    if ctrl_b {
        app.sidebar_open = !app.sidebar_open;
        return Some(false);
    }

    if ctrl_backslash {
        app.preview_open = !app.preview_open;
        let val = if app.preview_open { "true" } else { "false" };
        let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
            key: "preview".into(),
            val: val.into(),
        });
        let msg = if app.preview_open {
            "Markdown Live Preview ON (Ctrl + \\ to toggle, drag center knob)"
        } else {
            "Markdown Live Preview OFF (Ctrl + \\)"
        };
        app.set_status(msg, now);
        return Some(false);
    }

    if app.delete_confirm_open {
        if escape {
            app.delete_confirm_open = false;
        }
        return Some(false);
    }

    if app.search_open {
        if escape {
            app.search_open = false;
        }
        return Some(false);
    }

    if app.rename_open {
        if escape {
            app.rename_open = false;
        }
        return Some(false);
    }

    if app.settings_open {
        if escape {
            app.settings_open = false;
        }
        return Some(false);
    }

    if app.help_open {
        if escape {
            app.help_open = false;
        }
        return Some(false);
    }

    // Help & Guidance Center (F1 or Ctrl+H)
    let trigger_help = ctx.input(|i| {
        i.key_pressed(egui::Key::F1) || (i.modifiers.ctrl && i.key_pressed(egui::Key::H))
    });
    if trigger_help && !app.in_command {
        app.help_open = !app.help_open;
        app.help_just_opened = app.help_open;
        return Some(false);
    }

    if escape {
        app.showcmd.clear();
        app.vim.search.clear_matches();
        if app.editor_input_mode == crate::app::EditorInputMode::Vim && app.vim.mode != crate::vim::VimSubMode::Normal {
            if app.vim.is_searching() {
                app.vim.search.cancel(&mut app.ed);
            }
            app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
            return Some(true);
        }
        if app.ed.has_selection() {
            app.ed.clear_selection();
            return Some(true);
        }
        if app.sidebar_open {
            app.sidebar_open = false;
            return Some(false);
        }
        if app.in_command {
            app.in_command = false;
            app.cmd_ed.clear();
            return Some(false);
        }
        if app.mode == Mode::ScanReport || app.mode == Mode::ScanHistory {
            app.mode = app.prev_mode_before_scan;
            return Some(false);
        }
        if app.mode != Mode::Normal {
            app.mode = Mode::Normal;
            return Some(false);
        }
    }

    // Scan views keyboard navigation
    if app.mode == Mode::ScanReport && !app.in_command {
        let q_pressed = ctx.input(|i| i.key_pressed(egui::Key::Q));
        if q_pressed {
            app.mode = app.prev_mode_before_scan;
            return Some(false);
        }
    }

    if app.mode == Mode::ScanHistory && !app.in_command {
        let up_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K));
        let down_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J));
        let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));

        if up_pressed {
            if app.scan_history_selected > 0 {
                app.scan_history_selected -= 1;
            }
            return Some(false);
        }
        if down_pressed {
            if app.scan_history_selected + 1 < app.past_scans.len() {
                app.scan_history_selected += 1;
            }
            return Some(false);
        }
        if enter_pressed {
            if let Some(record) = app.past_scans.get(app.scan_history_selected) {
                let findings: Vec<webscan::Finding> = serde_json::from_str(&record.findings_json).unwrap_or_default();
                let result = webscan::ScanResult {
                    url: record.url.clone(),
                    status_code: 200,
                    response_time_ms: 0,
                    tls: None,
                    server_header: None,
                    page_size_bytes: 0,
                    note: record.note.clone(),
                    findings,
                };
                app.active_scan_result = Some(result);
                app.active_scan_error = None;
                app.scan_report_scroll_y = 0.0;
                app.mode = Mode::ScanReport;
            }
            return Some(false);
        }
    }

    // Suppress egui tab focus navigation so Tab key reaches the editor
    ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
    });

    None
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

#[cfg(target_os = "windows")]
pub fn set_win32_clipboard(text: &str) {
    #[link(name = "user32")]
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut std::ffi::c_void;
        fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
        fn GlobalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }
    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = utf16.len() * std::mem::size_of::<u16>();

    unsafe {
        let mut opened = false;
        for _ in 0..10 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opened {
            EmptyClipboard();
            let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if !hmem.is_null() {
                let ptr = GlobalLock(hmem) as *mut u16;
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                    GlobalUnlock(hmem);
                    if SetClipboardData(CF_UNICODETEXT, hmem).is_null() {
                        GlobalFree(hmem);
                    }
                } else {
                    GlobalFree(hmem);
                }
            }
            CloseClipboard();
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_win32_clipboard(_text: &str) {}

#[cfg(target_os = "windows")]
pub fn get_win32_clipboard() -> Option<String> {
    #[link(name = "user32")]
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(uFormat: u32) -> *mut std::ffi::c_void;
        fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
    }
    const CF_UNICODETEXT: u32 = 13;
    unsafe {
        let mut opened = false;
        for _ in 0..10 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opened {
            let handle = GetClipboardData(CF_UNICODETEXT);
            let mut result = None;
            if !handle.is_null() {
                let ptr = GlobalLock(handle) as *const u16;
                if !ptr.is_null() {
                    let mut len = 0;
                    while *ptr.add(len) != 0 {
                        len += 1;
                    }
                    let slice = std::slice::from_raw_parts(ptr, len);
                    result = String::from_utf16(slice).ok();
                    GlobalUnlock(handle);
                }
            }
            CloseClipboard();
            return result;
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_win32_clipboard() -> Option<String> {
    None
}

pub fn get_clipboard_text(app: &App) -> Option<String> {
    if let Some(text) = get_win32_clipboard() {
        if !text.is_empty() {
            return Some(text);
        }
    }
    if !app.vim.register.is_empty() {
        return Some(app.vim.register.clone());
    }
    if let Some(ref text) = app.clipboard_text {
        if !text.is_empty() {
            return Some(text.clone());
        }
    }
    None
}
