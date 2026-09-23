//! Keystroke HUD and pending command buffer management.

use super::VimEngine;
use crate::vim::types::VimSubMode;

impl VimEngine {
    /// Clears pending keys on timeout (~1s timeoutlen).
    pub fn update_hud(&mut self, now: f64) {
        let has_pending = !self.pending_keys.is_empty()
            || self.pending_op.is_some()
            || self.pending_prefix.is_some()
            || self.pending_text_object_scope.is_some()
            || self.pending_register.is_some()
            || self.count_accumulator.is_some();

        if has_pending && (now - self.pending_keys_time) > 1.0 {
            self.pending_keys.clear();
            self.pending_op = None;
            self.pending_text_object_scope = None;
            self.pending_prefix = None;
            self.pending_register = None;
            self.count_accumulator = None;
        }
    }

    /// Returns the active pending keys for HUD display (empty in Insert mode).
    pub fn pending_keys(&self) -> &str {
        if self.mode == VimSubMode::Insert {
            ""
        } else {
            &self.pending_keys
        }
    }
}
