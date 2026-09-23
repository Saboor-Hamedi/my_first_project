use super::*;

#[test]
fn test_editor_typing_and_backspace() {
    let mut ed = Editor::new();
    ed.insert_str("hello world");
    assert_eq!(ed.text(), "hello world");
    assert_eq!(ed.cur, 11);

    ed.delete_word();
    assert_eq!(ed.text(), "hello ");

    ed.backspace();
    assert_eq!(ed.text(), "hello");
}

#[test]
fn test_row_col_calculations() {
    let mut ed = Editor::new();
    ed.insert_str("abc\ndefgh\nijk");
    assert_eq!(ed.row_col(), (2, 3));

    ed.cur = 5;
    assert_eq!(ed.row_col(), (1, 1));
}

#[test]
fn test_insert_line_below() {
    let mut ed = Editor::new();
    ed.insert_str("first line\nsecond line");
    ed.cur = 2;
    ed.insert_line_below();
    assert_eq!(ed.text(), "first line\n\nsecond line");
}

#[test]
fn test_visual_line_wrapping() {
    let mut ed = Editor::new();
    ed.insert_str("The quick brown fox jumps over the lazy dog");
    let lines = ed.compute_visual_lines(16);
    assert!(lines.len() >= 3);
    let first_line: String = ed.buf[lines[0].char_start..lines[0].char_end].iter().collect();
    assert_eq!(first_line, "The quick brown ");

    ed.cur = 0;
    let (r0, c0) = ed.visual_row_col(&lines);
    assert_eq!((r0, c0), (0, 0));

    ed.down_visual(&lines);
    let (r1, _) = ed.visual_row_col(&lines);
    assert_eq!(r1, 1);

    ed.up_visual(&lines);
    let (r_back, _) = ed.visual_row_col(&lines);
    assert_eq!(r_back, 0);
}

#[test]
fn test_editor_undo_redo() {
    let mut ed = Editor::new();
    ed.insert_str("First");
    ed.insert_str(" Second");
    assert_eq!(ed.text(), "First Second");

    assert!(ed.undo());
    assert_eq!(ed.text(), "First");

    assert!(ed.redo());
    assert_eq!(ed.text(), "First Second");

    ed.clear();
    assert_eq!(ed.text(), "");
    assert!(ed.undo());
    assert_eq!(ed.text(), "First Second");
}

#[test]
fn test_editor_selection_and_replace() {
    let mut ed = Editor::new();
    ed.insert_str("Hello beautiful world");
    ed.cur = 6;
    ed.selection = Some(16);
    assert_eq!(ed.selected_text(), Some("beautiful ".to_string()));

    ed.insert_str("brave ");
    assert_eq!(ed.text(), "Hello brave world");

    ed.select_all();
    assert!(ed.delete_selection());
    assert_eq!(ed.text(), "");

    assert!(ed.undo());
    assert_eq!(ed.text(), "Hello brave world");
}

#[test]
fn test_end_visual_paragraph_line() {
    let mut ed = Editor::new();
    ed.insert_str("The quick brown fox jumps over the lazy dog and runs away");
    let lines = ed.compute_visual_lines(20);
    assert!(lines.len() > 1);

    ed.cur = 0;
    let (row_before, _) = ed.visual_row_col(&lines);
    assert_eq!(row_before, 0);

    ed.end_visual(&lines);
    let (row_after, col_after) = ed.visual_row_col(&lines);

    assert_eq!(row_after, 0);
    assert!(col_after > 0);
    assert_eq!(ed.buf[ed.cur], 'x');
}

#[test]
fn test_up_visual_from_trailing_empty_line() {
    let mut ed = Editor::new();
    ed.insert_str("Hello World\n");
    let lines = ed.compute_visual_lines(80);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].char_start, 0);
    assert_eq!(lines[0].char_end, 11);
    assert_eq!(lines[1].char_start, 12);
    assert_eq!(lines[1].char_end, 12);

    // Position cursor at the very bottom (trailing empty line)
    ed.cur = ed.buf.len();
    let (r_bottom, _) = ed.visual_row_col(&lines);
    assert_eq!(r_bottom, 1);

    // Press 'k' (up_visual) - should move immediately to previous line without needing 'h'
    ed.up_visual(&lines);
    let (r_up, _) = ed.visual_row_col(&lines);
    assert_eq!(r_up, 0);
    assert_eq!(ed.cur, 0);
}

