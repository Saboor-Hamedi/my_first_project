//! Sleek autocomplete dropdown popup for Wikilinks as the user types `[[...`.

use crate::editor::Editor;
use crate::theme::Theme;
use core::Note;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

#[derive(Debug, Clone, Default)]
pub struct WikiLinkAutocompleteState {
    pub is_active: bool,
    /// Starting character index of `[[` in Editor::buf
    pub trigger_start: usize,
    /// Filter needle typed so far
    pub query: String,
    /// Filtered note matches
    pub filtered_items: Vec<AutocompleteItem>,
    pub selected_index: usize,
    /// Exact pixel position on screen right under the `[[` trigger
    pub trigger_screen_pos: Pos2,
}

#[derive(Debug, Clone)]
pub struct AutocompleteItem {
    pub id: i64,
    /// Clean display title for dropdown rendering
    pub display_title: String,
    /// Wikilink target text to insert between `[[` and `]]`
    pub insert_target: String,
    /// Short folder or domain badge
    pub folder_badge: String,
}

/// Formats a note topic into a clean human-readable title, insert target, and badge.
pub fn clean_note_topic_for_autocomplete(topic: &str) -> (String, String, String) {
    let clean = topic.trim();
    if clean.is_empty() {
        return (String::new(), String::new(), String::new());
    }

    // 1. If it's a web URL (http:// or https://)
    if clean.starts_with("http://") || clean.starts_with("https://") {
        let stripped = clean
            .strip_prefix("https://")
            .or_else(|| clean.strip_prefix("http://"))
            .unwrap_or(clean);
        let (domain, path) = stripped.split_once('/').unwrap_or((stripped, ""));
        let leaf = path.rsplit('/').next().unwrap_or(domain);
        let leaf_clean = if leaf.is_empty() { domain } else { leaf };
        let leaf_no_ext = leaf_clean.strip_suffix(".md").unwrap_or(leaf_clean);
        let domain_short = if domain.len() > 16 {
            format!("{}...", &domain[..13])
        } else {
            domain.to_string()
        };
        let display = format_friendly_title(leaf_no_ext, 26);
        return (display, clean.to_string(), domain_short);
    }

    // 2. Check for domain-like text with loose dots (e.g. `supabase.co` or `. supabase. co`)
    let normalized_dots = clean.replace(" . ", ".").replace(". ", ".").replace(" .", ".");
    let domain_badge = extract_domain_badge(&normalized_dots);

    // 3. Normalized path check
    let normalized = clean.replace('\\', "/");
    let (folder_badge, leaf_raw) = if let Some((dir, leaf)) = normalized.rsplit_once('/') {
        let last_folder = dir.rsplit('/').next().unwrap_or(dir);
        let f_badge = if last_folder.len() > 16 {
            format!("{}...", &last_folder[..13])
        } else {
            last_folder.to_string()
        };
        (f_badge, leaf)
    } else if !domain_badge.is_empty() {
        (domain_badge, clean)
    } else {
        (String::new(), clean)
    };

    let leaf_no_ext = leaf_raw.strip_suffix(".md").unwrap_or(leaf_raw);
    let display = format_friendly_title(leaf_no_ext, 26);

    (display, clean.to_string(), folder_badge)
}

/// Cleans up noisy strings (unbalanced parens, huge JWT/API tokens, spaced dots)
/// and formats a friendly, readable label clamped to `max_len` characters.
pub fn format_friendly_title(raw: &str, max_len: usize) -> String {
    let collapsed: String = raw
        .split_whitespace()
        .filter(|w| *w != ")" && *w != "(" && *w != ".")
        .collect::<Vec<_>>()
        .join(" ");

    // Shorten any ultra-long token (>16 chars) inside the text to prevent token explosions
    let mut words = Vec::new();
    for word in collapsed.split_whitespace() {
        let clean_word = word.trim_matches(|c| c == ')' || c == '(' || c == ']' || c == '[' || c == ',');
        if clean_word.is_empty() {
            continue;
        }
        if clean_word.len() > 16 && !clean_word.contains('/') {
            // Long hash / token / slug
            words.push(format!("{}...", &clean_word[..10]));
        } else {
            words.push(clean_word.to_string());
        }
    }

    let joined = words.join(" ");
    let title = if joined.is_empty() {
        raw.trim().to_string()
    } else {
        joined
    };

    truncate_clean_title(&title, max_len)
}

pub fn truncate_clean_title(title: &str, max_len: usize) -> String {
    let chars: Vec<char> = title.chars().collect();
    if chars.len() > max_len {
        let take = max_len.saturating_sub(3);
        format!("{}...", chars[..take].iter().collect::<String>())
    } else {
        title.to_string()
    }
}

