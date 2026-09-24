//! Line classification and multi-line markdown structure promotion (Setext headings, table headers).

use super::elements::{parse_aligns, split_table_cells};
use super::types::{InlineLineKind, TableRowInfo};

/// Scans a single line's characters and classifies its markdown role.
/// Returns `(kind, prefix_len)`.
pub fn classify_line(chars: &[char]) -> (InlineLineKind, usize) {
    if chars.is_empty() {
        return (InlineLineKind::Blank, 0);
    }

    if chars.iter().all(|c| c.is_whitespace()) {
        return (InlineLineKind::Blank, chars.len());
    }

    // Code fence: ``` or ~~~
    if chars.starts_with(&['`', '`', '`']) {
        let lang: String = chars.iter().skip(3).collect();
        return (InlineLineKind::CodeFence(lang.trim().to_string()), chars.len());
    }
    if chars.starts_with(&['~', '~', '~']) {
        let lang: String = chars.iter().skip(3).collect();
        return (InlineLineKind::CodeFence(lang.trim().to_string()), chars.len());
    }

    // Horizontal Rule: ---, ***, ___ (>= 3 chars, only matching characters and spaces)
    if chars.len() >= 3
        && chars.iter().all(|&c| c == '-' || c == '*' || c == '_' || c == ' ')
        && chars.iter().filter(|&&c| c == '-' || c == '*' || c == '_').count() >= 3
    {
        return (InlineLineKind::Rule, chars.len());
    }

    // ATX Headings 1..=6: # ... ######
    for level in (1..=6).rev() {
        let hash_count = level as usize;
        if chars.len() > hash_count
            && chars[..hash_count].iter().all(|&c| c == '#')
            && chars[hash_count] == ' '
        {
            return (InlineLineKind::Heading(level), hash_count + 1);
        }
    }

    // Blockquote: > or >> or > >
    if chars.starts_with(&['>']) {
        let mut depth = 0;
        let mut idx = 0;
        while idx < chars.len() {
            if chars[idx] == '>' {
                depth += 1;
                idx += 1;
                while idx < chars.len() && chars[idx] == ' ' {
                    idx += 1;
                }
            } else {
                break;
            }
        }
        if depth > 0 {
            return (InlineLineKind::Quote(depth), idx);
        }
    }

    // Task list items: - [ ] or - [x] or * [ ] or * [x]
    // Supports with trailing space (6 chars) or at line end (5 chars)
    let is_task_prefix = |c0: char| c0 == '-' || c0 == '*';
    if chars.len() >= 5
        && is_task_prefix(chars[0])
        && chars[1] == ' '
        && chars[2] == '['
        && chars[4] == ']'
    {
        let check_c = chars[3];
        let is_checked = check_c == 'x' || check_c == 'X';
        let is_unchecked = check_c == ' ';
        if is_checked || is_unchecked {
            let prefix_len = if chars.len() >= 6 && chars[5] == ' ' {
                6
            } else {
                5
            };
            return (
                InlineLineKind::TaskItem {
                    checked: is_checked,
                    check_char_idx: 3,
                },
                prefix_len,
            );
        }
    }

    // Bullet items: - or * or +
    if chars.starts_with(&['-', ' ']) || chars.starts_with(&['*', ' ']) || chars.starts_with(&['+', ' ']) {
        return (InlineLineKind::BulletItem, 2);
    }

    // Numbered list item: 1. or 42.
    if let Some(dot_idx) = chars.iter().position(|&c| c == '.') {
        if dot_idx > 0 && dot_idx + 1 < chars.len() && chars[dot_idx + 1] == ' ' {
            let prefix = &chars[..dot_idx];
            if prefix.iter().all(|c| c.is_ascii_digit()) {
                let num_str: String = prefix.iter().collect();
                return (InlineLineKind::NumberedItem(num_str), dot_idx + 2);
            }
        }
    }

    // Markdown Table row: | col1 | col2 | or col1 | col2 |
    let s_str: String = chars.iter().collect();
    let s_trim = s_str.trim();
    if s_trim.contains('|') {
        let is_separator = s_trim.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
            && s_trim.contains('-');
        if is_separator {
            let aligns = parse_aligns(chars);
            return (
                InlineLineKind::TableRow(TableRowInfo {
                    is_header: false,
                    is_separator: true,
                    aligns,
                    col_count: 0,
                }),
                0,
            );
        } else if s_trim.starts_with('|') || s_trim.ends_with('|') || s_trim.split('|').filter(|p| !p.trim().is_empty()).count() >= 2 {
            return (
                InlineLineKind::TableRow(TableRowInfo {
                    is_header: false,
                    is_separator: false,
                    aligns: Vec::new(),
                    col_count: 0,
                }),
                0,
            );
        }
    }

    (InlineLineKind::Normal, 0)
}

