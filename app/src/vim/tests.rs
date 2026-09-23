//! Comprehensive unit tests for Vim modal engine.

use super::*;
use crate::editor::Editor;
use eframe::egui::{Key, Modifiers};

#[test]
fn test_vim_mode_transitions() {
    let mut ed = Editor::new();
    ed.insert_str("Hello World");
    ed.cur = 0;
    let mut vim = VimEngine::new();
    assert_eq!(vim.mode, VimSubMode::Normal);

    // 'i' -> Insert
    assert!(vim.handle_char(&mut ed, &[], 'i'));
    assert_eq!(vim.mode, VimSubMode::Insert);

    // Esc -> Normal
    assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
    assert_eq!(vim.mode, VimSubMode::Normal);

    // 'v' -> Visual
    assert!(vim.handle_char(&mut ed, &[], 'v'));
    assert_eq!(vim.mode, VimSubMode::Visual);
    assert!(ed.selection.is_some());

    // 'w' -> move in visual mode, expanding selection
    assert!(vim.handle_char(&mut ed, &[], 'w'));
    assert!(ed.has_selection());

    // Esc -> Normal
    assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
    assert_eq!(vim.mode, VimSubMode::Normal);
    assert!(!ed.has_selection());

    // 'v' -> Visual, then 'v' again -> Normal
    assert!(vim.handle_char(&mut ed, &[], 'v'));
    assert_eq!(vim.mode, VimSubMode::Visual);
    assert!(vim.handle_char(&mut ed, &[], 'v'));
    assert_eq!(vim.mode, VimSubMode::Normal);
    assert!(!ed.has_selection());
}

#[test]
fn test_vim_motion_multipliers() {
    let mut ed = Editor::new();
    ed.insert_str("word1 word2 word3 word4 word5");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    // Type '3', then 'w' -> jump forward 3 words
    assert!(vim.handle_char(&mut ed, &[], '3'));
    assert_eq!(vim.count_accumulator, Some(3));
    assert!(vim.handle_char(&mut ed, &[], 'w'));
    assert_eq!(ed.cur, 18); // Start of "word4"

    // Type '2', then 'b' -> jump back 2 words
    assert!(vim.handle_char(&mut ed, &[], '2'));
    assert!(vim.handle_char(&mut ed, &[], 'b'));
    assert_eq!(ed.cur, 6); // Start of "word2"
}

#[test]
fn test_vim_text_objects() {
    let mut ed = Editor::new();
    ed.insert_str("let x = \"hello world\";");
    ed.cur = 12; // inside "hello world"
    let mut vim = VimEngine::new();

    // 'c', 'i', '"' -> change inside quotes
    assert!(vim.handle_char(&mut ed, &[], 'c'));
    assert_eq!(vim.pending_op, Some(VimOperator::Change));
    assert!(vim.handle_char(&mut ed, &[], 'i'));
    assert_eq!(vim.pending_text_object_scope, Some(true));
    assert!(vim.handle_char(&mut ed, &[], '"'));

    assert_eq!(vim.mode, VimSubMode::Insert);
    assert_eq!(ed.text(), "let x = \"\";");
    assert_eq!(ed.cur, 9);
    assert_eq!(vim.register, "hello world");
}

