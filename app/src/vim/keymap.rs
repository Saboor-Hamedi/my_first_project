//! Configurable Vim keymap registry.
//!
//! Maps concrete keystrokes and characters to abstract `VimAction` definitions.
//! Allows seamless customization or remapping of shortcuts by users without
//! altering underlying engine or motion logic.

use crate::vim::types::{
    InsertPosition, TextObjectKind, VimAction, VimMotion, VimOperator,
};
use eframe::egui::Key;
use std::collections::HashMap;

/// Represents either a single character or a special keyboard key combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyStroke {
    Char(char),
    Key { key: Key, ctrl: bool },
}

impl From<char> for KeyStroke {
    fn from(c: char) -> Self {
        KeyStroke::Char(c)
    }
}

/// Configurable keymap table for Vim modes.
#[derive(Debug, Clone)]
pub struct VimKeymap {
    /// Normal mode key actions.
    pub normal: HashMap<KeyStroke, VimAction>,
    /// Visual and VisualLine mode key actions.
    pub visual: HashMap<KeyStroke, VimAction>,
    /// Operator-pending actions (e.g. `(Delete, 'd') -> OperatorLine(Delete)`).
    pub operator_combinations: HashMap<(VimOperator, char), VimAction>,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self::new_standard()
    }
}

impl VimKeymap {
    /// Constructs a standard default Vim keymap.
    pub fn new_standard() -> Self {
        let mut normal = HashMap::new();
        let mut visual = HashMap::new();
        let mut operator_combinations = HashMap::new();

        // ── Normal Mode Motions ──────────────────────────────────────────────
        normal.insert('h'.into(), VimAction::Motion(VimMotion::Left));
        normal.insert('l'.into(), VimAction::Motion(VimMotion::Right));
        normal.insert('j'.into(), VimAction::Motion(VimMotion::DownVisual));
        normal.insert('k'.into(), VimAction::Motion(VimMotion::UpVisual));
        normal.insert('w'.into(), VimAction::Motion(VimMotion::WordForward));
        normal.insert('b'.into(), VimAction::Motion(VimMotion::WordBackward));
        normal.insert('0'.into(), VimAction::Motion(VimMotion::LineStart));
        normal.insert('$'.into(), VimAction::Motion(VimMotion::LineEnd));
        normal.insert('G'.into(), VimAction::Motion(VimMotion::BufferEnd));

        // Arrow keys in Normal mode
        normal.insert(KeyStroke::Key { key: Key::ArrowLeft, ctrl: false }, VimAction::Motion(VimMotion::Left));
        normal.insert(KeyStroke::Key { key: Key::ArrowRight, ctrl: false }, VimAction::Motion(VimMotion::Right));
        normal.insert(KeyStroke::Key { key: Key::ArrowUp, ctrl: false }, VimAction::Motion(VimMotion::UpVisual));
        normal.insert(KeyStroke::Key { key: Key::ArrowDown, ctrl: false }, VimAction::Motion(VimMotion::DownVisual));
        normal.insert(KeyStroke::Key { key: Key::Home, ctrl: false }, VimAction::Motion(VimMotion::LineStart));
        normal.insert(KeyStroke::Key { key: Key::End, ctrl: false }, VimAction::Motion(VimMotion::LineEnd));
        normal.insert(KeyStroke::Key { key: Key::ArrowLeft, ctrl: true }, VimAction::Motion(VimMotion::WordBackward));
        normal.insert(KeyStroke::Key { key: Key::ArrowRight, ctrl: true }, VimAction::Motion(VimMotion::WordForward));

        // ── Normal Mode Verbs & Editing ──────────────────────────────────────
        normal.insert('x'.into(), VimAction::DeleteChar);
        normal.insert('u'.into(), VimAction::Undo);
        normal.insert(KeyStroke::Key { key: Key::R, ctrl: true }, VimAction::Redo);
        normal.insert('p'.into(), VimAction::Paste { before: false });
        normal.insert('P'.into(), VimAction::Paste { before: true });
        normal.insert(KeyStroke::Key { key: Key::D, ctrl: true }, VimAction::DuplicateLine);

        // ── Mode Transitions ─────────────────────────────────────────────────
        normal.insert('i'.into(), VimAction::EnterInsert(InsertPosition::AtCursor));
        normal.insert('a'.into(), VimAction::EnterInsert(InsertPosition::AfterCursor));
        normal.insert('I'.into(), VimAction::EnterInsert(InsertPosition::LineStart));
        normal.insert('A'.into(), VimAction::EnterInsert(InsertPosition::LineEnd));
        normal.insert('o'.into(), VimAction::EnterInsert(InsertPosition::LineBelow));
        normal.insert('O'.into(), VimAction::EnterInsert(InsertPosition::LineAbove));
        normal.insert('v'.into(), VimAction::EnterVisual { is_line: false });
        normal.insert('V'.into(), VimAction::EnterVisual { is_line: true });

        // ── Operators (Prefixes) ─────────────────────────────────────────────
        normal.insert('d'.into(), VimAction::Operator(VimOperator::Delete));
        normal.insert('y'.into(), VimAction::Operator(VimOperator::Yank));
        normal.insert('c'.into(), VimAction::Operator(VimOperator::Change));

        // ── In-Buffer Search ─────────────────────────────────────────────────
        normal.insert('/'.into(), VimAction::EnterSearch { backward: false });
        normal.insert('?'.into(), VimAction::EnterSearch { backward: true });
        normal.insert('n'.into(), VimAction::RepeatSearch { reverse: false });
        normal.insert('N'.into(), VimAction::RepeatSearch { reverse: true });

        // ── Operator Combinations (e.g. `dd`, `yy`, `cc`, `dw`, `yw`, `cw`) ──
        operator_combinations.insert((VimOperator::Delete, 'd'), VimAction::OperatorLine(VimOperator::Delete));
        operator_combinations.insert((VimOperator::Yank, 'y'), VimAction::OperatorLine(VimOperator::Yank));
        operator_combinations.insert((VimOperator::Change, 'c'), VimAction::OperatorLine(VimOperator::Change));

        operator_combinations.insert((VimOperator::Delete, 'w'), VimAction::Motion(VimMotion::WordForward));
        operator_combinations.insert((VimOperator::Yank, 'w'), VimAction::Motion(VimMotion::WordForward));
        operator_combinations.insert((VimOperator::Change, 'w'), VimAction::Motion(VimMotion::WordForward));

        // ── Visual Mode Bindings ─────────────────────────────────────────────
        visual.insert('h'.into(), VimAction::Motion(VimMotion::Left));
        visual.insert('l'.into(), VimAction::Motion(VimMotion::Right));
        visual.insert('j'.into(), VimAction::Motion(VimMotion::DownVisual));
        visual.insert('k'.into(), VimAction::Motion(VimMotion::UpVisual));
        visual.insert('w'.into(), VimAction::Motion(VimMotion::WordForward));
        visual.insert('b'.into(), VimAction::Motion(VimMotion::WordBackward));
        visual.insert('0'.into(), VimAction::Motion(VimMotion::LineStart));
        visual.insert('$'.into(), VimAction::Motion(VimMotion::LineEnd));
        visual.insert('G'.into(), VimAction::Motion(VimMotion::BufferEnd));

        visual.insert(KeyStroke::Key { key: Key::ArrowLeft, ctrl: false }, VimAction::Motion(VimMotion::Left));
        visual.insert(KeyStroke::Key { key: Key::ArrowRight, ctrl: false }, VimAction::Motion(VimMotion::Right));
        visual.insert(KeyStroke::Key { key: Key::ArrowUp, ctrl: false }, VimAction::Motion(VimMotion::UpVisual));
        visual.insert(KeyStroke::Key { key: Key::ArrowDown, ctrl: false }, VimAction::Motion(VimMotion::DownVisual));

        visual.insert('y'.into(), VimAction::Operator(VimOperator::Yank));
        visual.insert('d'.into(), VimAction::Operator(VimOperator::Delete));
        visual.insert('x'.into(), VimAction::Operator(VimOperator::Delete));
        visual.insert('c'.into(), VimAction::Operator(VimOperator::Change));
        visual.insert(KeyStroke::Key { key: Key::C, ctrl: true }, VimAction::Operator(VimOperator::Yank));

        Self {
            normal,
            visual,
            operator_combinations,
        }
    }