pub fn extract_domain_badge(text: &str) -> String {
    for word in text.split_whitespace() {
        let clean = word.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']' || c == ',' || c == '"' || c == '\'');
        if clean.contains(".co") || clean.contains(".com") || clean.contains(".io") || clean.contains(".org") || clean.contains(".dev") || clean.contains(".net") {
            let domain = clean.rsplit('/').next().unwrap_or(clean);
            return if domain.len() > 16 {
                format!("{}...", &domain[..13])
            } else {
                domain.to_string()
            };
        }
    }
    String::new()
}

impl WikiLinkAutocompleteState {
    pub fn clear(&mut self) {
        self.is_active = false;
        self.trigger_start = 0;
        self.query.clear();
        self.filtered_items.clear();
        self.selected_index = 0;
    }

    /// Checks editor text before caret for an open `[[` without a closing `]]`.
    pub fn check_trigger(&mut self, ed: &Editor, notes: &[Note]) {
        let cur = ed.cur;
        if cur == 0 || cur > ed.buf.len() {
            self.clear();
            return;
        }

        // Look back up to 60 characters for `[[` on the current line
        let lookback = cur.saturating_sub(60);
        let slice = &ed.buf[lookback..cur];

        // Ensure we don't cross a newline
        let mut trigger_rel = None;
        for i in (0..slice.len().saturating_sub(1)).rev() {
            if slice[i] == '\n' {
                break;
            }
            if slice[i] == '[' && slice[i + 1] == '[' {
                trigger_rel = Some(i);
                break;
            }
        }

        if let Some(rel) = trigger_rel {
            let after_trigger = rel + 2;
            let typed_slice = &slice[after_trigger..];
            // If already closed by `]]` before caret, not active
            let is_closed = typed_slice.windows(2).any(|w| w[0] == ']' && w[1] == ']');
            if is_closed {
                self.clear();
                return;
            }

            self.is_active = true;
            self.trigger_start = lookback + rel;
            self.query = typed_slice.iter().collect();

            self.update_filtered(notes);
        } else {
            self.clear();
        }
    }

    pub fn update_filtered(&mut self, notes: &[Note]) {
        let needle = self.query.trim().to_lowercase();
        let mut items = Vec::new();

        if needle.is_empty() {
            // When query is empty, show top 6 recent notes
            for note in notes.iter().take(6) {
                let (display_title, insert_target, folder_badge) =
                    clean_note_topic_for_autocomplete(&note.topic);

                items.push(AutocompleteItem {
                    id: note.id,
                    display_title,
                    insert_target,
                    folder_badge,
                });
            }
        } else {
            // When query is typed (e.g. [[we search]]), strictly filter by title or insert_target
            for note in notes {
                let (display_title, insert_target, folder_badge) =
                    clean_note_topic_for_autocomplete(&note.topic);
                let title_lower = display_title.to_lowercase();
                let topic_lower = note.topic.to_lowercase();

                if title_lower.contains(&needle) || topic_lower.contains(&needle) {
                    items.push(AutocompleteItem {
                        id: note.id,
                        display_title,
                        insert_target,
                        folder_badge,
                    });
                }
            }

            // If no existing note matches the search query, provide option to create it
            if items.is_empty() {
                let clean_query = self.query.trim().to_string();
                items.push(AutocompleteItem {
                    id: 0,
                    display_title: clean_query.clone(),
                    insert_target: clean_query,
                    folder_badge: "New note".to_string(),
                });
            }
        }

        self.filtered_items = items;
        if self.selected_index >= self.filtered_items.len() {
            self.selected_index = 0;
        }
    }
}

pub enum AutocompleteAction {
    Inserted { inserted_text: String },
}

/// Applies autocomplete selection into the editor buffer, replacing the typed query
/// and ensuring no duplicate `]]` are created if `[[]]` was already open.
pub fn apply_autocomplete_insertion(
    ed: &mut Editor,
    trigger_start: usize,
    target: &str,
) -> String {
    let replace_start = trigger_start + 2;
    let replace_end = ed.cur;

    if replace_start <= replace_end && replace_end <= ed.buf.len() {
        // Look ahead past cursor to see if `]]` or `]` already exists
        let has_double_close = replace_end + 1 < ed.buf.len()
            && ed.buf[replace_end] == ']'
            && ed.buf[replace_end + 1] == ']';

        let has_single_close = !has_double_close
            && replace_end < ed.buf.len()
            && ed.buf[replace_end] == ']';

        ed.save_undo_snapshot();

        if has_double_close {
            // `]]` already exists right after cursor — replace typed query only
            // and advance caret past the existing `]]`
            ed.buf.drain(replace_start..replace_end);
            for (idx, ch) in target.chars().enumerate() {
                ed.buf.insert(replace_start + idx, ch);
            }
            ed.cur = replace_start + target.chars().count() + 2;
        } else if has_single_close {
            // Single `]` exists — consume it and close cleanly with `]]`
            ed.buf.drain(replace_start..=replace_end);
            let replacement = format!("{target}]]");
            for (idx, ch) in replacement.chars().enumerate() {
                ed.buf.insert(replace_start + idx, ch);
            }
            ed.cur = replace_start + replacement.chars().count();
        } else {
            // Unclosed `[[` — insert target and close with `]]`
            ed.buf.drain(replace_start..replace_end);
            let replacement = format!("{target}]]");
            for (idx, ch) in replacement.chars().enumerate() {
                ed.buf.insert(replace_start + idx, ch);
            }
            ed.cur = replace_start + replacement.chars().count();
        }
    }

    target.to_string()
}

