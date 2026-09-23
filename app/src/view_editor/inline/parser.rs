//! Markdown line classification, tokenization, and LayoutJob generation with character mapping.

use super::types::InlineLineKind;
use crate::theme::Theme;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Stroke, TextFormat};

/// Scans a single line's characters and classifies its markdown role.
pub fn classify_line(chars: &[char]) -> (InlineLineKind, usize) {
    if chars.is_empty() {
        return (InlineLineKind::Normal, 0);
    }

    // Code fence
    if chars.starts_with(&['`', '`', '`']) {
        let lang: String = chars.iter().skip(3).collect();
        return (InlineLineKind::CodeFence(lang.trim().to_string()), chars.len());
    }

    // Horizontal Rule: ---, ***, ___
    if chars.len() >= 3
        && chars.iter().all(|&c| c == '-' || c == '*' || c == '_' || c == ' ')
        && chars.iter().filter(|&&c| c == '-' || c == '*' || c == '_').count() >= 3
    {
        return (InlineLineKind::Rule, chars.len());
    }

    // Headings: #, ##, ###, ####
    if chars.starts_with(&['#', ' ']) {
        return (InlineLineKind::Heading(1), 2);
    }
    if chars.starts_with(&['#', '#', ' ']) {
        return (InlineLineKind::Heading(2), 3);
    }
    if chars.starts_with(&['#', '#', '#', ' ']) {
        return (InlineLineKind::Heading(3), 4);
    }
    if chars.starts_with(&['#', '#', '#', '#', ' ']) {
        return (InlineLineKind::Heading(4), 5);
    }

    // Blockquote: >
    if chars.starts_with(&['>', ' ']) {
        return (InlineLineKind::Quote, 2);
    }

    // Task list items: - [ ] or - [x] or * [ ] or * [x]
    if (chars.starts_with(&['-', ' ', '[', ' ']) || chars.starts_with(&['*', ' ', '[', ' ']))
        && chars.len() >= 6
        && chars[4] == ']'
        && chars[5] == ' '
    {
        return (
            InlineLineKind::TaskItem {
                checked: false,
                check_char_idx: 3,
            },
            6,
        );
    }
    if (chars.starts_with(&['-', ' ', '[', 'x'])
        || chars.starts_with(&['-', ' ', '[', 'X'])
        || chars.starts_with(&['*', ' ', '[', 'x'])
        || chars.starts_with(&['*', ' ', '[', 'X']))
        && chars.len() >= 6
        && chars[4] == ']'
        && chars[5] == ' '
    {
        return (
            InlineLineKind::TaskItem {
                checked: true,
                check_char_idx: 3,
            },
            6,
        );
    }

    // Bullet items: - or * or +
    if chars.starts_with(&['-', ' ']) || chars.starts_with(&['*', ' ']) || chars.starts_with(&['+', ' ']) {
        return (InlineLineKind::BulletItem, 2);
    }

    // Numbered list item: 1. or 2.
    if let Some(dot_idx) = chars.iter().position(|&c| c == '.') {
        if dot_idx > 0 && dot_idx + 1 < chars.len() && chars[dot_idx + 1] == ' ' {
            let prefix = &chars[..dot_idx];
            if prefix.iter().all(|c| c.is_ascii_digit()) {
                let num_str: String = prefix.iter().collect();
                return (InlineLineKind::NumberedItem(num_str), dot_idx + 2);
            }
        }
    }

    // Markdown Table row: | col1 | col2 |
    if chars.len() >= 2 && chars[0] == '|' && chars[chars.len() - 1] == '|' {
        let is_separator = chars.iter().all(|&c| c == '|' || c == '-' || c == ':' || c == ' ');
        return (InlineLineKind::TableRow { is_header: false, is_separator }, 0);
    }

    (InlineLineKind::Normal, 0)
}