    /// Resolves a character or key stroke in Normal mode.
    pub fn lookup_normal(&self, stroke: &KeyStroke) -> Option<VimAction> {
        self.normal.get(stroke).cloned()
    }

    /// Resolves a character or key stroke in Visual mode.
    pub fn lookup_visual(&self, stroke: &KeyStroke) -> Option<VimAction> {
        self.visual.get(stroke).cloned()
    }

    /// Resolves a pending operator with a second key stroke (e.g. `d` + `d`).
    pub fn lookup_operator(&self, op: VimOperator, c: char) -> Option<VimAction> {
        self.operator_combinations.get(&(op, c)).cloned()
    }

    /// Customizes or remaps a key binding in Normal mode.
    #[allow(dead_code)]
    pub fn bind_normal(&mut self, stroke: KeyStroke, action: VimAction) {
        self.normal.insert(stroke, action);
    }

    /// Customizes or remaps a key binding in Visual mode.
    #[allow(dead_code)]
    pub fn bind_visual(&mut self, stroke: KeyStroke, action: VimAction) {
        self.visual.insert(stroke, action);
    }
}

/// Helper mapping delimiter character to `TextObjectKind`.
pub fn char_to_text_object_kind(c: char) -> Option<TextObjectKind> {
    match c {
        '"' => Some(TextObjectKind::DoubleQuote),
        '\'' => Some(TextObjectKind::SingleQuote),
        '`' => Some(TextObjectKind::Backtick),
        '(' | ')' | 'b' => Some(TextObjectKind::Parentheses),
        '{' | '}' | 'B' => Some(TextObjectKind::Braces),
        '[' | ']' => Some(TextObjectKind::Brackets),
        '<' | '>' => Some(TextObjectKind::AngleBrackets),
        'w' => Some(TextObjectKind::Word),
        _ => None,
    }
}