/// Renders the autocomplete dropdown right under `[[` and handles arrow/ctrl+j,k/enter navigation.
pub fn render_wikilink_autocomplete(
    ui: &egui::Ui,
    painter: &egui::Painter,
    state: &mut WikiLinkAutocompleteState,
    ed: &mut Editor,
    theme: &Theme,
    window_bounds: Rect,
) -> Option<AutocompleteAction> {
    if !state.is_active || state.filtered_items.is_empty() {
        return None;
    }

    let item_h = 28.0;
    let max_visible = 6.min(state.filtered_items.len());
    let list_h = max_visible as f32 * item_h + 12.0;
    let menu_w = 320.0;

    let mut menu_x = state.trigger_screen_pos.x;
    let mut menu_y = state.trigger_screen_pos.y;

    if menu_x + menu_w > window_bounds.max.x - 10.0 {
        menu_x = (window_bounds.max.x - menu_w - 10.0).max(window_bounds.min.x + 10.0);
    }
    if menu_y + list_h > window_bounds.max.y - 30.0 {
        menu_y = (state.trigger_screen_pos.y - list_h - 24.0).max(window_bounds.min.y + 35.0);
    }

    let menu_rect = Rect::from_min_size(pos2(menu_x, menu_y), vec2(menu_w, list_h));

    // Handle keyboard navigation before rendering
    let (nav_up, nav_down, nav_enter, nav_esc) = ui.input_mut(|i| {
        let ctrl_k = i.modifiers.ctrl && i.key_pressed(egui::Key::K);
        let ctrl_j = i.modifiers.ctrl && i.key_pressed(egui::Key::J);
        let up = i.key_pressed(egui::Key::ArrowUp) || ctrl_k;
        let down = i.key_pressed(egui::Key::ArrowDown) || ctrl_j;
        let enter = i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab);
        let esc = i.key_pressed(egui::Key::Escape);

        if up {
            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp);
            i.consume_key(egui::Modifiers::CTRL, egui::Key::K);
        }
        if down {
            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown);
            i.consume_key(egui::Modifiers::CTRL, egui::Key::J);
        }
        if enter {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        }

        (up, down, enter, esc)
    });

    if nav_esc {
        state.clear();
        return None;
    }

    if nav_up && !state.filtered_items.is_empty() {
        if state.selected_index > 0 {
            state.selected_index -= 1;
        } else {
            state.selected_index = state.filtered_items.len().saturating_sub(1);
        }
    }
    if nav_down && !state.filtered_items.is_empty() {
        if state.selected_index + 1 < state.filtered_items.len() {
            state.selected_index += 1;
        } else {
            state.selected_index = 0;
        }
    }

    if nav_enter {
        if let Some(item) = state.filtered_items.get(state.selected_index) {
            let target = item.insert_target.clone();
            let inserted = apply_autocomplete_insertion(ed, state.trigger_start, &target);
            state.clear();
            return Some(AutocompleteAction::Inserted { inserted_text: inserted });
        }
    }

    // Outer surface
    let shadow_color = Color32::from_black_alpha(45);
    painter.rect_filled(menu_rect.translate(vec2(0.0, 3.0)), 6.0, shadow_color);
    painter.rect(
        menu_rect,
        6.0,
        theme.surface(),
        Stroke::new(1.0, theme.border()),
        egui::StrokeKind::Inside,
    );

    // Render items
    let mut cur_y = menu_rect.min.y + 6.0;
    for (idx, item) in state.filtered_items.iter().take(max_visible).enumerate() {
        let is_selected = idx == state.selected_index;
        let item_rect = Rect::from_min_size(pos2(menu_rect.min.x + 6.0, cur_y), vec2(menu_w - 12.0, item_h));

        let is_hovered = ui.rect_contains_pointer(item_rect);
        if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            state.selected_index = idx;
            let target = item.insert_target.clone();
            let inserted = apply_autocomplete_insertion(ed, state.trigger_start, &target);
            state.clear();
            return Some(AutocompleteAction::Inserted { inserted_text: inserted });
        }

        // Left icon
        let is_folder = item.insert_target.contains('/') || item.insert_target.contains('\\');
        let icon_name = if item.id == 0 {
            "plus"
        } else if is_folder {
            "folder"
        } else {
            "doc"
        };
        let icon_rect = Rect::from_center_size(pos2(item_rect.min.x + 14.0, item_rect.center().y), vec2(13.0, 13.0));
        crate::ui_components::render_vector_icon(
            painter,
            icon_name,
            icon_rect,
            if is_selected { theme.accent } else { theme.muted },
        );

        // Title: Text only highlight (NO background box, clean title ONLY without metadata clutter)
        let text_color = if is_selected || is_hovered {
            theme.accent
        } else {
            theme.text
        };

        // Ensure title text has full readable space inside the menu
        let max_text_w = (item_rect.max.x - 12.0 - (item_rect.min.x + 30.0)).max(60.0);
        let max_chars = ((max_text_w / 7.2) as usize).max(8);
        let title_to_show = truncate_clean_title(&item.display_title, max_chars);

        painter.text(
            pos2(item_rect.min.x + 30.0, item_rect.center().y),
            Align2::LEFT_CENTER,
            title_to_show,
            FontId::monospace(11.5),
            text_color,
        );

        cur_y += item_h;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_clean_note_topic_messy_tokens_and_urls() {
        let messy = "foldevzzxzlksrnhepytzaqqf . supabase. co ) SUPABASE_ is hable_-aJBCtxX8A5ULZK6QKy9DA_e7eTMpbt";
        let (display, insert, badge) = clean_note_topic_for_autocomplete(messy);

        // Display title should be friendly, short, and never overflow
        assert!(display.len() <= 28);
        assert!(!display.contains("foldevzzxzlksrnhepytzaqqf")); // shortened long token
        assert!(!display.contains("-aJBCtxX8A5ULZK6QKy9DA_e7eTMpbt")); // shortened long token
        // Full raw topic is preserved for link insertion
        assert_eq!(insert, messy);
        // Recognizes domain badge
        assert!(!badge.is_empty());
    }

    #[test]
    fn test_wikilink_autocomplete_trigger_and_filtering() {
        let now = Local::now().naive_local();
        let notes = vec![
            Note { id: 1, topic: "Architecture".into(), body: "".into(), struggled_with: None, created_at: now },
            Note { id: 2, topic: "docs/networking".into(), body: "".into(), struggled_with: None, created_at: now },
            Note { id: 3, topic: "Personal Notes".into(), body: "".into(), struggled_with: None, created_at: now },
        ];

        let mut ed = Editor::new();
        ed.set_text("Hello see [[arch");
        ed.cur = ed.buf.len();

        let mut state = WikiLinkAutocompleteState::default();
        state.check_trigger(&ed, &notes);

        assert!(state.is_active);
        assert_eq!(state.query, "arch");
        assert_eq!(state.filtered_items.len(), 1);
        assert_eq!(state.filtered_items[0].display_title, "Architecture");

        // Closed wikilink should not trigger
        ed.set_text("Hello see [[arch]] and more");
        ed.cur = ed.buf.len();
        state.check_trigger(&ed, &notes);
        assert!(!state.is_active);
    }

    #[test]
    fn test_wikilink_autocomplete_prevents_duplicate_closing_brackets() {
        let mut ed = Editor::new();
        // User typed [[ within auto-paired [[]]
        ed.set_text("Hello [[arc]] world");
        ed.cur = 11; // right after "arc"

        let target = "Architecture";
        apply_autocomplete_insertion(&mut ed, 6, target);

        // Should NOT have duplicate ]]
        assert_eq!(ed.text(), "Hello [[Architecture]] world");
        assert_eq!(ed.cur, "Hello [[Architecture]]".len());

        // Now test completely unclosed [[
        let mut ed2 = Editor::new();
        ed2.set_text("Hello [[arc");
        ed2.cur = ed2.buf.len();
        apply_autocomplete_insertion(&mut ed2, 6, target);
        assert_eq!(ed2.text(), "Hello [[Architecture]]");
        assert_eq!(ed2.cur, "Hello [[Architecture]]".len());

        // Test single closing ]
        let mut ed3 = Editor::new();
        ed3.set_text("Hello [[arc] world");
        ed3.cur = 11;
        apply_autocomplete_insertion(&mut ed3, 6, target);
        assert_eq!(ed3.text(), "Hello [[Architecture]] world");
    }
}