/// Classifies a slice of lines, applying multi-line context promotions:
/// 1. Promotes lines followed by `===` to `SetextHeading(1)`.
/// 2. Promotes lines followed by `---` to `SetextHeading(2)`.
/// 3. Promotes table rows followed by table separators to `is_header = true` and propagates `aligns`.
/// 4. Converts newly created tables directly, promoting the first row to a header even before a separator is typed.
/// 5. Propagates canonical table column count to ALL rows in the table for strictly aligned grid rendering.
pub fn classify_lines(lines: &[Vec<char>]) -> Vec<(InlineLineKind, usize)> {
    let mut results: Vec<(InlineLineKind, usize)> = lines.iter().map(|l| classify_line(l)).collect();
    let n = results.len();
    if n == 0 {
        return results;
    }

    // Pass 2: Setext Headings
    for i in 0..n.saturating_sub(1) {
        let curr_chars = &lines[i];
        let next_chars = &lines[i + 1];
        let under: String = next_chars.iter().collect();
        let under_trim = under.trim();

        if !curr_chars.is_empty() && matches!(results[i].0, InlineLineKind::Normal) {
            if under_trim.len() >= 2 && under_trim.chars().all(|c| c == '=') {
                results[i].0 = InlineLineKind::SetextHeading(1);
                results[i + 1].0 = InlineLineKind::SetextUnderline(1);
            } else if under_trim.len() >= 2
                && under_trim.chars().all(|c| c == '-')
                && !next_chars.starts_with(&['-', ' '])
            {
                results[i].0 = InlineLineKind::SetextHeading(2);
                results[i + 1].0 = InlineLineKind::SetextUnderline(2);
            }
        }
    }

    // Pass 3: Table Header Promotion, Canonical Column Count & Alignment Propagation
    let mut i = 0;
    while i < n {
        if matches!(results[i].0, InlineLineKind::TableRow(_)) {
            let start = i;
            let mut end = i;
            while end + 1 < n && matches!(results[end + 1].0, InlineLineKind::TableRow(_)) {
                end += 1;
            }

            // Find separator in this table block
            let mut sep_idx = None;
            for k in start..=end {
                if let InlineLineKind::TableRow(ref info) = results[k].0 {
                    if info.is_separator {
                        sep_idx = Some(k);
                        break;
                    }
                }
            }

            let header_idx = if let Some(s_idx) = sep_idx {
                if s_idx > start { s_idx - 1 } else { start }
            } else {
                start
            };

            let sep_aligns = if let Some(s_idx) = sep_idx {
                if let InlineLineKind::TableRow(ref s_info) = results[s_idx].0 {
                    s_info.aligns.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            let header_cells = split_table_cells(&lines[header_idx]);
            // Canonical table column count is anchored to the header row / separator
            let table_cols = header_cells.len().max(sep_aligns.len()).max(1);

            for k in start..=end {
                let is_sep = sep_idx == Some(k);
                let is_h = !is_sep && (k == header_idx || (sep_idx.is_none() && k == start));
                results[k].0 = InlineLineKind::TableRow(TableRowInfo {
                    is_header: is_h,
                    is_separator: is_sep,
                    aligns: sep_aligns.clone(),
                    col_count: table_cols,
                });
            }

            i = end + 1;
        } else {
            i += 1;
        }
    }

    results
}