#[test]
fn test_compute_visual_lines_trailing_newlines() {
    let mut ed = Editor::new();
    ed.insert_str("Line 1\nLine 2\n");
    let lines = ed.compute_visual_lines(80);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], VisualLine { char_start: 0, char_end: 6 });
    assert_eq!(lines[1], VisualLine { char_start: 7, char_end: 13 });
    assert_eq!(lines[2], VisualLine { char_start: 14, char_end: 14 });

    // Cursor at bottom trailing line
    ed.cur = 14;
    ed.up_visual(&lines);
    let (r, _) = ed.visual_row_col(&lines);
    assert_eq!(r, 1);
    assert_eq!(ed.cur, 7);
}

#[test]
fn test_indent_and_dedent_line() {
    let mut ed = Editor::new();
    ed.insert_str("fn main() {\n    let x = 1;\n}");

    // Indent middle line
    ed.cur = 14; // inside "let x = 1;"
    ed.indent_line();
    assert_eq!(ed.text(), "fn main() {\n        let x = 1;\n}");

    // Dedent middle line back
    ed.dedent_line();
    assert_eq!(ed.text(), "fn main() {\n    let x = 1;\n}");

    // Dedent once more
    ed.dedent_line();
    assert_eq!(ed.text(), "fn main() {\nlet x = 1;\n}");

    // Dedent on line with 0 leading spaces is a no-op
    ed.dedent_line();
    assert_eq!(ed.text(), "fn main() {\nlet x = 1;\n}");
}

#[test]
fn test_indent_and_dedent_preserves_selection() {
    let mut ed = Editor::new();
    ed.insert_str("fn main() {\n    let x = 1;\n}");

    // Select the word "let" (indices 16..19 in "    let x = 1;\n")
    // "fn main() {\n" is 12 chars. "    let" starts at 12 + 4 = 16.
    ed.selection = Some(16);
    ed.cur = 19;
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    let orig_sel_len = ed.selected_text().unwrap().len();

    // Indent (Ctrl + ]) -> shifts line right by 4 spaces
    ed.indent_line();
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    assert_eq!(ed.selected_text().unwrap().len(), orig_sel_len);

    // Indent again
    ed.indent_line();
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    assert_eq!(ed.selected_text().unwrap().len(), orig_sel_len);

    // Dedent (Ctrl + [) -> shifts line left by 4 spaces
    ed.dedent_line();
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    assert_eq!(ed.selected_text().unwrap().len(), orig_sel_len);

    // Dedent back to original
    ed.dedent_line();
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    assert_eq!(ed.selected_text().unwrap().len(), orig_sel_len);

    // Dedent to column 0
    ed.dedent_line();
    assert_eq!(ed.selected_text().as_deref(), Some("let"));
    assert_eq!(ed.selected_text().unwrap().len(), orig_sel_len);
}

#[test]
fn test_caret_kind_on_whitespace() {
    let mut ed = Editor::new();
    ed.insert_str("line1\n\nline3");
    let lines = ed.compute_visual_lines(80);
    assert_eq!(lines.len(), 3);

    // Row 0: non-blank line
    ed.cur = 0;
    let (r0, _) = ed.visual_row_col(&lines);
    assert_eq!(r0, 0);
    let style_row0 = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Normal),
        crate::caret::CaretKind::Block,
    );

    // Row 1: blank line
    ed.down_visual(&lines);
    let (r1, _) = ed.visual_row_col(&lines);
    assert_eq!(r1, 1);
    let style_row1 = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Normal),
        crate::caret::CaretKind::Block,
    );

    // Row 2: non-blank line
    ed.down_visual(&lines);
    let (r2, _) = ed.visual_row_col(&lines);
    assert_eq!(r2, 2);
    let style_row2 = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Normal),
        crate::caret::CaretKind::Block,
    );

    // Assert caret style is identical across blank and non-blank lines
    assert_eq!(style_row0, crate::caret::CaretKind::Block);
    assert_eq!(style_row1, style_row0);
    assert_eq!(style_row2, style_row0);
}

#[test]
fn test_vim_caret_consistent_across_submodes() {
    let custom = crate::caret::CaretKind::Neon;
    let normal = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Normal),
        custom,
    );
    let insert = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Insert),
        custom,
    );
    let visual = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::Visual),
        custom,
    );
    let visual_line = crate::caret::resolve_caret_kind(
        crate::app::EditorInputMode::Vim,
        Some(crate::vim::VimSubMode::VisualLine),
        custom,
    );

    assert_eq!(normal, custom);
    assert_eq!(insert, normal);
    assert_eq!(visual, normal);
    assert_eq!(visual_line, normal);
}

