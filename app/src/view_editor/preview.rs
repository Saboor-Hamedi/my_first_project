//! Rich markdown live preview renderer with full multi-language syntax highlighting,
//! custom vector checkboxes, formatted tables, blockquotes, and smooth typography.

use crate::theme::Theme;
use eframe::egui::text::LayoutJob;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, TextFormat};

#[derive(Debug, Clone)]
pub enum MdBlock {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    Heading4(String),
    CodeBlock {
        lang: String,
        code: String,
    },
    Quote(String),
    ListItem {
        bullet: String,
        text: String,
        checked: Option<bool>,
        indent_level: usize,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Paragraph(String),
    Rule,
}

pub fn parse_markdown(text: &str) -> Vec<MdBlock> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Code block: ```
        if trimmed.starts_with("```") {
            let lang = trimmed.trim_start_matches('`').trim().to_string();
            let mut code_lines = Vec::new();
            while let Some(next_line) = lines.next() {
                if next_line.trim().starts_with("```") {
                    break;
                }
                code_lines.push(next_line);
            }
            blocks.push(MdBlock::CodeBlock {
                lang,
                code: code_lines.join("\n"),
            });
            continue;
        }

        // Horizontal Rule: --- or *** or ___
        if (trimmed.starts_with("---") || trimmed.starts_with("***") || trimmed.starts_with("___"))
            && trimmed.chars().all(|c| c == '-' || c == '*' || c == '_' || c == ' ')
            && trimmed.len() >= 3
        {
            blocks.push(MdBlock::Rule);
            continue;
        }

        // Headings: #, ##, ###, ####
        if let Some(rest) = trimmed.strip_prefix("# ") {
            blocks.push(MdBlock::Heading1(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            blocks.push(MdBlock::Heading2(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            blocks.push(MdBlock::Heading3(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("#### ") {
            blocks.push(MdBlock::Heading4(rest.trim().to_string()));
            continue;
        }

        // Blockquote: >
        if let Some(rest) = trimmed.strip_prefix("> ") {
            let mut quote_text = rest.trim().to_string();
            while let Some(next_line) = lines.peek() {
                let next_trimmed = next_line.trim();
                if let Some(next_rest) = next_trimmed.strip_prefix("> ") {
                    quote_text.push(' ');
                    quote_text.push_str(next_rest.trim());
                    lines.next();
                } else {
                    break;
                }
            }
            blocks.push(MdBlock::Quote(quote_text));
            continue;
        }

        // Table: lines with |
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|') {
            let mut table_lines = vec![trimmed.to_string()];
            while let Some(next_l) = lines.peek() {
                let n_trim = next_l.trim();
                if n_trim.starts_with('|') && n_trim.ends_with('|') {
                    table_lines.push(n_trim.to_string());
                    lines.next();
                } else {
                    break;
                }
            }
            if table_lines.len() >= 2 {
                let parse_row = |r: &str| -> Vec<String> {
                    r.trim_matches('|')
                        .split('|')
                        .map(|c| c.trim().to_string())
                        .collect()
                };
                let headers = parse_row(&table_lines[0]);
                let is_sep = table_lines[1]
                    .chars()
                    .all(|c| c == '|' || c == '-' || c == ':' || c == ' ');
                let data_start = if is_sep { 2 } else { 1 };
                let mut rows = Vec::new();
                for row_line in &table_lines[data_start..] {
                    rows.push(parse_row(row_line));
                }
                blocks.push(MdBlock::Table { headers, rows });
                continue;
            }
        }

        // Compute indentation level for nested list items
        let indent_spaces = line.chars().take_while(|&c| c == ' ' || c == '\t').fold(0, |acc, c| {
            if c == '\t' { acc + 4 } else { acc + 1 }
        });
        let indent_level = indent_spaces / 2;

        // List item with checkbox: - [ ] or - [x]
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            blocks.push(MdBlock::ListItem {
                bullet: "".to_string(),
                text: rest.trim().to_string(),
                checked: Some(false),
                indent_level,
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "".to_string(),
                text: rest.trim().to_string(),
                checked: Some(true),
                indent_level,
            });
            continue;
        }

        // Unordered List item: - or * or +
        if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")).or_else(|| trimmed.strip_prefix("+ ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "•".to_string(),
                text: rest.trim().to_string(),
                checked: None,
                indent_level,
            });
            continue;
        }

        // Ordered List item: 1. or 2.
        if let Some(dot_idx) = trimmed.find(". ") {
            let prefix = &trimmed[..dot_idx];
            if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
                blocks.push(MdBlock::ListItem {
                    bullet: format!("{}.", prefix),
                    text: trimmed[dot_idx + 2..].trim().to_string(),
                    checked: None,
                    indent_level,
                });
                continue;
            }
        }

        // Standard Paragraph (can span multiple lines)
        let mut p_text = line.to_string();
        while let Some(next_line) = lines.peek() {
            let next_trimmed = next_line.trim();
            let is_numbered = next_trimmed.find(". ").map_or(false, |dot_idx| {
                let prefix = &next_trimmed[..dot_idx];
                prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty()
            });

            if next_trimmed.is_empty()
                || next_trimmed.starts_with('#')
                || next_trimmed.starts_with("```")
                || next_trimmed.starts_with("> ")
                || next_trimmed.starts_with("- ")
                || next_trimmed.starts_with("* ")
                || next_trimmed.starts_with("+ ")
                || next_trimmed.starts_with("---")
                || is_numbered
                || (next_trimmed.starts_with('|') && next_trimmed.ends_with('|'))
            {
                break;
            }
            p_text.push(' ');
            p_text.push_str(next_trimmed);
            lines.next();
        }
        blocks.push(MdBlock::Paragraph(p_text));
    }

    blocks
}

