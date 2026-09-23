//! Vim modal engine facade and state machine coordinator.
//!
//! Organizes modal state transitions, count multipliers, in-buffer search,
//! text objects, and keymap routing into a clean object-oriented architecture.

pub mod hud;
pub mod keymap;
pub mod motions;
pub(crate) mod normal;
pub(crate) mod operator;
pub mod register;
pub mod router;
pub mod search;
pub mod state;
pub mod text_objects;
pub mod types;
pub(crate) mod visual;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use keymap::{KeyStroke, VimKeymap};
#[allow(unused_imports)]
pub use motions::{execute_normal_motion, execute_visual_motion};
pub use search::VimSearchState;
#[allow(unused_imports)]
pub use text_objects::{apply_text_object_operator, select_text_object};
#[allow(unused_imports)]
pub use types::{
    InsertPosition, TextObjectKind, VimAction, VimMotion, VimOperator, VimSubMode,
};

use crate::editor::Editor;

/// High-level Vim state machine coordinator.
#[derive(Debug, Clone)]
pub struct VimEngine {
    /// Active sub-mode.
    pub mode: VimSubMode,
    /// Configurable keymap binding table.
    pub keymap: VimKeymap,
    /// In-buffer interactive search state machine.
    pub search: VimSearchState,
    /// Pending operator awaiting a motion or text object (e.g. `d`, `y`, `c`).
    pub pending_op: Option<VimOperator>,
    /// Pending text object scope (`Some(true)` for inner `i`, `Some(false)` for around `a`).
    pub pending_text_object_scope: Option<bool>,
    /// Pending prefix key for two-character sequences like `gg`.
    pub pending_prefix: Option<char>,
    /// Accumulated numeric motion/operator multiplier count (e.g. `3` in `3j` or `5` in `5w`).
    pub count_accumulator: Option<usize>,
    /// Clipboard register text.
    pub register: String,
    /// Whether register contains line-wise content (for `p`/`P` pasting).
    pub register_is_line: bool,
    /// Temporary feedback or search status notification.
    pub status_feedback: Option<String>,
    /// Pending register selection prefix (`"` waiting for register name).
    pub pending_register: Option<char>,
    /// Keystroke HUD buffer tracking pending/partial commands (e.g. `d`, `2j`, `gg`, `"a`).
    pub pending_keys: String,
    /// Timestamp when pending_keys was last updated (for ~1s timeoutlen).
    pub pending_keys_time: f64,
    /// Last completed action (e.g. "ci\"", "vi\"", "40j", "dw", "x") for HUD flash.
    pub last_completed_action: Option<String>,
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VimEngine {
    /// Constructs a new VimEngine in Normal mode with loaded or default keymaps.
    pub fn new() -> Self {
        Self {
            mode: VimSubMode::Normal,
            keymap: VimKeymap::load_or_init(),
            search: VimSearchState::new(),
            pending_op: None,
            pending_text_object_scope: None,
            pending_prefix: None,
            count_accumulator: None,
            register: String::new(),
            register_is_line: false,
            status_feedback: None,
            pending_register: None,
            pending_keys: String::new(),
            pending_keys_time: 0.0,
            last_completed_action: None,
        }
    }

    /// Full descriptive label for status displays.
    #[allow(dead_code)]
    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            VimSubMode::Normal => "-- NORMAL --",
            VimSubMode::Insert => "-- INSERT --",
            VimSubMode::Visual => "-- VISUAL --",
            VimSubMode::VisualLine => "-- VISUAL LINE --",
            VimSubMode::Search { .. } => "-- SEARCH --",
        }
    }

    /// Sleek, subtle compact badge label for the status bar.
    pub fn compact_label(&self) -> String {
        if let Some(count) = self.count_accumulator {
            return format!("COUNT: {}", count);
        }
        match self.mode {
            VimSubMode::Normal => "NORMAL".into(),
            VimSubMode::Insert => "INSERT".into(),
            VimSubMode::Visual => "VISUAL".into(),
            VimSubMode::VisualLine => "V-LINE".into(),
            VimSubMode::Search { backward: false } => "/ SEARCH".into(),
            VimSubMode::Search { backward: true } => "? SEARCH".into(),
        }
    }

    /// Returns true if user is in interactive search mode.
    pub fn is_searching(&self) -> bool {
        matches!(self.mode, VimSubMode::Search { .. })
    }

    /// Sets the active vim sub-mode and synchronizes editor selection state.
    pub fn set_mode(&mut self, mode: VimSubMode, ed: &mut Editor) {
        self.mode = mode;
        self.pending_op = None;
        self.pending_text_object_scope = None;
        self.pending_prefix = None;
        self.count_accumulator = None;
        self.pending_register = None;
        self.pending_keys.clear();

        match mode {
            VimSubMode::Normal => {
                ed.clear_selection();
            }
            VimSubMode::Visual => {
                if ed.selection.is_none() {
                    ed.selection = Some(ed.cur);
                }
                ed.selection_inclusive = true;
            }
            VimSubMode::VisualLine => {
                let (start, end) = ed.current_line_span();
                ed.selection = Some(start);
                ed.cur = end;
                ed.selection_inclusive = false;
            }
            VimSubMode::Insert => {
                ed.clear_selection();
            }
            VimSubMode::Search { .. } => {}
        }
    }
}