#[test]
fn test_caret_kinds_parsing_and_properties() {
    assert_eq!(crate::caret::CaretKind::parse("snow"), Some(crate::caret::CaretKind::Snow));
    assert_eq!(crate::caret::CaretKind::parse("water"), Some(crate::caret::CaretKind::Water));
    assert_eq!(crate::caret::CaretKind::parse("fire"), Some(crate::caret::CaretKind::Fire));
    assert_eq!(crate::caret::CaretKind::parse("candle"), Some(crate::caret::CaretKind::Candle));
    assert_eq!(crate::caret::CaretKind::parse("ice"), Some(crate::caret::CaretKind::Ice));
    assert_eq!(crate::caret::CaretKind::parse("neon"), Some(crate::caret::CaretKind::Neon));

    for &kind in crate::caret::CaretKind::ALL {
        assert!(!kind.name().is_empty());
        assert!(!kind.description().is_empty());
        assert_eq!(crate::caret::CaretKind::parse(kind.name()), Some(kind));
    }
}

#[test]
fn test_enter_preserves_indentation() {
    let mut ed = Editor::new();
    ed.insert_str("    let x = 42;");
    ed.cur = ed.buf.len();
    ed.handle_enter();
    assert_eq!(ed.text(), "    let x = 42;\n    ");
    assert_eq!(ed.cur, "    let x = 42;\n    ".len());

    // Single undo step reverts both the newline and indentation
    assert!(ed.undo());
    assert_eq!(ed.text(), "    let x = 42;");

    // Preserves tabs as well
    let mut ed_tab = Editor::new();
    ed_tab.insert_str("\t\tlet y = 1;");
    ed_tab.cur = ed_tab.buf.len();
    ed_tab.handle_enter();
    assert_eq!(ed_tab.text(), "\t\tlet y = 1;\n\t\t");
}

#[test]
fn test_enter_numbered_list_continuation() {
    let mut ed = Editor::new();
    ed.insert_str("1. Hello");
    ed.cur = ed.buf.len();
    ed.handle_enter();
    assert_eq!(ed.text(), "1. Hello\n2. ");
    assert_eq!(ed.cur, "1. Hello\n2. ".len());

    // With leading indentation
    let mut ed_indented = Editor::new();
    ed_indented.insert_str("  1. Hello");
    ed_indented.cur = ed_indented.buf.len();
    ed_indented.handle_enter();
    assert_eq!(ed_indented.text(), "  1. Hello\n  2. ");

    // Higher numbers
    let mut ed_high = Editor::new();
    ed_high.insert_str("9. Step nine");
    ed_high.cur = ed_high.buf.len();
    ed_high.handle_enter();
    assert_eq!(ed_high.text(), "9. Step nine\n10. ");
}

#[test]
fn test_enter_bullet_list_continuation() {
    // Dash bullet
    let mut ed_dash = Editor::new();
    ed_dash.insert_str("- Hello");
    ed_dash.cur = ed_dash.buf.len();
    ed_dash.handle_enter();
    assert_eq!(ed_dash.text(), "- Hello\n- ");

    // Asterisk bullet
    let mut ed_ast = Editor::new();
    ed_ast.insert_str("* Hello");
    ed_ast.cur = ed_ast.buf.len();
    ed_ast.handle_enter();
    assert_eq!(ed_ast.text(), "* Hello\n* ");

    // Plus bullet
    let mut ed_plus = Editor::new();
    ed_plus.insert_str("+ Hello");
    ed_plus.cur = ed_plus.buf.len();
    ed_plus.handle_enter();
    assert_eq!(ed_plus.text(), "+ Hello\n+ ");

    // Indented bullet
    let mut ed_indented = Editor::new();
    ed_indented.insert_str("  - Hello");
    ed_indented.cur = ed_indented.buf.len();
    ed_indented.handle_enter();
    assert_eq!(ed_indented.text(), "  - Hello\n  - ");
}

