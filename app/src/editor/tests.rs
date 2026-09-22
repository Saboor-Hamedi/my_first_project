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