/// Builds the `LayoutJob` and corresponding `char_map` for a single visual line.
pub fn build_line_layout(
    chars: &[char],
    char_start: usize,
    is_active: bool,
    base_font_size: f32,
    theme: &Theme,
    kind: &InlineLineKind,
    in_code_block: bool,
) -> (LayoutJob, Vec<usize>, f32) {
    let mut job = LayoutJob::default();
    let mut char_map = Vec::with_capacity(chars.len() + 1);

    if in_code_block {
        let font = FontId::monospace(base_font_size * 0.95);
        let line_h = (base_font_size * 1.5).round();
        let fmt = TextFormat::simple(font, theme.accent);
        for (i, &c) in chars.iter().enumerate() {
            job.append(&c.to_string(), 0.0, fmt.clone());
            char_map.push(char_start + i);
        }
        char_map.push(char_start + chars.len());
        return (job, char_map, line_h);
    }

    // Determine font size and line height based on line kind
    let (font_size, line_h, _is_heading) = match kind {
        InlineLineKind::Heading(lvl) => {
            let (font_id, lh) = super::elements::heading_metrics(*lvl, base_font_size);
            (font_id.size, lh, true)
        }
        InlineLineKind::Quote => (base_font_size, (base_font_size * 1.55).round(), false),
        InlineLineKind::Rule => (base_font_size * 0.8, 18.0, false),
        InlineLineKind::TableRow { is_separator: true, .. } => (base_font_size * 0.8, 16.0, false),
        InlineLineKind::TableRow { .. } => (base_font_size, (base_font_size * 1.55).round(), false),
        _ => (base_font_size, (base_font_size * 1.55).round(), false),
    };

    let syntax_color = Color32::from_rgba_unmultiplied(
        theme.muted.r(),
        theme.muted.g(),
        theme.muted.b(),
        if is_active { 150 } else { 70 },
    );
    let base_text_color = match kind {
        InlineLineKind::Heading(lvl) => super::elements::heading_color(*lvl, theme),
        InlineLineKind::Quote => theme.text,
        _ => theme.text,
    };

    let default_font = FontId::proportional(font_size);
    let syntax_font = FontId::monospace(font_size * 0.92);

    // On Active Line: show all markdown characters with syntax coloring so user can edit them cleanly
    if is_active {
        let mut i = 0;
        let n = chars.len();

        while i < n {
            // Check prefix syntax on active line (e.g. #, ##, -, >)
            if i == 0 {
                match kind {
                    InlineLineKind::Heading(lvl) => {
                        let hash_count = *lvl as usize;
                        if n > hash_count && chars[..hash_count].iter().all(|&c| c == '#') && chars[hash_count] == ' ' {
                            let prefix_len = hash_count + 1;
                            let fmt = TextFormat::simple(syntax_font.clone(), theme.accent);
                            for j in 0..prefix_len {
                                job.append(&chars[j].to_string(), 0.0, fmt.clone());
                                char_map.push(char_start + j);
                            }
                            i += prefix_len;
                            continue;
                        }
                    }
                    InlineLineKind::TaskItem { check_char_idx: _, .. } => {
                        if n >= 6 && (chars[0] == '-' || chars[0] == '*') && chars[1] == ' ' && chars[2] == '[' && chars[4] == ']' && chars[5] == ' ' {
                            let fmt_box = TextFormat::simple(syntax_font.clone(), theme.accent);
                            for j in 0..6 {
                                job.append(&chars[j].to_string(), 0.0, fmt_box.clone());
                                char_map.push(char_start + j);
                            }
                            i += 6;
                            continue;
                        }
                    }
                    InlineLineKind::BulletItem => {
                        if n >= 2 && (chars[0] == '-' || chars[0] == '*' || chars[0] == '+') && chars[1] == ' ' {
                            let fmt_bullet = TextFormat::simple(syntax_font.clone(), theme.accent);
                            for j in 0..2 {
                                job.append(&chars[j].to_string(), 0.0, fmt_bullet.clone());
                                char_map.push(char_start + j);
                            }
                            i += 2;
                            continue;
                        }
                    }
                    InlineLineKind::Quote => {
                        if n >= 2 && chars[0] == '>' && chars[1] == ' ' {
                            let fmt_quote = TextFormat::simple(syntax_font.clone(), theme.accent);
                            for j in 0..2 {
                                job.append(&chars[j].to_string(), 0.0, fmt_quote.clone());
                                char_map.push(char_start + j);
                            }
                            i += 2;
                            continue;
                        }
                    }
                    _ => {}
                }
            }

            // Inline bold: **text**
            if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
                if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['*', '*']) {
                    let bold_end = i + 2 + end_rel;
                    let fmt_star = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("**", 0.0, fmt_star);
                    char_map.push(char_start + i);
                    char_map.push(char_start + i + 1);

                    let bold_fmt = TextFormat::simple(default_font.clone(), theme.highlight);
                    for b in i + 2..bold_end {
                        job.append(&chars[b].to_string(), 0.0, bold_fmt.clone());
                        char_map.push(char_start + b);
                    }

                    let fmt_star_end = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("**", 0.0, fmt_star_end);
                    char_map.push(char_start + bold_end);
                    char_map.push(char_start + bold_end + 1);

                    i = bold_end + 2;
                    continue;
                }
            }

            // Inline code: `text`
            if chars[i] == '`' {
                if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '`') {
                    let code_end = i + 1 + end_rel;
                    let fmt_tick = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("`", 0.0, fmt_tick);
                    char_map.push(char_start + i);

                    let code_fmt = TextFormat::simple(FontId::monospace(font_size * 0.92), theme.accent);
                    for c in i + 1..code_end {
                        job.append(&chars[c].to_string(), 0.0, code_fmt.clone());
                        char_map.push(char_start + c);
                    }

                    let fmt_tick_end = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("`", 0.0, fmt_tick_end);
                    char_map.push(char_start + code_end);

                    i = code_end + 1;
                    continue;
                }
            }

            // Inline strikethrough: ~~text~~
            if i + 1 < n && chars[i] == '~' && chars[i + 1] == '~' {
                if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['~', '~']) {
                    let strike_end = i + 2 + end_rel;
                    let fmt_tilde = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("~~", 0.0, fmt_tilde);
                    char_map.push(char_start + i);
                    char_map.push(char_start + i + 1);

                    let mut strike_fmt = TextFormat::simple(default_font.clone(), theme.muted);
                    strike_fmt.strikethrough = Stroke::new(1.0, theme.muted);
                    for s in i + 2..strike_end {
                        job.append(&chars[s].to_string(), 0.0, strike_fmt.clone());
                        char_map.push(char_start + s);
                    }

                    let fmt_tilde_end = TextFormat::simple(syntax_font.clone(), syntax_color);
                    job.append("~~", 0.0, fmt_tilde_end);
                    char_map.push(char_start + strike_end);
                    char_map.push(char_start + strike_end + 1);

                    i = strike_end + 2;
                    continue;
                }
            }

            // Normal character on active line
            let fmt = TextFormat::simple(default_font.clone(), base_text_color);
            job.append(&chars[i].to_string(), 0.0, fmt);
            char_map.push(char_start + i);
            i += 1;
        }

        if job.is_empty() {
            let fmt = TextFormat::simple(default_font, Color32::TRANSPARENT);
            job.append(" ", 0.0, fmt);
            char_map.push(char_start);
        }

        // Sentinel for end of line cursor position
        char_map.push(char_start + chars.len());
        return (job, char_map, line_h);
    }

    // On Inactive Line: render pure styled typography
    let mut i = 0;
    let n = chars.len();

    // Check prefix formatting
    match kind {
        InlineLineKind::Heading(lvl) => {
            let hash_count = *lvl as usize;
            if n > hash_count && chars[..hash_count].iter().all(|&c| c == '#') && chars[hash_count] == ' ' {
                i = hash_count + 1;
            }
        }
        InlineLineKind::TaskItem { .. } => {
            if n >= 6 {
                i = 6;
                // Leave 3 spaces for the vector checkbox icon
                let space_fmt = TextFormat::simple(default_font.clone(), base_text_color);
                job.append("    ", 0.0, space_fmt);
                for _ in 0..4 {
                    char_map.push(char_start + 3);
                }
            }
        }
        InlineLineKind::BulletItem => {
            if n >= 2 {
                i = 2;
                let bullet_fmt = TextFormat::simple(default_font.clone(), theme.accent);
                job.append("•  ", 0.0, bullet_fmt);
                char_map.push(char_start);
                char_map.push(char_start);
                char_map.push(char_start + 1);
            }
        }
        InlineLineKind::Quote => {
            if n >= 2 {
                i = 2;
                // Soft margin inside wrapper card
                let space_fmt = TextFormat::simple(default_font.clone(), base_text_color);
                job.append(" ", 0.0, space_fmt);
                char_map.push(char_start);
                char_map.push(char_start + 1);
            }
        }
        InlineLineKind::Rule => {
            let hr_fmt = TextFormat::simple(default_font.clone(), Color32::TRANSPARENT);
            job.append(" ", 0.0, hr_fmt);
            char_map.push(char_start);
            char_map.push(char_start + chars.len());
            return (job, char_map, line_h);
        }
        InlineLineKind::TableRow { is_separator: true, .. } => {
            let sep_fmt = TextFormat::simple(default_font.clone(), Color32::TRANSPARENT);
            job.append(" ", 0.0, sep_fmt);
            char_map.push(char_start);
            char_map.push(char_start + chars.len());
            return (job, char_map, line_h);
        }
        _ => {}
    }

    while i < n {
        // Inline bold: **text**
        if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['*', '*']) {
                let bold_end = i + 2 + end_rel;
                let bold_fmt = TextFormat::simple(default_font.clone(), theme.highlight);
                for b in i + 2..bold_end {
                    job.append(&chars[b].to_string(), 0.0, bold_fmt.clone());
                    char_map.push(char_start + b);
                }
                i = bold_end + 2;
                continue;
            }
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '`') {
                let code_end = i + 1 + end_rel;
                let code_fmt = TextFormat::simple(FontId::monospace(font_size * 0.92), theme.accent);
                for c in i + 1..code_end {
                    job.append(&chars[c].to_string(), 0.0, code_fmt.clone());
                    char_map.push(char_start + c);
                }
                i = code_end + 1;
                continue;
            }
        }

        // Inline strikethrough: ~~text~~
        if i + 1 < n && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['~', '~']) {
                let strike_end = i + 2 + end_rel;
                let mut strike_fmt = TextFormat::simple(default_font.clone(), theme.muted);
                strike_fmt.strikethrough = Stroke::new(1.0, theme.muted);
                for s in i + 2..strike_end {
                    job.append(&chars[s].to_string(), 0.0, strike_fmt.clone());
                    char_map.push(char_start + s);
                }
                i = strike_end + 2;
                continue;
            }
        }

        // Inline italic: *text*
        if chars[i] == '*' && (i + 1 >= n || chars[i + 1] != '*') {
            if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '*') {
                let ital_end = i + 1 + end_rel;
                let mut ital_fmt = TextFormat::simple(default_font.clone(), base_text_color);
                ital_fmt.italics = true;
                for it in i + 1..ital_end {
                    job.append(&chars[it].to_string(), 0.0, ital_fmt.clone());
                    char_map.push(char_start + it);
                }
                i = ital_end + 1;
                continue;
            }
        }

        // Normal character on inactive line
        let fmt = TextFormat::simple(default_font.clone(), base_text_color);
        job.append(&chars[i].to_string(), 0.0, fmt);
        char_map.push(char_start + i);
        i += 1;
    }

    if job.is_empty() {
        let fmt = TextFormat::simple(default_font, base_text_color);
        job.append(" ", 0.0, fmt);
        char_map.push(char_start);
    }

    char_map.push(char_start + chars.len());
    (job, char_map, line_h)
}
