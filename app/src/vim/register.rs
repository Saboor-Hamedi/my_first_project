//! Vim register and clipboard management.

use super::VimEngine;

impl VimEngine {
    /// Sets the register text, line-wise flag, and synchronizes to the Windows OS clipboard.
    pub fn set_register(&mut self, text: String, is_line: bool) {
        crate::input::global::set_win32_clipboard(&text);
        self.register = text;
        self.register_is_line = is_line;
    }
}
