//! Text object resolution engine for Vim commands like `ci"`, `di(`, `da{`, `yaw`.
//!
//! Accurately finds nested bracket pairs, quote boundaries, and words surrounding
//! the cursor position, supporting both inner (`i`) and around (`a`) scopes.

use crate::editor::Editor;
use crate::vim::types::{TextObjectKind, VimOperator};

/// Finds the `(start, end)` character indices of the requested text object.
/// Returns `None` if no matching pair or word boundary could be identified.
pub fn find_text_object_range(
    buf: &[char],
    cur: usize,
    inner: bool,
    kind: TextObjectKind,
) -> Option<(usize, usize)> {
    if buf.is_empty() {
        return None;
    }

    match kind {
        TextObjectKind::DoubleQuote => find_quote_range(buf, cur, '"', inner),
        TextObjectKind::SingleQuote => find_quote_range(buf, cur, '\'', inner),
        TextObjectKind::Backtick => find_quote_range(buf, cur, '`', inner),
        TextObjectKind::Parentheses => find_bracket_range(buf, cur, '(', ')', inner),
        TextObjectKind::Braces => find_bracket_range(buf, cur, '{', '}', inner),
        TextObjectKind::Brackets => find_bracket_range(buf, cur, '[', ']', inner),
        TextObjectKind::AngleBrackets => find_bracket_range(buf, cur, '<', '>', inner),
        TextObjectKind::Word => find_word_range(buf, cur, inner),
    }
}

/// Finds the range of quote-enclosed text on the current line.
fn find_quote_range(buf: &[char], cur: usize, quote: char, inner: bool) -> Option<(usize, usize)> {
    let cur_pos = cur.min(buf.len());

    // Locate current line boundaries to avoid multi-line quote matches
    let mut line_start = cur_pos;
    while line_start > 0 && buf[line_start - 1] != '\n' {
        line_start -= 1;
    }
    let mut line_end = cur_pos;
    while line_end < buf.len() && buf[line_end] != '\n' {
        line_end += 1;
    }

    // Collect all indices of the target quote character on the current line
    let quote_indices: Vec<usize> = (line_start..line_end)
        .filter(|&i| buf[i] == quote)
        .collect();

    if quote_indices.len() < 2 {
        return None;
    }

    // Find the pair enclosing cur_pos, or the next available pair on the line
    for chunk in quote_indices.chunks_exact(2) {
        let open = chunk[0];
        let close = chunk[1];

        if (cur_pos >= open && cur_pos <= close) || cur_pos < open {
            if inner {
                if open + 1 <= close {
                    return Some((open + 1, close));
                } else {
                    return Some((open + 1, open + 1));
                }
            } else {
                return Some((open, close + 1));
            }
        }
    }

    None
}

/// Finds paired brackets accounting for nested depth (e.g. `(foo (bar) baz)`).
fn find_bracket_range(
    buf: &[char],
    cur: usize,
    open_char: char,
    close_char: char,
    inner: bool,
) -> Option<(usize, usize)> {
    let cur_pos = cur.min(buf.len().saturating_sub(1));

    // Scan backwards from cursor to locate matching open bracket
    let mut depth = 0;
    let mut open_idx = None;
    let mut i = cur_pos;

    loop {
        if buf[i] == close_char && i != cur_pos {
            depth += 1;
        } else if buf[i] == open_char {
            if depth == 0 {
                open_idx = Some(i);
                break;
            } else {
                depth -= 1;
            }
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    // If cursor was before the opening bracket on the line, scan forward
    if open_idx.is_none() {
        let mut forward_i = cur_pos;
        while forward_i < buf.len() && buf[forward_i] != '\n' {
            if buf[forward_i] == open_char {
                open_idx = Some(forward_i);
                break;
            }
            forward_i += 1;
        }
    }

    let open = open_idx?;

    // Scan forward from open bracket to find its matching close bracket
    let mut close_depth = 0;
    let mut close_idx = None;

    for j in (open + 1)..buf.len() {
        if buf[j] == open_char {
            close_depth += 1;
        } else if buf[j] == close_char {
            if close_depth == 0 {
                close_idx = Some(j);
                break;
            } else {
                close_depth -= 1;
            }
        }
    }

    let close = close_idx?;

    if inner {
        Some((open + 1, close))
    } else {
        Some((open, close + 1))
    }
}

/// Finds the inner or around range of the word under or adjacent to cursor.
fn find_word_range(buf: &[char], cur: usize, inner: bool) -> Option<(usize, usize)> {
    if buf.is_empty() {
        return None;
    }
    let cur_pos = cur.min(buf.len().saturating_sub(1));

    // If on whitespace, find whitespace span
    if buf[cur_pos].is_whitespace() {
        let mut start = cur_pos;
        while start > 0 && buf[start - 1].is_whitespace() && buf[start - 1] != '\n' {
            start -= 1;
        }
        let mut end = cur_pos;
        while end < buf.len() && buf[end].is_whitespace() && buf[end] != '\n' {
            end += 1;
        }
        return Some((start, end));
    }

    // Identify word characters (alphanumeric or underscore)
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let is_target = is_word_char(buf[cur_pos]);

    let mut start = cur_pos;
    while start > 0 && is_word_char(buf[start - 1]) == is_target && !buf[start - 1].is_whitespace() {
        start -= 1;
    }

    let mut end = cur_pos;
    while end < buf.len() && is_word_char(buf[end]) == is_target && !buf[end].is_whitespace() {
        end += 1;
    }

    if !inner {
        // "aw": consume trailing spaces if present, or leading spaces if not
        let mut trail_end = end;
        while trail_end < buf.len() && buf[trail_end].is_whitespace() && buf[trail_end] != '\n' {
            trail_end += 1;
        }
        if trail_end > end {
            end = trail_end;
        } else {
            while start > 0 && buf[start - 1].is_whitespace() && buf[start - 1] != '\n' {
                start -= 1;
            }
        }
    }

    Some((start, end))
}

/// Applies a text object operation (Delete, Yank, Change) to the editor.
/// Returns the yanked/deleted text if successful.
pub fn apply_text_object_operator(
    ed: &mut Editor,
    op: VimOperator,
    inner: bool,
    kind: TextObjectKind,
) -> Option<String> {
    let (start, end) = find_text_object_range(&ed.buf, ed.cur, inner, kind)?;
    if start > end || start > ed.buf.len() || end > ed.buf.len() {
        return None;
    }

    let extracted: String = ed.buf[start..end].iter().collect();

    match op {
        VimOperator::Yank => {
            ed.cur = start;
            Some(extracted)
        }
        VimOperator::Delete | VimOperator::Change => {
            ed.save_undo_snapshot();
            ed.buf.drain(start..end);
            ed.cur = start.min(ed.buf.len());
            ed.selection = None;
            Some(extracted)
        }
    }
}

/// Selects a text object in Visual mode.
pub fn select_text_object(ed: &mut Editor, inner: bool, kind: TextObjectKind) -> bool {
    if let Some((start, end)) = find_text_object_range(&ed.buf, ed.cur, inner, kind) {
        ed.selection = Some(start);
        ed.cur = end;
        true
    } else {
        false
    }
}
