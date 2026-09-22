//! Built-in Brain Documentation & Interactive Tutorials.
//! Embeds the user-friendly MindForge guides for offline, instant access
//! in both development and production release packages.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub struct DocItem {
    pub id: &'static str,
    pub title: &'static str,
    pub filename: &'static str,
    pub content: &'static str,
}

pub const BRAIN_DOCS: &[DocItem] = &[
    DocItem {
        id: "welcome",
        title: "Welcome",
        filename: "welcome.md",
        content: include_str!("../../brain/welcome.md"),
    },
    DocItem {
        id: "basics",
        title: "Basics",
        filename: "basics.md",
        content: include_str!("../../brain/basics.md"),
    },
    DocItem {
        id: "shortcuts",
        title: "Shortcuts",
        filename: "shortcuts.md",
        content: include_str!("../../brain/shortcuts.md"),
    },
    DocItem {
        id: "vim",
        title: "Vim",
        filename: "vim.md",
        content: include_str!("../../brain/vim.md"),
    },
    DocItem {
        id: "features",
        title: "Features",
        filename: "features.md",
        content: include_str!("../../brain/features.md"),
    },
];

/// Returns all available documentation documents.
/// If a local `brain/` folder exists on disk, reads the latest version from disk;
/// otherwise, falls back to the embedded compile-time copy.
pub fn get_docs() -> Vec<DocItem> {
    BRAIN_DOCS
        .iter()
        .map(|doc| {
            let disk_paths = [
                format!("brain/{}", doc.filename),
                format!("../brain/{}", doc.filename),
            ];
            let disk_content = disk_paths
                .iter()
                .find_map(|p| std::fs::read_to_string(p).ok());

            if let Some(content) = disk_content {
                let leaked = Box::leak(content.into_boxed_str());
                DocItem {
                    id: doc.id,
                    title: doc.title,
                    filename: doc.filename,
                    content: leaked,
                }
            } else {
                DocItem {
                    id: doc.id,
                    title: doc.title,
                    filename: doc.filename,
                    content: doc.content,
                }
            }
        })
        .collect()
}

/// Formats a raw markdown string into a clean, beautifully formatted reader text.
/// Strips raw symbols (#, ##, ###, ---, *, `, etc.) and replaces them with
/// elegant typography, section dividers, and bullet points so it wraps perfectly like the editor.
pub fn format_doc_for_reader(raw_md: &str) -> String {
    let mut out = String::new();
    let clean = raw_md.replace("\r\n", "\n").replace('\r', "\n");
    let mut in_code_block = false;

    for line in clean.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                out.push_str("┌─────────────────────────────────────────────────────────────\n");
            } else {
                out.push_str("└─────────────────────────────────────────────────────────────\n");
            }
            continue;
        }

        if in_code_block {
            out.push_str("│  ");
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            out.push_str("───────────────────────────────────────────────────────────────\n");
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("# ") {
            let clean_title = clean_inline_formatting(title.trim());
            let upper = clean_title.to_uppercase();
            out.push_str(&upper);
            out.push('\n');
            out.push_str("═══════════════════════════════════════════════════════════════\n");
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("## ") {
            let clean_title = clean_inline_formatting(title.trim());
            let upper = clean_title.to_uppercase();
            out.push_str("■ ");
            out.push_str(&upper);
            out.push('\n');
            out.push_str("───────────────────────────────────────────────────────────────\n");
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("### ") {
            let clean_title = clean_inline_formatting(title.trim());
            out.push_str("▶ ");
            out.push_str(&clean_title);
            out.push('\n');
            continue;
        }

        if let Some(quote) = trimmed.strip_prefix("> ") {
            let clean_quote = clean_inline_formatting(quote.trim());
            out.push_str("  │ ");
            out.push_str(&clean_quote);
            out.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            let clean_text = clean_inline_formatting(rest.trim());
            out.push_str("  ☐ ");
            out.push_str(&clean_text);
            out.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            let clean_text = clean_inline_formatting(rest.trim());
            out.push_str("  ☑ ");
            out.push_str(&clean_text);
            out.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- ") {
            let clean_text = clean_inline_formatting(rest.trim());
            out.push_str("  • ");
            out.push_str(&clean_text);
            out.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("* ") {
            let clean_text = clean_inline_formatting(rest.trim());
            out.push_str("  • ");
            out.push_str(&clean_text);
            out.push('\n');
            continue;
        }

        // Standard text line: clean inline markdown symbols
        let clean_line = clean_inline_formatting(line);
        out.push_str(&clean_line);
        out.push('\n');
    }

    out
}

/// Strips inline formatting such as **bold**, *italic*, `code`, and [links](url).
pub fn clean_inline_formatting(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                }
            }
            '`' => {}
            '[' => {
                let mut label = String::new();
                let mut found_close = false;
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc == ']' {
                        found_close = true;
                        break;
                    }
                    label.push(nc);
                }
                if found_close && chars.peek() == Some(&'(') {
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == ')' {
                            break;
                        }
                    }
                }
                out.push_str(&label);
            }
            _ => {
                out.push(c);
            }
        }
    }

    out
}