#[test]
fn test_vim_visual_text_objects() {
    // ── Test 1: vi' with cursor INSIDE the quotes ──────────────────────
    let mut ed = Editor::new();
    ed.insert_str("hello 'world' end");
    ed.cur = 8; // on 'o' inside 'world'
    let mut vim = VimEngine::new();

    assert!(vim.handle_char(&mut ed, &[], 'v'));
    assert_eq!(vim.mode, VimSubMode::Visual);
    assert!(vim.handle_char(&mut ed, &[], 'i'));
    assert!(vim.handle_char(&mut ed, &[], '\''));
    // Should select "world" (positions 7..12)
    assert!(ed.has_selection(), "vi' from inside quotes must produce a selection");
    assert_eq!(ed.selected_text().unwrap(), "world");

    // ── Test 2: vi' with cursor BEFORE the quotes ───────────────────────
    let mut ed2 = Editor::new();
    ed2.insert_str("hello 'world' end");
    ed2.cur = 2; // on 'l', before 'world'
    let mut vim2 = VimEngine::new();

    assert!(vim2.handle_char(&mut ed2, &[], 'v'));
    assert!(vim2.handle_char(&mut ed2, &[], 'i'));
    assert!(vim2.handle_char(&mut ed2, &[], '\''));
    assert!(ed2.has_selection(), "vi' from before quotes must produce a selection");
    assert_eq!(ed2.selected_text().unwrap(), "world");

    // ── Test 3: vi[ with cursor INSIDE brackets ─────────────────────────
    let mut ed3 = Editor::new();
    ed3.insert_str("arr[one, two]");
    ed3.cur = 6; // on 'n' inside [one, two]
    let mut vim3 = VimEngine::new();

    assert!(vim3.handle_char(&mut ed3, &[], 'v'));
    assert!(vim3.handle_char(&mut ed3, &[], 'i'));
    assert!(vim3.handle_char(&mut ed3, &[], '['));
    assert!(ed3.has_selection(), "vi[ from inside brackets must produce a selection");
    assert_eq!(ed3.selected_text().unwrap(), "one, two");
}

#[test]
fn test_vim_bracket_text_objects() {
    let mut ed = Editor::new();
    ed.insert_str("fn call(param1, param2);");
    ed.cur = 10;
    let mut vim = VimEngine::new();

    // 'd', 'i', '(' -> delete inside parens
    assert!(vim.handle_char(&mut ed, &[], 'd'));
    assert!(vim.handle_char(&mut ed, &[], 'i'));
    assert!(vim.handle_char(&mut ed, &[], '('));

    assert_eq!(vim.mode, VimSubMode::Normal);
    assert_eq!(ed.text(), "fn call();");
    assert_eq!(vim.register, "param1, param2");
}

#[test]
fn test_vim_search() {
    let mut ed = Editor::new();
    ed.insert_str("alpha beta gamma beta delta");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    // Press '/' -> enter search mode
    assert!(vim.handle_char(&mut ed, &[], '/'));
    assert!(vim.is_searching());

    // Type "beta"
    for c in "beta".chars() {
        assert!(vim.handle_char(&mut ed, &[], c));
    }
    assert_eq!(vim.search.query, "beta");
    assert_eq!(vim.search.match_indices, vec![6, 17]);
    assert_eq!(ed.cur, 6); // live jumps to first match

    // Press Enter to confirm search
    assert!(vim.handle_key(&mut ed, &[], Key::Enter, Modifiers::default()));
    assert_eq!(vim.mode, VimSubMode::Normal);
    assert_eq!(vim.search.last_query, "beta");
    assert_eq!(ed.cur, 6);

    // Press 'n' -> next match
    assert!(vim.handle_char(&mut ed, &[], 'n'));
    assert_eq!(ed.cur, 17);

    // Press 'n' -> wraps to first match
    assert!(vim.handle_char(&mut ed, &[], 'n'));
    assert_eq!(ed.cur, 6);

    // Press 'N' -> previous match (wraps to 17)
    assert!(vim.handle_char(&mut ed, &[], 'N'));
    assert_eq!(ed.cur, 17);

    // Matches are still highlighted after confirming
    assert!(!vim.search.match_indices.is_empty());

    // Escape in normal mode clears search match highlighting (equivalent to :noh)
    assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
    assert!(vim.search.match_indices.is_empty());
}

