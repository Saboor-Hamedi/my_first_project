//! Rich markdown live preview renderer with stylish typographic design and smooth scrolling.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

#[derive(Debug, Clone)]
pub enum MdBlock {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    CodeBlock { lang: String, code: String },
    Quote(String),
    ListItem { bullet: String, text: String },
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

        // Headings: #, ##, ###
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
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "☑".to_string(),
                text: rest.trim().to_string(),
            });
            continue;
        }

        // Unordered List item: - or *
        if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            blocks.push(MdBlock::ListItem {
                bullet: "•".to_string(),
                text: rest.trim().to_string(),
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

/// Renders parsed markdown blocks inside `rect` with mouse-wheel scrolling and custom styling.
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

    // Subtle container background with soft card styling
    let card_bg = Color32::from_rgba_unmultiplied(
        theme.bg.r().saturating_add(6),
        theme.bg.g().saturating_add(7),
        theme.bg.b().saturating_add(9),
        230,
    );
    preview_painter.rect_filled(rect, 4.0, card_bg);

    // Inner padding
    let pad_x = 16.0;
    let pad_y = 12.0;
    let max_text_w = (rect.width() - pad_x * 2.0).max(60.0);

    // Mouse scroll handling inside preview pane
    if ui.rect_contains_pointer(rect) {
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

    let blocks = parse_markdown(content);

    let start_x = rect.min.x + pad_x;
    let mut current_y = rect.min.y + pad_y - *scroll_y;

    for block in &blocks {
        match block {
            MdBlock::Heading1(text) => {
                current_y += 10.0;
                let font = FontId::proportional(font_size * 1.55);
                let color = theme.highlight;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    preview_painter.galley(pos2(start_x, current_y), galley, color);
                    // Underline accent bar
                    let bar_rect = Rect::from_min_size(
                        pos2(start_x, current_y + text_h + 3.0),
                        vec2(max_text_w.min(220.0), 2.0),
                    );
                    preview_painter.rect_filled(bar_rect, 1.0, theme.accent);
                }
                current_y += text_h + 14.0;
            }
            MdBlock::Heading2(text) => {
                current_y += 8.0;
                let font = FontId::proportional(font_size * 1.30);
                let color = theme.accent;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    preview_painter.galley(pos2(start_x, current_y), galley, color);
                    // Subtle thin line
                    preview_painter.line_segment(
                        [
                            pos2(start_x, current_y + text_h + 2.0),
                            pos2(start_x + max_text_w, current_y + text_h + 2.0),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(theme.muted.r(), theme.muted.g(), theme.muted.b(), 70)),
                    );
                }
                current_y += text_h + 10.0;
            }
            MdBlock::Heading3(text) => {
                current_y += 6.0;
                let font = FontId::proportional(font_size * 1.15);
                let color = theme.text;
                let galley = painter.layout(text.clone(), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    preview_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 8.0;
            }
            MdBlock::Paragraph(text) => {
                let font = FontId::proportional(font_size);
                let color = theme.text;
                let galley = painter.layout(clean_inline(text), font, color, max_text_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    preview_painter.galley(pos2(start_x, current_y), galley, color);
                }
                current_y += text_h + 8.0;
            }
            MdBlock::Quote(text) => {
                let font = FontId::proportional(font_size);
                let color = theme.muted;
                let galley = painter.layout(clean_inline(text), font, color, max_text_w - 18.0);
                let text_h = galley.size().y;
                let box_h = text_h + 8.0;
                if current_y + box_h >= rect.min.y && current_y <= rect.max.y {
                    // Left quote vertical accent border
                    let bar_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(3.5, box_h));
                    preview_painter.rect_filled(bar_rect, 1.5, theme.accent);
                    // Light quote background box
                    let bg_rect = Rect::from_min_size(pos2(start_x + 4.0, current_y), vec2(max_text_w - 4.0, box_h));
                    preview_painter.rect_filled(
                        bg_rect,
                        2.0,
                        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 18),
                    );
                    preview_painter.galley(pos2(start_x + 14.0, current_y + 4.0), galley, color);
                }
                current_y += box_h + 8.0;
            }
            MdBlock::ListItem { bullet, text } => {
                let font = FontId::proportional(font_size);
                let color = theme.text;
                let bullet_font = FontId::monospace(font_size * 0.95);
                let bullet_w = 20.0;
                let galley = painter.layout(clean_inline(text), font, color, max_text_w - bullet_w);
                let text_h = galley.size().y;
                if current_y + text_h >= rect.min.y && current_y <= rect.max.y {
                    let b_color = if bullet == "☑" {
                        theme.accent
                    } else {
                        theme.highlight
                    };
                    preview_painter.text(pos2(start_x, current_y), Align2::LEFT_TOP, bullet, bullet_font, b_color);
                    preview_painter.galley(pos2(start_x + bullet_w, current_y), galley, color);
                }
                current_y += text_h + 5.0;
            }
            MdBlock::CodeBlock { lang, code } => {
                current_y += 4.0;
                let font = FontId::monospace(font_size * 0.90);
                let color = theme.highlight;
                let galley = painter.layout(code.clone(), font, color, max_text_w - 18.0);
                let block_h = galley.size().y + 24.0;
                if current_y + block_h >= rect.min.y && current_y <= rect.max.y {
                    let code_rect = Rect::from_min_size(pos2(start_x, current_y), vec2(max_text_w, block_h));
                    // Dark contrasting container box
                    let box_color = Color32::from_rgba_unmultiplied(12, 14, 18, 240);
                    preview_painter.rect_filled(code_rect, 4.0, box_color);
                    preview_painter.rect_stroke(
                        code_rect,
                        4.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 65, 80, 80)),
                        egui::StrokeKind::Inside,
                    );

                    // Language badge top-right
                    if !lang.is_empty() {
                        let badge_font = FontId::monospace(10.5);
                        preview_painter.text(
                            pos2(code_rect.max.x - 8.0, code_rect.min.y + 4.0),
                            Align2::RIGHT_TOP,
                            lang.to_uppercase(),
                            badge_font,
                            theme.muted,
                        );
                    }

                    preview_painter.galley(pos2(start_x + 10.0, current_y + 14.0), galley, color);
                }
                current_y += block_h + 10.0;
            }
            MdBlock::Rule => {
                current_y += 6.0;
                if current_y >= rect.min.y && current_y <= rect.max.y {
                    preview_painter.line_segment(
                        [pos2(start_x, current_y), pos2(start_x + max_text_w, current_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(80, 85, 100, 60)),
                    );
                }
                current_y += 10.0;
            }
        }
    }

    // Clamp scroll
    let total_h = (current_y + *scroll_y - (rect.min.y + pad_y)).max(0.0);
    let max_scroll = (total_h - rect.height() + pad_y * 2.0).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);
}

fn clean_inline(s: &str) -> String {
    s.replace("**", "").replace('*', "").replace('`', "")
}