pub enum DocSidebarAction {
    SelectDoc(usize),
    BackToEditor,
}

/// Renders the dedicated documentation navigation sidebar on the left side of the doc view.
/// Allows users to click and navigate between guides without cluttering the main SQLite notes sidebar.
pub fn render_doc_sidebar(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    active_idx: usize,
    accent: Color32,
    text_color: Color32,
    muted_color: Color32,
) -> Option<DocSidebarAction> {
    let mut action = None;

    // Floating panel background with subtle border (5px round) - EXACT match to sidebar.rs
    painter.rect(
        rect,
        5.0,
        Color32::from_rgb(12, 12, 14),
        Stroke::new(1.0, Color32::from_rgb(34, 34, 40)),
        egui::StrokeKind::Inside,
    );

    let origin = rect.min + vec2(16.0, 18.0);

    // Header matching sidebar.rs
    painter.text(
        origin,
        Align2::LEFT_TOP,
        "MINDFORGE",
        FontId::monospace(15.0),
        accent,
    );
    painter.text(
        origin + vec2(0.0, 20.0),
        Align2::LEFT_TOP,
        "📖 User Guides",
        FontId::monospace(11.0),
        Color32::from_gray(100),
    );

    // Navigation items
    let doc_icons = ["📖", "⚡", "⌨", "⚔", "✨"];
    let start_y = origin.y + 48.0;
    let item_w = rect.width() - 32.0;

    for (idx, doc) in BRAIN_DOCS.iter().enumerate() {
        let item_rect = Rect::from_min_size(
            pos2(origin.x, start_y + idx as f32 * 32.0),
            vec2(item_w, 26.0),
        );
        let is_active = idx == active_idx;
        let is_hovered = ui.rect_contains_pointer(item_rect);

        if is_active || is_hovered {
            let bg = if is_active {
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35)
            } else {
                Color32::from_rgb(20, 20, 24)
            };
            painter.rect_filled(item_rect, 4.0, bg);

            if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                action = Some(DocSidebarAction::SelectDoc(idx));
            }
        }

        let icon = doc_icons.get(idx).copied().unwrap_or("📄");
        let label = format!("{} {}", icon, doc.title);
        painter.text(
            item_rect.min + vec2(8.0, 4.0),
            Align2::LEFT_TOP,
            label,
            FontId::monospace(12.0),
            if is_active { accent } else { text_color },
        );
    }

    // Bottom "Return to Notes" button
    let back_rect = Rect::from_min_size(
        pos2(origin.x, rect.max.y - 44.0),
        vec2(item_w, 26.0),
    );
    let back_hovered = ui.rect_contains_pointer(back_rect);

    if back_hovered {
        painter.rect_filled(back_rect, 4.0, Color32::from_rgb(28, 30, 36));
        if ui.input(|i| i.pointer.primary_clicked()) {
            action = Some(DocSidebarAction::BackToEditor);
        }
    } else {
        painter.rect(
            back_rect,
            4.0,
            Color32::from_rgb(18, 18, 22),
            Stroke::new(1.0, Color32::from_rgb(34, 34, 42)),
            egui::StrokeKind::Inside,
        );
    }

    painter.text(
        back_rect.center(),
        Align2::CENTER_CENTER,
        "← Return to Notes",
        FontId::monospace(11.5),
        if back_hovered { accent } else { muted_color },
    );

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_docs_embedded_and_valid() {
        assert_eq!(BRAIN_DOCS.len(), 5);
        for doc in BRAIN_DOCS {
            assert!(!doc.id.is_empty());
            assert!(!doc.title.is_empty());
            assert!(!doc.filename.is_empty());
            assert!(!doc.content.is_empty());
            assert!(doc.content.len() > 100);
            assert!(!doc.filename.contains('-'));
            assert!(!doc.filename.contains('_'));
        }
    }

    #[test]
    fn test_format_doc_for_reader() {
        let raw = "# Hello World\n\n## Subheading\n\n- Point 1\n- Point 2\n\nText with **bold**.";
        let formatted = format_doc_for_reader(raw);
        assert!(!formatted.contains("# Hello"));
        assert!(formatted.contains("HELLO WORLD"));
        assert!(formatted.contains("═════"));
        assert!(formatted.contains("• Point 1"));
        assert!(!formatted.contains("**bold**"));
        assert!(formatted.contains("bold"));
    }
}