#[test]
fn test_enter_clears_empty_marker() {
    // Numbered empty marker: "1. " -> cleared and plain newline inserted
    let mut ed_num = Editor::new();
    ed_num.insert_str("1. ");
    ed_num.cur = ed_num.buf.len();
    ed_num.handle_enter();
    assert_eq!(ed_num.text(), "\n");
    assert_eq!(ed_num.cur, 1);

    // Bullet empty marker: "- " -> cleared and plain newline inserted
    let mut ed_bullet = Editor::new();
    ed_bullet.insert_str("- ");
    ed_bullet.cur = ed_bullet.buf.len();
    ed_bullet.handle_enter();
    assert_eq!(ed_bullet.text(), "\n");
    assert_eq!(ed_bullet.cur, 1);

    // Indented empty marker: "  - "
    let mut ed_ind = Editor::new();
    ed_ind.insert_str("  - ");
    ed_ind.cur = ed_ind.buf.len();
    ed_ind.handle_enter();
    assert_eq!(ed_ind.text(), "\n");
    assert_eq!(ed_ind.cur, 1);

    // Consecutive workflow: "1. Hello" -> Enter ("2. ") -> Enter (clears marker)
    let mut ed_flow = Editor::new();
    ed_flow.insert_str("1. Hello");
    ed_flow.cur = ed_flow.buf.len();
    ed_flow.handle_enter();
    assert_eq!(ed_flow.text(), "1. Hello\n2. ");

    ed_flow.handle_enter();
    assert_eq!(ed_flow.text(), "1. Hello\n\n");
}

#[test]
fn test_enter_with_selection_replaces_without_continuation() {
    let mut ed = Editor::new();
    ed.insert_str("1. Hello world");
    ed.selection = Some(3); // select "Hello"
    ed.cur = 8;
    ed.handle_enter();
    assert_eq!(ed.text(), "1. \n world");

    // Single undo reverts selection replacement
    assert!(ed.undo());
    assert_eq!(ed.text(), "1. Hello world");
}

#[test]
fn test_caret_cell_does_not_ignore_letters() {
    let line = VisualLine { char_start: 0, char_end: 5 }; // "hello"
    // At column 0: sits at 0
    assert_eq!(caret_cell(0, &line, None), 0);
    // Mid line: sits at character offset
    assert_eq!(caret_cell(2, &line, None), 2);
    // On last character (index 4): sits at 4
    assert_eq!(caret_cell(4, &line, None), 4);
    // At end of line (index 5): sits after last char at 5, does not ignore any letters
    assert_eq!(caret_cell(5, &line, None), 5);
    // Clamps to line_len if beyond
    assert_eq!(caret_cell(10, &line, None), 5);
}

#[test]
fn test_caret_cell_empty_line() {
    let line = VisualLine { char_start: 3, char_end: 3 };
    assert_eq!(caret_cell(3, &line, None), 0);
}

#[test]
fn test_caret_cell_normal_on_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(2, &line, Some(crate::vim::VimSubMode::Normal)), 2);
}

#[test]
fn test_caret_cell_normal_at_end() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(crate::vim::VimSubMode::Normal)), 3);
}

#[test]
fn test_caret_cell_insert_at_end_sits_past_last_char() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(crate::vim::VimSubMode::Insert)), 3);
}

#[test]
fn test_caret_cell_insert_mid_line() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(2, &line, Some(crate::vim::VimSubMode::Insert)), 2);
}

#[test]
fn test_caret_cell_visual_modes() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, Some(crate::vim::VimSubMode::Visual)), 3);
    assert_eq!(caret_cell(3, &line, Some(crate::vim::VimSubMode::VisualLine)), 3);
}

#[test]
fn test_caret_cell_empty_line_all_modes() {
    let line = VisualLine { char_start: 5, char_end: 5 };
    assert_eq!(caret_cell(5, &line, Some(crate::vim::VimSubMode::Normal)), 0);
    assert_eq!(caret_cell(5, &line, Some(crate::vim::VimSubMode::Insert)), 0);
    assert_eq!(caret_cell(5, &line, None), 0);
}

#[test]
fn test_caret_cell_no_vim_mode_behaves_like_insert() {
    let line = VisualLine { char_start: 0, char_end: 3 };
    assert_eq!(caret_cell(3, &line, None), 3);
}

#[test]
fn test_end_visual_reaches_last_char() {
    let mut ed = Editor::new();
    ed.insert_str("hello");
    let lines = ed.compute_visual_lines(80);
    ed.cur = 0;
    ed.end_visual(&lines);
    assert_eq!(ed.cur, 5);
}

#[test]
#[test]
fn test_end_visual_reaches_end_of_line() {
    let mut ed = Editor::new();
    ed.insert_str("hello");
    let lines = ed.compute_visual_lines(80);
    ed.cur = 0;
    ed.end_visual(&lines);
    assert_eq!(ed.cur, 5);
    let col = caret_cell(ed.cur, &lines[0], Some(crate::vim::VimSubMode::Normal));
    assert_eq!(col, 5);
}
