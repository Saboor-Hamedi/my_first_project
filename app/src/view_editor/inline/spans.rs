//! Span-based inline markdown parser producing deterministic Vec<InlineSpan>.

use super::types::{InlineSpan, InlineSpanKind};

/// Parses inline markdown spans within a line of characters.
pub fn parse_inline_spans(chars: &[char]) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let n = chars.len();
    if n == 0 {
        return spans;
    }

    let mut i = 0;
    let mut text_start = 0;

    // Check for hard line break at end of line: trailing '  ' or trailing '\'
    let mut hard_break_start = None;
    if n >= 2 && chars[n - 1] == ' ' && chars[n - 2] == ' ' {
        hard_break_start = Some(n - 2);
    } else if n >= 1 && chars[n - 1] == '\\' && (n < 2 || chars[n - 2] != '\\') {
        hard_break_start = Some(n - 1);
    }

    let parse_limit = hard_break_start.unwrap_or(n);

    while i < parse_limit {
        let c = chars[i];

        // 1. Escaped character: \X
        if c == '\\' && i + 1 < parse_limit {
            let next_c = chars[i + 1];
            if is_escapable(next_c) {
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                spans.push(InlineSpan {
                    kind: InlineSpanKind::Escape { ch: next_c },
                    start: i,
                    end: i + 2,
                    marker_len: 1,
                });
                i += 2;
                text_start = i;
                continue;
            }
        }

        // 2. Inline Code: `code` or ``code``
        if c == '`' {
            let tick_count = chars[i..parse_limit].iter().take_while(|&&ch| ch == '`').count();
            let after_ticks = i + tick_count;
            if let Some(close_rel) = find_matching_ticks(&chars[after_ticks..parse_limit], tick_count) {
                let close_start = after_ticks + close_rel;
                let close_end = close_start + tick_count;

                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }

                spans.push(InlineSpan {
                    kind: InlineSpanKind::Code,
                    start: i,
                    end: close_end,
                    marker_len: tick_count,
                });

                i = close_end;
                text_start = i;
                continue;
            }
        }

        // 3. Image: ![alt](url)
        if c == '!' && i + 1 < parse_limit && chars[i + 1] == '[' {
            if let Some((span_end, alt, url)) = parse_link_or_image(&chars[i + 1..parse_limit]) {
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                spans.push(InlineSpan {
                    kind: InlineSpanKind::Image { url, alt },
                    start: i,
                    end: i + 1 + span_end,
                    marker_len: 2,
                });
                i = i + 1 + span_end;
                text_start = i;
                continue;
            }
        }

        // 4. Footnote Ref: [^id]
        if c == '[' && i + 2 < parse_limit && chars[i + 1] == '^' {
            if let Some(close_idx) = chars[i + 2..parse_limit].iter().position(|&ch| ch == ']') {
                let id_end = i + 2 + close_idx;
                let id: String = chars[i + 2..id_end].iter().collect();
                if !id.trim().is_empty() && !id.contains(' ') {
                    if i > text_start {
                        spans.push(InlineSpan {
                            kind: InlineSpanKind::Text,
                            start: text_start,
                            end: i,
                            marker_len: 0,
                        });
                    }
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::FootnoteRef { id },
                        start: i,
                        end: id_end + 1,
                        marker_len: 2,
                    });
                    i = id_end + 1;
                    text_start = i;
                    continue;
                }
            }
        }

        // 4b. WikiLink: [[target]] or [[target|display]]
        if c == '[' && i + 1 < parse_limit && chars[i + 1] == '[' {
            if let Some(close_rel) = chars[i + 2..parse_limit].windows(2).position(|w| w[0] == ']' && w[1] == ']') {
                let inner_start = i + 2;
                let inner_end = inner_start + close_rel;
                let inner: String = chars[inner_start..inner_end].iter().collect();
                let inner_trim = inner.trim();
                if !inner_trim.is_empty() {
                    let (target, display) = if let Some((tgt, alias)) = inner_trim.split_once('|') {
                        (tgt.trim().to_string(), alias.trim().to_string())
                    } else {
                        (inner_trim.to_string(), inner_trim.to_string())
                    };

                    if i > text_start {
                        spans.push(InlineSpan {
                            kind: InlineSpanKind::Text,
                            start: text_start,
                            end: i,
                            marker_len: 0,
                        });
                    }

                    spans.push(InlineSpan {
                        kind: InlineSpanKind::WikiLink { target, display },
                        start: i,
                        end: inner_end + 2,
                        marker_len: 2,
                    });

                    i = inner_end + 2;
                    text_start = i;
                    continue;
                }
            }
        }

        // 5. Link: [text](url) or [text](url "title")
        if c == '[' {
            if let Some((span_end, text, raw_url)) = parse_link_or_image(&chars[i..parse_limit]) {
                let (url, title) = parse_url_and_title(&raw_url);
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                let _ = text; // text is bounded between start+1 and start+1+text_len
                spans.push(InlineSpan {
                    kind: InlineSpanKind::Link { url, title },
                    start: i,
                    end: i + span_end,
                    marker_len: 1,
                });
                i = i + span_end;
                text_start = i;
                continue;
            }
        }

        // 6. Autolink or HTML inline tag: <https://...> or <tag>
        if c == '<' {
            if let Some(close_idx) = chars[i + 1..parse_limit].iter().position(|&ch| ch == '>') {
                let inner_end = i + 1 + close_idx;
                let inner: String = chars[i + 1..inner_end].iter().collect();
                let inner_trim = inner.trim();

                if inner_trim.starts_with("http://")
                    || inner_trim.starts_with("https://")
                    || (inner_trim.contains('@') && !inner_trim.contains(' '))
                {
                    if i > text_start {
                        spans.push(InlineSpan {
                            kind: InlineSpanKind::Text,
                            start: text_start,
                            end: i,
                            marker_len: 0,
                        });
                    }
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Autolink { url: inner_trim.to_string() },
                        start: i,
                        end: inner_end + 1,
                        marker_len: 1,
                    });
                    i = inner_end + 1;
                    text_start = i;
                    continue;
                } else if is_html_tag(inner_trim) {
                    if i > text_start {
                        spans.push(InlineSpan {
                            kind: InlineSpanKind::Text,
                            start: text_start,
                            end: i,
                            marker_len: 0,
                        });
                    }
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Html { tag: inner.clone() },
                        start: i,
                        end: inner_end + 1,
                        marker_len: 1,
                    });
                    i = inner_end + 1;
                    text_start = i;
                    continue;
                }
            }
        }

        // 7. Strikethrough: ~~text~~
        if c == '~' && i + 1 < parse_limit && chars[i + 1] == '~' {
            if let Some(end_rel) = chars[i + 2..parse_limit].windows(2).position(|w| w == ['~', '~']) {
                let strike_end = i + 2 + end_rel + 2;
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                spans.push(InlineSpan {
                    kind: InlineSpanKind::Strike,
                    start: i,
                    end: strike_end,
                    marker_len: 2,
                });
                i = strike_end;
                text_start = i;
                continue;
            }
        }

        // 8. Bold Italic: ***text*** or ___text___
        if (c == '*' || c == '_') && i + 2 < parse_limit && chars[i + 1] == c && chars[i + 2] == c {
            let pat = [c, c, c];
            if let Some(end_rel) = chars[i + 3..parse_limit].windows(3).position(|w| w == pat) {
                let bi_end = i + 3 + end_rel + 3;
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                spans.push(InlineSpan {
                    kind: InlineSpanKind::BoldItalic,
                    start: i,
                    end: bi_end,
                    marker_len: 3,
                });
                i = bi_end;
                text_start = i;
                continue;
            }
        }

        // 9. Bold: **text** or __text__
        if (c == '*' || c == '_') && i + 1 < parse_limit && chars[i + 1] == c {
            let pat = [c, c];
            if let Some(end_rel) = chars[i + 2..parse_limit].windows(2).position(|w| w == pat) {
                let bold_end = i + 2 + end_rel + 2;
                if i > text_start {
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Text,
                        start: text_start,
                        end: i,
                        marker_len: 0,
                    });
                }
                spans.push(InlineSpan {
                    kind: InlineSpanKind::Bold,
                    start: i,
                    end: bold_end,
                    marker_len: 2,
                });
                i = bold_end;
                text_start = i;
                continue;
            }
        }

        // 10. Italic: *text* or _text_
        if c == '*' || c == '_' {
            // Guard against mid-word underscore: a_b_c is common in identifiers, not markdown italics
            let is_mid_word = c == '_'
                && i > 0
                && chars[i - 1].is_alphanumeric()
                && i + 1 < parse_limit
                && chars[i + 1].is_alphanumeric();

            if !is_mid_word {
                if let Some(end_rel) = chars[i + 1..parse_limit].iter().position(|&ch| ch == c) {
                    let ital_end = i + 1 + end_rel + 1;
                    if i > text_start {
                        spans.push(InlineSpan {
                            kind: InlineSpanKind::Text,
                            start: text_start,
                            end: i,
                            marker_len: 0,
                        });
                    }
                    spans.push(InlineSpan {
                        kind: InlineSpanKind::Italic,
                        start: i,
                        end: ital_end,
                        marker_len: 1,
                    });
                    i = ital_end;
                    text_start = i;
                    continue;
                }
            }
        }

        i += 1;
    }

    if text_start < parse_limit {
        spans.push(InlineSpan {
            kind: InlineSpanKind::Text,
            start: text_start,
            end: parse_limit,
            marker_len: 0,
        });
    }

    // Trailing hard break
    if let Some(hb_start) = hard_break_start {
        spans.push(InlineSpan {
            kind: InlineSpanKind::HardBreak,
            start: hb_start,
            end: n,
            marker_len: n - hb_start,
        });
    }

    spans
}