#[test]
fn test_vim_keystroke_hud() {
    let mut ed = Editor::new();
    ed.insert_str("line 1\nline 2\nline 3\n");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    // 1. Partial operator 'd'
    assert!(vim.handle_char(&mut ed, &[], 'd'));
    assert_eq!(vim.pending_keys(), "d");

    // Complete 'dw'
    assert!(vim.handle_char(&mut ed, &[], 'w'));
    assert_eq!(vim.pending_keys(), "");

    // 2. Multiplier '2'
    assert!(vim.handle_char(&mut ed, &[], '2'));
    assert_eq!(vim.pending_keys(), "2");

    // Complete '2j'
    assert!(vim.handle_char(&mut ed, &[], 'j'));
    assert_eq!(vim.pending_keys(), "");

    // 3. Prefix 'g'
    assert!(vim.handle_char(&mut ed, &[], 'g'));
    assert_eq!(vim.pending_keys(), "g");

    // Complete 'gg'
    assert!(vim.handle_char(&mut ed, &[], 'g'));
    assert_eq!(vim.pending_keys(), "");

    // 4. Register prefix '"' then 'a'
    assert!(vim.handle_char(&mut ed, &[], '"'));
    assert_eq!(vim.pending_keys(), "\"");
    assert!(vim.handle_char(&mut ed, &[], 'a'));
    assert_eq!(vim.pending_keys(), "\"a");

    // Cancel with Escape
    assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
    assert_eq!(vim.pending_keys(), "");

    // 5. HUD empty in Insert mode
    vim.set_mode(VimSubMode::Insert, &mut ed);
    assert_eq!(vim.pending_keys(), "");
    vim.set_mode(VimSubMode::Normal, &mut ed);

    // 6. Timeout after 1.0s clears pending keys
    assert!(vim.handle_char(&mut ed, &[], 'd'));
    assert_eq!(vim.pending_keys(), "d");
    vim.pending_keys_time = 100.0;
    vim.update_hud(101.5);
    assert_eq!(vim.pending_keys(), "");
}

#[test]
fn test_vim_tab_indentation() {
    let mut ed = Editor::new();
    let mut vim = VimEngine::new();

    // In Insert mode at line start: Tab inserts 4 spaces
    vim.set_mode(VimSubMode::Insert, &mut ed);
    assert!(vim.handle_key(&mut ed, &[], Key::Tab, Modifiers::default()));
    assert_eq!(ed.text(), "    ");

    // Shift+Tab dedents line
    let mut shift_mod = Modifiers::default();
    shift_mod.shift = true;
    assert!(vim.handle_key(&mut ed, &[], Key::Tab, shift_mod));
    assert_eq!(ed.text(), "");

    // In Normal mode: Tab indents line
    ed.insert_str("hello");
    vim.set_mode(VimSubMode::Normal, &mut ed);
    assert!(vim.handle_key(&mut ed, &[], Key::Tab, Modifiers::default()));
    assert_eq!(ed.text(), "    hello");

    // In Normal mode: Shift+Tab dedents line
    assert!(vim.handle_key(&mut ed, &[], Key::Tab, shift_mod));
    assert_eq!(ed.text(), "hello");
}

#[test]
fn test_vim_visual_dollar_inclusive_selection() {
    let mut ed = Editor::new();
    let mut vim = VimEngine::new();
    ed.insert_str("Hello World");
    ed.cur = 0;
    let lines = ed.compute_visual_lines(80);

    // Enter Visual mode
    vim.handle_char(&mut ed, &lines, 'v');
    assert_eq!(vim.mode, VimSubMode::Visual);
    assert!(ed.selection_inclusive);

    // Press '$'
    vim.handle_char(&mut ed, &lines, '$');
    assert_eq!(ed.cur, 11); // at line end (inclusive of all characters)
    let (row, col) = ed.visual_row_col(&lines);
    assert_eq!(row, 0);
    assert_eq!(col, 11);

    // Selection must include the entire line, including 'd'
    assert_eq!(ed.selected_text(), Some("Hello World".to_string()));

    // Yank 'y'
    vim.handle_char(&mut ed, &lines, 'y');
    assert_eq!(vim.mode, VimSubMode::Normal);
    assert_eq!(vim.register, "Hello World");
}

#[test]
fn test_vim_normal_yank_dollar_inclusive() {
    let mut ed = Editor::new();
    let mut vim = VimEngine::new();
    ed.insert_str("Mindforge Note");
    ed.cur = 0;
    let lines = ed.compute_visual_lines(80);

    // 'y' operator pending
    vim.handle_char(&mut ed, &lines, 'y');
    assert_eq!(vim.pending_op, Some(VimOperator::Yank));

    // '$' motion
    vim.handle_char(&mut ed, &lines, '$');
    assert_eq!(vim.pending_op, None);
    // Register must contain all characters up to and including 'e'
    assert_eq!(vim.register, "Mindforge Note");
}