/// Builds an egui LayoutJob supporting inline bold (**text**), italic (*text*),
/// inline code (`text`), strikethrough (~~text~~), and markdown links ([text](url)).
pub fn build_inline_job(
    text: &str,
    base_font_size: f32,
    default_color: Color32,
    theme: &Theme,
    wrap_width: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut plain_acc = String::new();

    let flush_plain = |acc: &mut String, job: &mut LayoutJob| {
        if !acc.is_empty() {
            let fmt = TextFormat::simple(FontId::proportional(base_font_size), default_color);
            job.append(acc, 0.0, fmt);
            acc.clear();
        }
    };

    while i < n {
        // Check for **bold**
        if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['*', '*']) {
                let bold_end = i + 2 + end_rel;
                flush_plain(&mut plain_acc, &mut job);
                let bold_text: String = chars[i + 2..bold_end].iter().collect();
                let fmt = TextFormat::simple(FontId::proportional(base_font_size), theme.highlight);
                job.append(&bold_text, 0.0, fmt);
                i = bold_end + 2;
                continue;
            }
        }

        // Check for ~~strikethrough~~
        if i + 1 < n && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end_rel) = chars[i + 2..].windows(2).position(|w| w == ['~', '~']) {
                let strike_end = i + 2 + end_rel;
                flush_plain(&mut plain_acc, &mut job);
                let strike_text: String = chars[i + 2..strike_end].iter().collect();
                let mut fmt = TextFormat::simple(FontId::proportional(base_font_size), theme.muted);
                fmt.strikethrough = Stroke::new(1.0, theme.muted);
                job.append(&strike_text, 0.0, fmt);
                i = strike_end + 2;
                continue;
            }
        }

        // Check for `inline code`
        if chars[i] == '`' {
            if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '`') {
                let code_end = i + 1 + end_rel;
                flush_plain(&mut plain_acc, &mut job);
                let code_text: String = chars[i + 1..code_end].iter().collect();
                // Clean syntax highlighting on text without any clunky background box/layer
                let fmt = TextFormat::simple(FontId::monospace(base_font_size * 0.92), theme.accent);
                job.append(&code_text, 0.0, fmt);
                i = code_end + 1;
                continue;
            }
        }

        // Check for [link](url)
        if chars[i] == '[' {
            if let Some(close_bracket) = chars[i + 1..].iter().position(|&c| c == ']') {
                let bracket_end = i + 1 + close_bracket;
                if bracket_end + 1 < n && chars[bracket_end + 1] == '(' {
                    if let Some(close_paren) = chars[bracket_end + 2..].iter().position(|&c| c == ')') {
                        let paren_end = bracket_end + 2 + close_paren;
                        flush_plain(&mut plain_acc, &mut job);
                        let link_text: String = chars[i + 1..bracket_end].iter().collect();
                        let mut fmt = TextFormat::simple(FontId::proportional(base_font_size), theme.accent);
                        fmt.underline = Stroke::new(1.0, theme.accent);
                        job.append(&link_text, 0.0, fmt);
                        i = paren_end + 1;
                        continue;
                    }
                }
            }
        }

        // Check for *italic*
        if chars[i] == '*' && (i + 1 >= n || chars[i + 1] != '*') {
            if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '*') {
                let ital_end = i + 1 + end_rel;
                flush_plain(&mut plain_acc, &mut job);
                let ital_text: String = chars[i + 1..ital_end].iter().collect();
                let mut fmt = TextFormat::simple(FontId::proportional(base_font_size), theme.text);
                fmt.italics = true;
                job.append(&ital_text, 0.0, fmt);
                i = ital_end + 1;
                continue;
            }
        }

        plain_acc.push(chars[i]);
        i += 1;
    }

    flush_plain(&mut plain_acc, &mut job);
    job
}

