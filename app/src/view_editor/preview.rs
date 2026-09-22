//! Rich markdown live preview renderer with stylish typographic design, theme integration, and smooth scrolling.

use crate::theme::Theme;
use eframe::egui::text::LayoutJob;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, TextFormat};

#[derive(Debug, Clone)]
pub enum MdBlock {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    Heading4(String),
    CodeBlock { lang: String, code: String },
    Quote(String),
    ListItem {
        bullet: String,
        text: String,
        checked: Option<bool>,
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

        // List item with checkbox: - [ ] or - [x]
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            blocks.push(MdBlock::ListItem {
                bullet: "☐".to_string(),
                text: rest.trim().to_string(),
                checked: Some(false),
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "☑".to_string(),
                text: rest.trim().to_string(),
                checked: Some(true),
            });
            continue;
        }

        // Unordered List item: - or *
        if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "•".to_string(),
                text: rest.trim().to_string(),
                checked: None,
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
                });
                continue;
            }
        }

        // Standard Paragraph (can span multiple lines)
        let mut p_text = line.to_string();
        while let Some(next_line) = lines.peek() {
            let next_trimmed = next_line.trim();
            if next_trimmed.is_empty()
                || next_trimmed.starts_with('#')
                || next_trimmed.starts_with("```")
                || next_trimmed.starts_with("> ")
                || next_trimmed.starts_with("- ")
                || next_trimmed.starts_with("* ")
                || next_trimmed.starts_with("---")
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

/// Builds an egui LayoutJob supporting inline bold (**text**), italic (*text*), and code (`text`).
fn build_inline_job(
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

        // Check for `inline code`
        if chars[i] == '`' {
            if let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '`') {
                let code_end = i + 1 + end_rel;
                flush_plain(&mut plain_acc, &mut job);
                let code_text: String = chars[i + 1..code_end].iter().collect();
                let mut fmt = TextFormat::simple(FontId::monospace(base_font_size * 0.88), theme.accent);
                fmt.background = Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 32);
                job.append(&format!(" {} ", code_text), 0.0, fmt);
                i = code_end + 1;
                continue;
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

/// Renders parsed markdown blocks inside `rect` with mouse-wheel scrolling and polished styling.
pub fn render_markdown_preview(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    content: &str,
    scroll_y: &mut f32,
    theme: &Theme,
    font_size: f32,
) {
    let preview_painter = painter.with_clip_rect(rect);

    // Subtle container background with soft card styling and refined border
    let card_bg = Color32::from_rgba_unmultiplied(
        theme.bg.r(),
        theme.bg.g(),
        theme.bg.b(),
        245,
    );
    preview_painter.rect_filled(rect, 6.0, card_bg);
    preview_painter.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 38)),
        egui::StrokeKind::Inside,
    );

    // Top Header Strip inside preview pane
    let top_bar_h = 26.0;
    let top_bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), top_bar_h));
    preview_painter.rect_filled(
        top_bar_rect,
        egui::CornerRadius {
            nw: 6,
            ne: 6,
            sw: 0,
            se: 0,
        },
        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 12),
    );
    preview_painter.line_segment(
        [top_bar_rect.left_bottom(), top_bar_rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 28)),
    );

    let blocks = parse_markdown(content);

    // Left indicator: dot + PREVIEW label
    preview_painter.text(
        pos2(top_bar_rect.min.x + 12.0, top_bar_rect.center().y),
        Align2::LEFT_CENTER,
        "●  LIVE PREVIEW",
        FontId::monospace(10.0),
        Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 130),
    );

    // Right stats: block count
    if !blocks.is_empty() {
        preview_painter.text(
            pos2(top_bar_rect.max.x - 12.0, top_bar_rect.center().y),
            Align2::RIGHT_CENTER,
            format!("{} items", blocks.len()),
            FontId::monospace(10.0),
            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 100),
        );
    }

    // Inner padding & content clip area (underneath top bar)
    let pad_x = 14.0;
    let pad_y = 10.0;
    let content_rect = Rect::from_min_max(
        pos2(rect.min.x, rect.min.y + top_bar_h),
        rect.max,
    );
    let content_painter = painter.with_clip_rect(content_rect);
    let max_text_w = (rect.width() - pad_x * 2.0).max(60.0);

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
        return;
    }

    let start_x = rect.min.x + pad_x;
    let mut current_y = content_rect.min.y + pad_y - *scroll_y;

    for block in &blocks {
        match block {
            MdBlock::Heading1(text) => {
                current_y += 12.0;
                let font = FontId::proportional(font_size * 1.50);
                let color = theme.highlight;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                    // Underline divider line across text width
                    let line_y = current_y + text_h + 4.0;
                    content_painter.line_segment(
                        [pos2(start_x, line_y), pos2(start_x + max_text_w, line_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 35)),
                    );
                    // Sleek accent marker
                    content_painter.rect_filled(
                        Rect::from_min_size(pos2(start_x, line_y - 0.5), vec2(36.0, 2.0)),
                        1.0,
                        theme.accent,
                    );
                }
                current_y += text_h + 14.0;
            }
            MdBlock::Heading2(text) => {
                current_y += 10.0;
                let font = FontId::proportional(font_size * 1.26);
                let color = theme.accent;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                    let line_y = current_y + text_h + 3.0;
                    content_painter.line_segment(
                        [pos2(start_x, line_y), pos2(start_x + max_text_w, line_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 25)),
                    );
                }
                current_y += text_h + 10.0;
            }
            MdBlock::Heading3(text) => {
                current_y += 8.0;
                let font = FontId::proportional(font_size * 1.12);
                let color = theme.text;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 8.0;
            }
            MdBlock::Heading4(text) => {
                current_y += 6.0;
                let font = FontId::proportional(font_size * 1.02);
                let color = theme.muted;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 6.0;
            }
            MdBlock::Paragraph(text) => {
                let job = build_inline_job(text, font_size, theme.text, theme, max_text_w);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.galley(pos2(start_x, current_y), galley, Color32::WHITE);
                }
                current_y += text_h + 9.0;
            }
            MdBlock::Quote(text) => {
                let inner_w = max_text_w - 18.0;
                let job = build_inline_job(text, font_size * 0.96, theme.muted, theme, inner_w);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                let box_h = text_h + 10.0;
                if current_y + box_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    // Left quote vertical accent border
                    let bar_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(3.5, box_h));
                    content_painter.rect_filled(bar_rect, 1.5, theme.accent);
                    // Subtle quote background box
                    let bg_rect = Rect::from_min_size(pos2(start_x + 4.0, current_y), vec2(max_text_w - 4.0, box_h));
                    content_painter.rect_filled(
                        bg_rect,
                        3.0,
                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 14),
                    );
                    content_painter.galley(pos2(start_x + 12.0, current_y + 5.0), galley, Color32::WHITE);
                }
                current_y += box_h + 8.0;
            }
            MdBlock::ListItem { bullet, text, checked } => {
                let bullet_w = 22.0;
                let text_color = match checked {
                    Some(true) => Color32::from_rgba_unmultiplied(theme.text.r(), theme.text.g(), theme.text.b(), 170),
                    _ => theme.text,
                };
                let job = build_inline_job(text, font_size, text_color, theme, max_text_w - bullet_w);
                let galley = painter.layout_job(job);
                let text_h = galley.size().y;
                if current_y + text_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    let bullet_font = FontId::monospace(font_size * 0.92);
                    let b_color = match checked {
                        Some(true) => theme.accent,
                        Some(false) => Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 130),
                        None => if bullet.starts_with(|c: char| c.is_ascii_digit()) { theme.accent } else { theme.accent },
                    };
                    content_painter.text(pos2(start_x, current_y), Align2::LEFT_TOP, bullet, bullet_font, b_color);
                    content_painter.galley(pos2(start_x + bullet_w, current_y), galley, Color32::WHITE);
                }
                current_y += text_h + 5.0;
            }
            MdBlock::CodeBlock { lang, code } => {
                current_y += 4.0;
                let font = FontId::monospace(font_size * 0.88);
                let color = theme.highlight;
                let galley = painter.layout(code.clone(), font, color, max_text_w - 20.0);
                let block_h = galley.size().y + 24.0;
                if current_y + block_h >= content_rect.min.y && current_y <= content_rect.max.y {
                    let code_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(max_text_w, block_h));
                    // Dark contrasting container box
                    let box_color = Color32::from_rgba_unmultiplied(12, 14, 18, 245);
                    content_painter.rect_filled(code_rect, 5.0, box_color);
                    content_painter.rect_stroke(
                        code_rect,
                        5.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 38)),
                        egui::StrokeKind::Inside,
                    );

                    // Language pill badge in top-right
                    if !lang.is_empty() {
                        let badge_text = lang.to_uppercase();
                        let badge_font = FontId::monospace(9.5);
                        let badge_w = (badge_text.len() as f32 * 6.5 + 10.0).max(28.0);
                        let badge_rect = Rect::from_min_size(
                            pos2(code_rect.max.x - badge_w - 8.0, code_rect.min.y + 5.0),
                            vec2(badge_w, 16.0),
                        );
                        content_painter.rect_filled(
                            badge_rect,
                            3.0,
                            Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 25),
                        );
                        content_painter.text(
                            badge_rect.center(),
                            Align2::CENTER_CENTER,
                            badge_text,
                            badge_font,
                            theme.accent,
                        );
                    }

                    content_painter.galley(pos2(start_x + 10.0, current_y + 14.0), galley, color);
                }
                current_y += block_h + 10.0;
            }
            MdBlock::Rule => {
                current_y += 6.0;
                if current_y >= content_rect.min.y && current_y <= content_rect.max.y {
                    content_painter.line_segment(
                        [pos2(start_x, current_y), pos2(start_x + max_text_w, current_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 40)),
                    );
                }
                current_y += 10.0;
            }
        }
    }

    // Clamp scroll
    let total_h = (current_y + *scroll_y - (content_rect.min.y + pad_y)).max(0.0);
    let max_scroll = (total_h - content_rect.height() + pad_y * 2.0).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);
}