#[test]
fn test_vim_open_line_above_multiline() {
    let mut ed = Editor::new();
    ed.insert_str("abc\ndef\nghi");
    // put cursor on the 'd' of "def"
    ed.cur = 5;
    let mut vim = VimEngine::new();

    // 'O' -> open line above
    assert!(vim.handle_char(&mut ed, &[], 'O'));
    assert_eq!(vim.mode, VimSubMode::Insert);
    assert_eq!(ed.text(), "abc\n\ndef\nghi");
    // Cursor must be on the new blank line, not on the trailing char of "abc"
    let (row, _) = ed.row_col();
    assert_eq!(row, 1, "cursor must be on the newly inserted blank line");
}

#[test]
fn test_vim_search_cancel_restores_cursor() {
    let mut ed = Editor::new();
    ed.insert_str("alpha beta gamma");
    ed.cur = 7;
    let mut vim = VimEngine::new();

    // Open search, type something that would move the cursor if confirmed
    assert!(vim.handle_char(&mut ed, &[], '/'));
    assert!(vim.is_searching());
    for c in "gamma".chars() {
        assert!(vim.handle_char(&mut ed, &[], c));
    }
    assert_eq!(ed.cur, 11, "live search should have jumped to 'gamma'");

    // Escape must restore original cursor
    assert!(vim.handle_key(&mut ed, &[], Key::Escape, Modifiers::default()));
    assert_eq!(ed.cur, 7, "Escape must restore the pre-search cursor");
    assert_eq!(vim.mode, VimSubMode::Normal);
}

#[test]
fn test_vim_search_opens_with_empty_query() {
    let mut ed = Editor::new();
    ed.insert_str("one two three");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    // Search for "two", confirm
    vim.handle_char(&mut ed, &[], '/');
    for c in "two".chars() { vim.handle_char(&mut ed, &[], c); }
    vim.handle_key(&mut ed, &[], Key::Enter, Modifiers::default());

    // Open search again — query must be empty, not "two"
    vim.handle_char(&mut ed, &[], '/');
    assert_eq!(vim.search.query, "", "opening / must clear any previous query");
    assert!(vim.search.active, "search.active must be true after /");
}

#[test]
fn test_vim_operator_cancels_on_unknown_suffix() {
    let mut ed = Editor::new();
    ed.insert_str("hello world");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    // 'd' starts operator-pending
    assert!(vim.handle_char(&mut ed, &[], 'd'));
    assert_eq!(vim.pending_op, Some(VimOperator::Delete));

    // 'q' is not a motion, not a text object scope, not an operator-line combo
    let handled = vim.handle_char(&mut ed, &[], 'q');
    assert!(handled, "unknown operator suffix must be consumed, not passed through");
    assert_eq!(vim.pending_op, None, "pending operator must be cancelled");
    assert!(vim.pending_keys.is_empty(), "pending keys must be cleared");
    assert_eq!(ed.text(), "hello world", "buffer must not be modified");
    assert_eq!(vim.mode, VimSubMode::Normal);
}

#[test]
fn test_vim_operator_default_keymap_still_works() {
    // Sanity: default d/y/c still enter operator-pending.
    let mut ed = Editor::new();
    ed.insert_str("hello world");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    assert!(vim.handle_char(&mut ed, &[], 'd'));
    assert_eq!(vim.pending_op, Some(VimOperator::Delete));
    assert_eq!(vim.pending_keys(), "d");

    assert!(vim.handle_char(&mut ed, &[], 'w'));
    assert_eq!(vim.pending_op, None);
    assert_eq!(ed.text(), "world");
}

#[test]
fn test_vim_operator_remap_via_keymap() {
    // Remap 'j' to Delete operator. `jw` must then behave like `dw`.
    let mut ed = Editor::new();
    ed.insert_str("hello world");
    ed.cur = 0;
    let mut vim = VimEngine::new();

    vim.keymap
        .bind_normal('j'.into(), VimAction::Operator(VimOperator::Delete));

    assert!(vim.handle_char(&mut ed, &[], 'j'));
    assert_eq!(
        vim.pending_op,
        Some(VimOperator::Delete),
        "remapped operator key must enter operator-pending"
    );

    assert!(vim.handle_char(&mut ed, &[], 'w'));
    assert_eq!(ed.text(), "world", "remapped jw must delete a word like dw");
}