/// Tokenizes a code line into rich multi-color syntax highlighting.
pub fn highlight_code_line(
    line: &str,
    lang: &str,
    font_size: f32,
    theme: &Theme,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mono_font = FontId::monospace(font_size);
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    let is_python = lang.eq_ignore_ascii_case("python") || lang.eq_ignore_ascii_case("py");
    let is_bash = lang.eq_ignore_ascii_case("bash") || lang.eq_ignore_ascii_case("sh") || lang.eq_ignore_ascii_case("shell");

    while i < n {
        // 1. Comments
        if (chars[i] == '/' && i + 1 < n && chars[i + 1] == '/')
            || ((is_python || is_bash) && chars[i] == '#')
        {
            let comment_text: String = chars[i..].iter().collect();
            let mut fmt = TextFormat::simple(
                mono_font.clone(),
                Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 170),
            );
            fmt.italics = true;
            job.append(&comment_text, 0.0, fmt);
            break;
        }

        // 2. Strings ("..." or '...')
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let start = i;
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                } else if chars[i] == quote {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            let str_val: String = chars[start..i].iter().collect();
            // Strings: emerald / lime green
            let str_color = Color32::from_rgb(152, 195, 121);
            job.append(&str_val, 0.0, TextFormat::simple(mono_font.clone(), str_color));
            continue;
        }

        // 3. Numbers (integers, floats, hex)
        if chars[i].is_ascii_digit() && (i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_')) {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            let num_val: String = chars[start..i].iter().collect();
            // Numbers: amber / orange
            let num_color = Color32::from_rgb(209, 154, 102);
            job.append(&num_val, 0.0, TextFormat::simple(mono_font.clone(), num_color));
            continue;
        }

        // 4. Words (Keywords, Types, Identifiers, Macros)
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            // Macro indicator `!` (e.g. `println!`)
            if i < n && chars[i] == '!' {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            let is_keyword = match word.as_str() {
                // Rust
                "fn" | "let" | "mut" | "pub" | "struct" | "enum" | "impl" | "match" | "if" | "else"
                | "return" | "use" | "mod" | "trait" | "type" | "where" | "async" | "await" | "for"
                | "in" | "while" | "loop" | "const" | "static" | "crate" | "super" | "self"
                // Python / JS / General
                | "def" | "class" | "import" | "from" | "as" | "with" | "yield" | "pass" | "lambda"
                | "function" | "var" | "export" | "try" | "catch" | "finally" | "new" | "this"
                | "typeof" | "instanceof" | "echo" | "sudo" => true,
                _ => false,
            };

            let is_bool_or_none = match word.as_str() {
                "true" | "false" | "None" | "Some" | "Ok" | "Err" | "null" | "undefined" => true,
                _ => false,
            };

            let is_type = !is_keyword && (
                word.starts_with(|c: char| c.is_ascii_uppercase())
                || matches!(word.as_str(), "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                    | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                    | "f32" | "f64" | "char" | "str" | "int" | "float" | "dict" | "list" | "number" | "string")
            );

            let color = if is_keyword {
                theme.accent
            } else if is_bool_or_none {
                Color32::from_rgb(209, 154, 102) // amber
            } else if is_type {
                Color32::from_rgb(229, 192, 123) // warm gold
            } else if word.ends_with('!') {
                Color32::from_rgb(97, 175, 239)  // cyan/blue for macros
            } else {
                theme.text
            };

            job.append(&word, 0.0, TextFormat::simple(mono_font.clone(), color));
            continue;
        }

        // 5. Punctuation & Operators
        let punc_char = chars[i];
        let punc_color = match punc_char {
            '=' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '!' | '<' | '>' | '~' | '?' | ':' => {
                Color32::from_rgb(97, 175, 239) // vibrant operator
            }
            '{' | '}' | '(' | ')' | '[' | ']' => {
                Color32::from_rgb(224, 108, 117) // coral bracket
            }
            _ => Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 180),
        };
        job.append(&punc_char.to_string(), 0.0, TextFormat::simple(mono_font.clone(), punc_color));
        i += 1;
    }

    job
}

