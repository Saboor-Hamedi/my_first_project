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
        TextObjectKind::Paragraph => find_paragraph_range(buf, cur, inner),
    }
}

/// Finds the range of quote-enclosed text on the current line.
///
/// Strategy (matching real Vim behaviour):
/// 1. If cursor is on the opening quote  → select content until the next quote.
/// 2. If cursor is on the closing quote  → search backward for the opening quote.
/// 3. If cursor is inside                → search backward for opening, forward for closing.
/// 4. If cursor is before any quote pair → use the first upcoming pair.
fn find_quote_range(buf: &[char], cur: usize, quote: char, inner: bool) -> Option<(usize, usize)> {
    if buf.is_empty() {
        return None;
    }
    let cur_pos = cur.min(buf.len() - 1);

    // Locate current line boundaries to avoid multi-line quote matches
    let mut line_start = cur_pos;
    while line_start > 0 && buf[line_start - 1] != '\n' {
        line_start -= 1;
    }
    let mut line_end = cur_pos;
    while line_end < buf.len() && buf[line_end] != '\n' {
        line_end += 1;
    }

    // If cursor is on the opening quote: search forward for its closing partner.
    if buf[cur_pos] == quote {
        let close = (cur_pos + 1..line_end).find(|&i| buf[i] == quote)?;
        return if inner {
            Some((cur_pos + 1, close))
        } else {
            Some((cur_pos, close + 1))
        };
    }

    // Search backward from cursor for an opening quote on this line.
    let open = (line_start..cur_pos).rfind(|&i| buf[i] == quote);

    if let Some(open) = open {
        // Found an opening quote to the left — search forward for its closing partner.
        if let Some(close) = (open + 1..line_end).find(|&i| buf[i] == quote) {
            // Cursor must be inside [open, close].
            if cur_pos <= close {
                return if inner {
                    Some((open + 1, close))
                } else {
                    Some((open, close + 1))
                };
            }
            // Cursor is after close — fall through to forward search.
        }
    }

    // Cursor is before any pair or between closed pairs: use the next upcoming pair.
    let first_quote = (cur_pos..line_end).find(|&i| buf[i] == quote)?;
    let second_quote = (first_quote + 1..line_end).find(|&i| buf[i] == quote)?;
    if inner {
        Some((first_quote + 1, second_quote))
    } else {
        Some((first_quote, second_quote + 1))
    }
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

/// Finds the character range `(start, end)` of the paragraph surrounding `cur`.
///
/// An inner paragraph (`inner == true`) selects the contiguous block of non-blank lines,
/// excluding leading or trailing blank lines.
/// An around paragraph (`inner == false`) includes the adjacent blank line(s) (trailing by default,
/// or leading if at EOF).
pub fn find_paragraph_range(buf: &[char], cur: usize, inner: bool) -> Option<(usize, usize)> {
    if buf.is_empty() {
        return None;
    }

    struct LineSpan {
        start: usize,
        end: usize,
        is_blank: bool,
    }

    let mut lines = Vec::new();
    let mut line_start = 0;
    while line_start <= buf.len() {
        let mut line_end = line_start;
        while line_end < buf.len() && buf[line_end] != '\n' {
            line_end += 1;
        }
        let is_blank = buf[line_start..line_end].iter().all(|c| c.is_whitespace());
        lines.push(LineSpan {
            start: line_start,
            end: line_end,
            is_blank,
        });
        if line_end >= buf.len() {
            break;
        }
        line_start = line_end + 1;
    }

    if lines.is_empty() {
        return None;
    }

    // Locate which line the cursor is currently on
    let cur_pos = cur.min(buf.len());
    let mut cur_line_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        let line_full_end = if line.end < buf.len() { line.end + 1 } else { line.end };
        if cur_pos >= line.start && cur_pos < line_full_end {
            cur_line_idx = idx;
            break;
        }
        if cur_pos == line_full_end && idx + 1 == lines.len() {
            cur_line_idx = idx;
            break;
        }
    }

    if !lines[cur_line_idx].is_blank {
        // Cursor is on a text paragraph line
        let mut start_idx = cur_line_idx;
        while start_idx > 0 && !lines[start_idx - 1].is_blank {
            start_idx -= 1;
        }

        let mut end_idx = cur_line_idx;
        while end_idx + 1 < lines.len() && !lines[end_idx + 1].is_blank {
            end_idx += 1;
        }

        if inner {
            // "ip": text of the paragraph only, excluding surrounding blank lines
            let start = lines[start_idx].start;
            let end = if lines[end_idx].end < buf.len() {
                lines[end_idx].end + 1
            } else {
                lines[end_idx].end
            };
            Some((start, end))
        } else {
            // "ap": text of the paragraph plus adjacent blank line(s)
            let mut start = lines[start_idx].start;
            let mut end = if lines[end_idx].end < buf.len() {
                lines[end_idx].end + 1
            } else {
                lines[end_idx].end
            };

            // Trailing blank lines take precedence in standard Vim
            if end_idx + 1 < lines.len() && lines[end_idx + 1].is_blank {
                let mut post_blank = end_idx + 1;
                while post_blank + 1 < lines.len() && lines[post_blank + 1].is_blank {
                    post_blank += 1;
                }
                end = if lines[post_blank].end < buf.len() {
                    lines[post_blank].end + 1
                } else {
                    lines[post_blank].end
                };
            } else if start_idx > 0 && lines[start_idx - 1].is_blank {
                // If no trailing blank line (e.g. at EOF), consume preceding blank lines
                let mut pre_blank = start_idx - 1;
                while pre_blank > 0 && lines[pre_blank - 1].is_blank {
                    pre_blank -= 1;
                }
                start = lines[pre_blank].start;
            }
            Some((start, end))
        }
    } else {
        // Cursor is on a blank line: select contiguous blank lines (or blank lines + adjacent paragraph)
        let mut start_blank = cur_line_idx;
        while start_blank > 0 && lines[start_blank - 1].is_blank {
            start_blank -= 1;
        }

        let mut end_blank = cur_line_idx;
        while end_blank + 1 < lines.len() && lines[end_blank + 1].is_blank {
            end_blank += 1;
        }

        if inner {
            let start = lines[start_blank].start;
            let end = if lines[end_blank].end < buf.len() {
                lines[end_blank].end + 1
            } else {
                lines[end_blank].end
            };
            Some((start, end))
        } else {
            // "ap" on blank lines includes following paragraph if available, or preceding
            if end_blank + 1 < lines.len() && !lines[end_blank + 1].is_blank {
                let mut post_para = end_blank + 1;
                while post_para + 1 < lines.len() && !lines[post_para + 1].is_blank {
                    post_para += 1;
                }
                let end = if lines[post_para].end < buf.len() {
                    lines[post_para].end + 1
                } else {
                    lines[post_para].end
                };
                Some((lines[start_blank].start, end))
            } else if start_blank > 0 && !lines[start_blank - 1].is_blank {
                let mut pre_para = start_blank - 1;
                while pre_para > 0 && !lines[pre_para - 1].is_blank {
                    pre_para -= 1;
                }
                let end = if lines[end_blank].end < buf.len() {
                    lines[end_blank].end + 1
                } else {
                    lines[end_blank].end
                };
                Some((lines[pre_para].start, end))
            } else {
                let start = lines[start_blank].start;
                let end = if lines[end_blank].end < buf.len() {
                    lines[end_blank].end + 1
                } else {
                    lines[end_blank].end
                };
                Some((start, end))
            }
        }
    }
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
        ed.selection_inclusive = false;
        true
    } else {
        false
    }
}