#[inline]
fn is_escapable(c: char) -> bool {
    matches!(
        c,
        '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '|' | '~'
    )
}

fn find_matching_ticks(slice: &[char], count: usize) -> Option<usize> {
    let mut i = 0;
    while i < slice.len() {
        if slice[i] == '`' {
            let c = slice[i..].iter().take_while(|&&ch| ch == '`').count();
            if c == count {
                return Some(i);
            }
            i += c;
        } else {
            i += 1;
        }
    }
    None
}

/// Parses `[text](url)` from a slice starting at `[`.
/// Returns `Some((total_len, text, url))`.
fn parse_link_or_image(slice: &[char]) -> Option<(usize, String, String)> {
    if slice.first() != Some(&'[') {
        return None;
    }
    let close_bracket = slice[1..].iter().position(|&c| c == ']')? + 1;
    let text: String = slice[1..close_bracket].iter().collect();

    if close_bracket + 1 < slice.len() && slice[close_bracket + 1] == '(' {
        let open_paren = close_bracket + 1;
        let close_paren = slice[open_paren + 1..].iter().position(|&c| c == ')')? + open_paren + 1;
        let url: String = slice[open_paren + 1..close_paren].iter().collect();
        return Some((close_paren + 1, text, url));
    }
    None
}

fn parse_url_and_title(raw: &str) -> (String, Option<String>) {
    let raw = raw.trim();
    if let Some(quote_idx) = raw.find('"') {
        let url = raw[..quote_idx].trim().to_string();
        let title = raw[quote_idx + 1..].trim_end_matches('"').to_string();
        (url, Some(title))
    } else {
        (raw.to_string(), None)
    }
}

fn is_html_tag(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let s = s.strip_prefix('/').unwrap_or(s);
    let s = s.strip_suffix('/').unwrap_or(s).trim();
    let tag = s.split_whitespace().next().unwrap_or("");
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "span" | "div" | "b" | "i" | "u" | "s" | "br" | "hr" | "p" | "a" | "img" | "code" | "pre" | "kbd" | "sub" | "sup" | "em" | "strong"
    )
}