/// Renders parsed markdown blocks inside `rect` with mouse-wheel scrolling, top header with close button, and polished styling.
/// Returns `true` if the user clicked the close button.
pub fn render_markdown_preview(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    content: &str,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
) -> bool {
    render_markdown_view_inner(ui, painter, rect, content, scroll_y, theme, font_size, false)
}

/// Renders parsed markdown blocks inside `rect` as a full-height document without any nested header bar.
/// Ideal for tabs such as the Quick Start Help guide.
pub fn render_markdown_document(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    content: &str,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
) {
    let _ = render_markdown_view_inner(ui, painter, rect, content, scroll_y, theme, font_size, false);
}

fn render_markdown_view_inner(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    content: &str,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
    show_header: bool,
) -> bool {
    let mut close_clicked = false;

    let content_rect = if show_header {
        // Header bar (28px height)
        let header_h = 28.0;
        let header_rect = Rect::from_min_max(
            rect.min,
            pos2(rect.max.x, rect.min.y + header_h),
        );

        // Header subtle background & bottom border
        painter.rect_filled(header_rect, 0.0, theme.surface());
        painter.line_segment(
            [pos2(header_rect.min.x, header_rect.max.y), pos2(header_rect.max.x, header_rect.max.y)],
            Stroke::new(1.0, theme.border()),
        );

        // Header title
        painter.text(
            pos2(header_rect.min.x + 12.0, header_rect.center().y),
            Align2::LEFT_CENTER,
            "PREVIEW",
            FontId::proportional(font_size * 0.75),
            theme.muted,
        );

        // Unified sleek close button on top-right
        let btn_center = pos2(header_rect.max.x - 16.0, header_rect.center().y);
        if crate::ui_components::render_close_button(
            ui,
            painter,
            btn_center,
            20.0,
            theme,
            "preview_header_close",
        ) {
            close_clicked = true;
        }

        Rect::from_min_max(
            pos2(rect.min.x, rect.min.y + header_h),
            rect.max,
        )
    } else {
        rect
    };

    let blocks = parse_markdown(content);

    // Inner padding & content area aligned with editor body
    let pad_x = 24.0;
    let pad_y = if show_header { 12.0 } else { 18.0 };
    let content_painter = painter.with_clip_rect(content_rect);
    let max_text_w = (content_rect.width() - pad_x * 2.0).max(60.0);

    // Mouse scroll handling inside preview pane
    if ui.rect_contains_pointer(content_rect) {
        let delta = ui.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.001 {
                i.smooth_scroll_delta.y
            } else {
                i.raw_scroll_delta.y * 0.5
            }
        });
        if delta != 0.0 {
            *scroll_y = (*scroll_y - delta).max(0.0);
        }
    }

    // Empty state placeholder
    if blocks.is_empty() {
        let placeholder_y = content_rect.center().y - 10.0;
        content_painter.text(
            pos2(content_rect.center().x, placeholder_y),
            Align2::CENTER_CENTER,
            "Nothing to preview yet",
            FontId::proportional(font_size * 1.05),
            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 120),
        );
        content_painter.text(
            pos2(content_rect.center().x, placeholder_y + 20.0),
            Align2::CENTER_CENTER,
            "Type markdown in the editor to see live rendering",
            FontId::proportional(font_size * 0.85),
            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 80),
        );
        return close_clicked;
    }

    let start_x = content_rect.min.x + pad_x;
    let mut current_y = content_rect.min.y + pad_y - *scroll_y;
    let mut code_block_idx: usize = 0;

    for block in &blocks {
        match block {
            MdBlock::Heading1(text) => {
                current_y += 14.0;
                let font = FontId::proportional(font_size * 1.52);
                let color = theme.highlight;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 16.0;
            }
            MdBlock::Heading2(text) => {
                current_y += 12.0;
                let font = FontId::proportional(font_size * 1.28);
                let color = theme.accent;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 12.0;
            }
            MdBlock::Heading3(text) => {
                current_y += 10.0;
                let font = FontId::proportional(font_size * 1.12);
                let color = theme.text;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 8.0;
            }
            MdBlock::Heading4(text) => {
                current_y += 8.0;
                let font = FontId::proportional(font_size * 1.02);
                let color = theme.muted;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 6.0;
            }
            MdBlock::Paragraph(text) => {
                let job = build_inline_job(text, font_size, theme.text, theme, max_text_w);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, theme.text);
                }
                current_y += text_h + 10.0;
            }
            MdBlock::Quote(text) => {
                let inner_w = max_text_w - 24.0;
                let job = build_inline_job(text, font_size * 0.96, theme.muted, theme, inner_w);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                let box_h = text_h + 12.0;
                if current_y + box_h >= rect.min.y && current_y <= rect.max.y {
                    // Left quote vertical accent border
                    let bar_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(3.5, box_h));
                    content_painter.rect_filled(bar_rect, 1.5, theme.accent);
                    // Subtle quote background box
                    let bg_rect = Rect::from_min_size(pos2(start_x + 4.0, current_y), vec2(max_text_w - 4.0, box_h));
                    content_painter.rect_filled(
                        bg_rect,
                        4.0,
                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 14),
                    );
                    content_painter.rect_stroke(
                        bg_rect,
                        4.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 22)),
                        egui::StrokeKind::Inside,
                    );
                    content_painter.galley(pos2(start_x + 14.0, current_y + 6.0), galley, theme.text);
                }
                current_y += box_h + 10.0;
            }
            MdBlock::ListItem { bullet, text, checked, indent_level } => {
                let indent_offset = (*indent_level as f32) * 20.0;
                let item_start_x = start_x + indent_offset;
                let available_w = (max_text_w - indent_offset).max(40.0);

                let job = build_inline_job(text, font_size, theme.text, theme, available_w - 28.0);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                let row_h = text_h.max(20.0);

                if current_y + row_h >= rect.min.y && current_y <= rect.max.y {
                    if let Some(is_checked) = checked {
                        // Custom vector checkbox widget
                        let cb_size = 15.0;
                        let cb_y = current_y + 1.5;
                        let cb_rect = Rect::from_min_size(pos2(item_start_x, cb_y), vec2(cb_size, cb_size));

                        if *is_checked {
                            // Checked: Filled accent box with crisp checkmark
                            content_painter.rect_filled(cb_rect, 3.5, theme.accent);
                            // Vector checkmark
                            let p1 = pos2(cb_rect.min.x + 3.5, cb_rect.min.y + 7.5);
                            let p2 = pos2(cb_rect.min.x + 6.0, cb_rect.min.y + 11.0);
                            let p3 = pos2(cb_rect.min.x + 11.5, cb_rect.min.y + 4.5);
                            content_painter.line_segment([p1, p2], Stroke::new(1.8, theme.bg));
                            content_painter.line_segment([p2, p3], Stroke::new(1.8, theme.bg));

                            // Text: dimmed with strike-through
                            let text_start = pos2(item_start_x + 24.0, current_y);
                            content_painter.galley(text_start, galley, Color32::from_rgba_unmultiplied(255, 255, 255, 140));
                            let strike_y = current_y + text_h * 0.52;
                            content_painter.line_segment(
                                [pos2(text_start.x, strike_y), pos2(text_start.x + available_w - 28.0, strike_y)],
                                Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 120)),
                            );
                        } else {
                            // Unchecked: Rounded outline box with subtle background
                            content_painter.rect_filled(
                                cb_rect,
                                3.5,
                                Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 18),
                            );
                            content_painter.rect_stroke(
                                cb_rect,
                                3.5,
                                Stroke::new(1.5, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 140)),
                                egui::StrokeKind::Inside,
                            );
                            content_painter.galley(pos2(item_start_x + 24.0, current_y), galley, theme.text);
                        }
                    } else {
                        // Standard bullet or numbered list
                        let bullet_font = FontId::monospace(font_size * 0.92);
                        let b_color = theme.accent;
                        let bullet_w = if bullet.ends_with('.') {
                            (bullet.len() as f32 * font_size * 0.58).max(18.0)
                        } else {
                            16.0
                        };
                        content_painter.text(pos2(item_start_x, current_y), Align2::LEFT_TOP, bullet, bullet_font, b_color);
                        content_painter.galley(pos2(item_start_x + bullet_w + 6.0, current_y), galley, theme.text);
                    }
                }
                current_y += row_h + 6.0;
            }
            MdBlock::CodeBlock { lang, code } => {
                code_block_idx += 1;
                current_y += 6.0;
                let lines: Vec<&str> = code.lines().collect();
                let line_count = lines.len().max(1);
                let line_h = (font_size * 1.45).round();
                let has_header = !lang.is_empty();
                let pad_x = 14.0;
                let pad_top = if has_header { 28.0 } else { 12.0 };

                let avail_w = (max_text_w - pad_x * 2.0).max(10.0);
                let line_galleys: Vec<std::sync::Arc<egui::Galley>> = lines
                    .iter()
                    .map(|line_str| painter.layout_job(highlight_code_line(line_str, lang, font_size, theme)))
                    .collect();
                let max_line_w = line_galleys.iter().map(|g| g.size().x).fold(0.0f32, f32::max);
                let max_scroll_x = (max_line_w - avail_w).max(0.0);
                let needs_h_scroll = max_scroll_x > 0.0;
                let pad_bottom = if needs_h_scroll { 16.0 } else { 12.0 };
                let block_h = pad_top + (line_count as f32 * line_h) + pad_bottom;

                if current_y + block_h >= rect.min.y && current_y <= rect.max.y {
                    let code_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(max_text_w, block_h));

                    // Single cohesive code wrapper card matching theme surface and border
                    content_painter.rect(
                        code_rect,
                        5.0,
                        theme.surface(),
                        Stroke::new(1.0, theme.border()),
                        egui::StrokeKind::Inside,
                    );

                    // Horizontal scrolling state & input handling
                    let scroll_id = ui.id().with(("code_block_scroll_x", code_block_idx));
                    let mut scroll_x: f32 = ui.data(|d| d.get_temp(scroll_id).unwrap_or(0.0));
                    if ui.rect_contains_pointer(code_rect) && needs_h_scroll {
                        let h_delta = ui.input(|i| {
                            if i.modifiers.shift {
                                if i.smooth_scroll_delta.y.abs() > 0.001 {
                                    i.smooth_scroll_delta.y
                                } else {
                                    i.raw_scroll_delta.y * 0.5
                                }
                            } else if i.smooth_scroll_delta.x.abs() > 0.001 {
                                i.smooth_scroll_delta.x
                            } else {
                                i.raw_scroll_delta.x * 0.5
                            }
                        });
                        if h_delta != 0.0 {
                            scroll_x = (scroll_x - h_delta).clamp(0.0, max_scroll_x);
                            ui.data_mut(|d| d.insert_temp(scroll_id, scroll_x));
                            ui.ctx().request_repaint();
                        }
                    }
                    scroll_x = scroll_x.clamp(0.0, max_scroll_x);

                    // Copy button logic & state
                    let copy_id = ui.id().with(("code_block_copy", code_block_idx));
                    let current_time = ui.input(|i| i.time);
                    let last_copied: Option<f64> = ui.data(|d| d.get_temp(copy_id));
                    let is_copied = last_copied.map_or(false, |t| current_time - t < 1.0);

                    let btn_w = 58.0;
                    let btn_h = 18.0;
                    let btn_rect = Rect::from_min_size(
                        pos2(code_rect.max.x - btn_w - 10.0, code_rect.min.y + 6.0),
                        vec2(btn_w, btn_h),
                    );

                    let is_btn_hovered = ui.rect_contains_pointer(btn_rect);
                    if is_btn_hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if is_btn_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                        ui.ctx().copy_text(code.clone());
                        ui.data_mut(|d| d.insert_temp(copy_id, current_time));
                        ui.ctx().request_repaint();
                    }
                    if is_copied {
                        let elapsed = current_time - last_copied.unwrap();
                        let remaining = 1.0 - elapsed;
                        if remaining > 0.0 {
                            ui.ctx().request_repaint_after(std::time::Duration::from_millis((remaining * 1000.0) as u64 + 20));
                        }
                    }

                    if has_header {
                        // Subtle language label on the left (integrated into card, NO separate top layer)
                        content_painter.text(
                            pos2(code_rect.min.x + pad_x, code_rect.min.y + 15.0),
                            Align2::LEFT_CENTER,
                            lang.to_uppercase(),
                            FontId::monospace(10.0),
                            theme.accent,
                        );
                    }

                    // Render Copy button in top-right
                    let (btn_bg, btn_text, btn_color) = if is_copied {
                        (
                            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 45),
                            "✓ Copied",
                            theme.accent,
                        )
                    } else if is_btn_hovered {
                        (
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 45),
                            "Copy",
                            theme.text,
                        )
                    } else {
                        (
                            Color32::TRANSPARENT,
                            "Copy",
                            theme.muted,
                        )
                    };
                    if btn_bg != Color32::TRANSPARENT {
                        content_painter.rect_filled(btn_rect, 3.0, btn_bg);
                    }
                    content_painter.text(
                        btn_rect.center(),
                        Align2::CENTER_CENTER,
                        btn_text,
                        FontId::monospace(9.5),
                        btn_color,
                    );

                    // Render lines cleanly with horizontal scroll offset inside clip rect
                    let code_clip = Rect::from_min_max(
                        pos2(code_rect.min.x + pad_x, code_rect.min.y),
                        pos2(code_rect.max.x - pad_x, code_rect.max.y),
                    );
                    let code_painter = content_painter.with_clip_rect(code_clip);

                    let mut line_y = code_rect.min.y + pad_top;
                    for galley in line_galleys {
                        code_painter.galley(
                            pos2(code_rect.min.x + pad_x - scroll_x, line_y),
                            galley,
                            theme.text,
                        );
                        line_y += line_h;
                    }

                    // Interactive horizontal scrollbar at bottom of code block when text overflows
                    if needs_h_scroll {
                        let track_h = 3.5;
                        let track_y = code_rect.max.y - 7.0;
                        let track_rect = Rect::from_min_size(
                            pos2(code_rect.min.x + pad_x, track_y),
                            vec2(avail_w, track_h),
                        );
                        let thumb_w = ((avail_w / max_line_w) * avail_w).clamp(24.0, avail_w);
                        let ratio = if max_scroll_x > 0.0 { scroll_x / max_scroll_x } else { 0.0 };
                        let thumb_x = code_rect.min.x + pad_x + ratio * (avail_w - thumb_w);
                        let thumb_rect = Rect::from_min_size(pos2(thumb_x, track_y), vec2(thumb_w, track_h));

                        let hit_track = Rect::from_min_size(
                            pos2(code_rect.min.x + pad_x, track_y - 3.0),
                            vec2(avail_w, track_h + 6.0),
                        );
                        let is_hit = ui.rect_contains_pointer(hit_track);
                        if is_hit && ui.input(|i| i.pointer.primary_down()) {
                            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                let r = ((pos.x - (code_rect.min.x + pad_x) - thumb_w * 0.5) / (avail_w - thumb_w).max(1.0)).clamp(0.0, 1.0);
                                scroll_x = r * max_scroll_x;
                                ui.data_mut(|d| d.insert_temp(scroll_id, scroll_x));
                                ui.ctx().request_repaint();
                            }
                        }

                        content_painter.rect_filled(
                            track_rect,
                            1.75,
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 30),
                        );
                        let thumb_color = if is_hit {
                            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 180)
                        } else {
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 110)
                        };
                        content_painter.rect_filled(thumb_rect, 1.75, thumb_color);
                    }
                }
                current_y += block_h + 12.0;
            }
            MdBlock::Table { headers, rows } => {
                current_y += 8.0;
                let col_count = headers.len().max(1);
                let col_w = (max_text_w / col_count as f32).max(60.0);
                let cell_pad_x = 10.0;
                let cell_pad_y = 6.0;
                let row_h = font_size * 1.5 + cell_pad_y * 2.0;
                let table_h = row_h * (1 + rows.len()) as f32;

                if current_y + table_h >= rect.min.y && current_y <= rect.max.y {
                    let table_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(max_text_w, table_h));

                    // Table outer rounded container
                    content_painter.rect_stroke(
                        table_rect,
                        4.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 40)),
                        egui::StrokeKind::Inside,
                    );

                    // Header row background
                    let header_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(max_text_w, row_h));
                    content_painter.rect_filled(
                        header_rect,
                        egui::CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 },
                        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 24),
                    );
                    content_painter.line_segment(
                        [header_rect.left_bottom(), header_rect.right_bottom()],
                        Stroke::new(1.5, theme.accent),
                    );

                    // Header cells
                    for (c_idx, h_text) in headers.iter().enumerate() {
                        let c_x = start_x + c_idx as f32 * col_w + cell_pad_x;
                        let job = build_inline_job(h_text, font_size * 0.95, theme.highlight, theme, col_w - cell_pad_x * 2.0);
                        let galley = painter.layout_job(job);
                        content_painter.galley(pos2(c_x, current_y + cell_pad_y), galley, theme.highlight);
                    }

                    // Data rows
                    let mut r_y = current_y + row_h;
                    for (r_idx, row) in rows.iter().enumerate() {
                        let r_rect = Rect::from_min_size(pos2(start_x, r_y), vec2(max_text_w, row_h));
                        if r_idx % 2 == 1 {
                            content_painter.rect_filled(
                                r_rect,
                                0.0,
                                Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 10),
                            );
                        }
                        // Bottom row border
                        if r_idx + 1 < rows.len() {
                            content_painter.line_segment(
                                [r_rect.left_bottom(), r_rect.right_bottom()],
                                Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 25)),
                            );
                        }
                        for (c_idx, cell_text) in row.iter().enumerate() {
                            if c_idx < col_count {
                                let c_x = start_x + c_idx as f32 * col_w + cell_pad_x;
                                let job = build_inline_job(cell_text, font_size * 0.90, theme.text, theme, col_w - cell_pad_x * 2.0);
                                let galley = painter.layout_job(job);
                                content_painter.galley(pos2(c_x, r_y + cell_pad_y), galley, theme.text);
                            }
                        }
                        r_y += row_h;
                    }
                }
                current_y += table_h + 12.0;
            }
            MdBlock::Rule => {
                current_y += 6.0;
                if current_y >= rect.min.y && current_y <= rect.max.y {
                    content_painter.line_segment(
                        [pos2(start_x, current_y), pos2(start_x + max_text_w, current_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 40)),
                    );
                }
                current_y += 10.0;
            }
        }
    }

    // Clamp scroll with bottom breathing room aligned with editor body
    let total_h = (current_y + *scroll_y - (rect.min.y + pad_y)).max(0.0);
    let bottom_pad = 28.0;
    let max_scroll = (total_h + bottom_pad - rect.height()).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);

    close_clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_blocks() {
        let md = r#"# Main Title
## Sub Title
### Section
#### Minor

- [ ] Unchecked task
- [x] Checked task
* Normal bullet
1. First step

```rust
fn main() {
    println!("hello");
}
```

> A famous quote

| Header A | Header B |
| -------- | -------- |
| Val 1    | Val 2    |

---
Paragraph text
"#;
        let blocks = parse_markdown(md);
        assert!(matches!(blocks[0], MdBlock::Heading1(ref t) if t == "Main Title"));
        assert!(matches!(blocks[1], MdBlock::Heading2(ref t) if t == "Sub Title"));
        assert!(matches!(blocks[2], MdBlock::Heading3(ref t) if t == "Section"));
        assert!(matches!(blocks[3], MdBlock::Heading4(ref t) if t == "Minor"));

        // Checkboxes
        assert!(matches!(blocks[4], MdBlock::ListItem { checked: Some(false), ref text, .. } if text == "Unchecked task"));
        assert!(matches!(blocks[5], MdBlock::ListItem { checked: Some(true), ref text, .. } if text == "Checked task"));
        assert!(matches!(blocks[6], MdBlock::ListItem { checked: None, ref text, .. } if text == "Normal bullet"));
        assert!(matches!(blocks[7], MdBlock::ListItem { checked: None, ref text, .. } if text == "First step"));

        // Code block
        assert!(matches!(blocks[8], MdBlock::CodeBlock { ref lang, ref code } if lang == "rust" && code.contains("println!")));

        // Quote
        assert!(matches!(blocks[9], MdBlock::Quote(ref q) if q == "A famous quote"));

        // Table
        if let MdBlock::Table { ref headers, ref rows } = blocks[10] {
            assert_eq!(headers, &["Header A", "Header B"]);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0], &["Val 1", "Val 2"]);
        } else {
            panic!("Expected Table block");
        }

        // Rule & Paragraph
        assert!(matches!(blocks[11], MdBlock::Rule));
        assert!(matches!(blocks[12], MdBlock::Paragraph(ref p) if p == "Paragraph text"));
    }

    #[test]
    fn test_nested_list_indent_levels() {
        let md = "- Level 0\n  - Level 1\n    - Level 2\n  1. Nested numbered\n";
        let blocks = parse_markdown(md);
        assert_eq!(blocks.len(), 4);
        assert!(matches!(blocks[0], MdBlock::ListItem { indent_level: 0, ref text, .. } if text == "Level 0"));
        assert!(matches!(blocks[1], MdBlock::ListItem { indent_level: 1, ref text, .. } if text == "Level 1"));
        assert!(matches!(blocks[2], MdBlock::ListItem { indent_level: 2, ref text, .. } if text == "Level 2"));
        assert!(matches!(blocks[3], MdBlock::ListItem { indent_level: 1, ref text, .. } if text == "Nested numbered"));
    }
}
