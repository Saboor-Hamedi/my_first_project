//! Tab bar widget for multi-note and documentation views.
//!
//! Renders as the top strip inside the editor panel card.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub const TAB_ROW_H: f32 = 35.0;

pub enum TabAction {
    Select(usize),
    Close(usize),
}

pub struct TabItem<'a> {
    pub title: &'a str,
    pub is_dirty: bool,
    pub is_active: bool,
}

/// Renders the tab bar strip at the top of the editor panel.
pub fn render_tab_bar(
    ui: &egui::Ui,
    painter: &egui::Painter,
    tab_bar_rect: Rect,
    tabs: &[TabItem],
    theme: &Theme,
    scroll_offset: &mut f32,
) -> Option<TabAction> {
    if tabs.is_empty() {
        return None;
    }

    let mut action = None;
    let clip_painter = painter.with_clip_rect(tab_bar_rect);

    // Subtle bottom divider line separating tabs from editor/preview split
    painter.line_segment(
        [
            pos2(tab_bar_rect.min.x, tab_bar_rect.max.y),
            pos2(tab_bar_rect.max.x, tab_bar_rect.max.y),
        ],
        Stroke::new(1.0, Color32::from_rgb(28, 30, 38)),
    );

    let tab_gap = 2.0;
    let tab_h = tab_bar_rect.height() - 2.0;
    let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
    let mouse_pos = ui.input(|i| i.pointer.interact_pos());
    let mouse_in_bar = ui.rect_contains_pointer(tab_bar_rect);

    // 1. Measure all tabs and calculate relative positions
    let mut tab_widths: Vec<f32> = Vec::with_capacity(tabs.len());
    let mut display_titles: Vec<String> = Vec::with_capacity(tabs.len());

    for tab in tabs {
        let title_clean = if tab.title.trim().is_empty() {
            "Untitled"
        } else {
            tab.title
        };
        let max_chars = 22;
        let display_title = if title_clean.chars().count() > max_chars {
            format!("{}...", title_clean.chars().take(max_chars.saturating_sub(3)).collect::<String>())
        } else {
            title_clean.to_string()
        };

        let text_w = display_title.len() as f32 * 7.2;
        let tab_w = (text_w + 50.0).clamp(94.0, 210.0);
        tab_widths.push(tab_w);
        display_titles.push(display_title);
    }

    // Relative start and end offsets from content origin (0.0 = first tab left edge)
    let mut tab_positions: Vec<(f32, f32)> = Vec::with_capacity(tabs.len());
    let mut current_offset = 0.0;
    for &w in &tab_widths {
        tab_positions.push((current_offset, current_offset + w));
        current_offset += w + tab_gap;
    }
    let total_content_w = current_offset - tab_gap;
    let viewport_w = tab_bar_rect.width();
    let max_scroll = (total_content_w - viewport_w).max(0.0);

    // 2. Mouse wheel scrolling on tab bar
    if mouse_in_bar {
        let raw_scroll = ui.input(|i| i.raw_scroll_delta);
        let delta = if raw_scroll.x.abs() > 0.1 {
            raw_scroll.x
        } else if raw_scroll.y.abs() > 0.1 {
            raw_scroll.y
        } else {
            0.0
        };
        if delta.abs() > 0.1 {
            *scroll_offset -= delta;
        }
    }

    // 3. Auto-scroll to keep active tab fully in view
    if let Some(active_idx) = tabs.iter().position(|t| t.is_active) {
        let (active_start, active_end) = tab_positions[active_idx];
        if active_end > *scroll_offset + viewport_w {
            *scroll_offset = (active_end - viewport_w + 8.0).min(max_scroll);
        }
        if active_start < *scroll_offset {
            *scroll_offset = active_start.max(0.0);
        }
    }

    *scroll_offset = (*scroll_offset).clamp(0.0, max_scroll);

    // 4. Render visible tabs
    for (idx, tab) in tabs.iter().enumerate() {
        let (rel_start, rel_end) = tab_positions[idx];
        let tab_w = rel_end - rel_start;
        // First tab has zero gap from the left edge of the tab bar
        let tab_x = tab_bar_rect.min.x + rel_start - *scroll_offset;

        let tab_rect = Rect::from_min_size(
            pos2(tab_x, tab_bar_rect.min.y + 1.0),
            vec2(tab_w, tab_h),
        );

        // Skip completely off-screen tabs without breaking the loop
        if tab_rect.max.x < tab_bar_rect.min.x || tab_rect.min.x > tab_bar_rect.max.x {
            continue;
        }

        let is_tab_hovered = mouse_in_bar && ui.rect_contains_pointer(tab_rect);

        // Close button: taller rounded button almost matching tab interior height
        let btn_w = 20.0;
        let btn_h = (tab_h - 10.0).clamp(20.0, 24.0);
        let close_rect = Rect::from_center_size(
            pos2(tab_rect.max.x - 15.0, tab_rect.center().y),
            vec2(btn_w, btn_h),
        );
        let is_close_hovered = mouse_in_bar && ui.rect_contains_pointer(close_rect);

        // Tab surface styling
        if tab.is_active {
            // Elevated active tab card
            clip_painter.rect_filled(
                tab_rect,
                4.0,
                Color32::from_rgb(20, 22, 28),
            );
            // Accent bottom indicator line
            clip_painter.line_segment(
                [
                    pos2(tab_rect.min.x + 4.0, tab_rect.max.y),
                    pos2(tab_rect.max.x - 4.0, tab_rect.max.y),
                ],
                Stroke::new(2.0, theme.accent),
            );
        } else if is_tab_hovered {
            // Subtle, soft faded hover background
            clip_painter.rect_filled(
                tab_rect,
                4.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 6),
            );
        }

        // Title and dirty indicator
        let text_color = if tab.is_active {
            theme.text
        } else if is_tab_hovered {
            Color32::from_gray(165)
        } else {
            theme.muted
        };

        let label_text = if tab.is_dirty {
            format!("{} ●", display_titles[idx])
        } else {
            display_titles[idx].clone()
        };

        let label_pos = pos2(tab_rect.min.x + 10.0, tab_rect.center().y);
        clip_painter.text(
            label_pos,
            Align2::LEFT_CENTER,
            label_text,
            FontId::monospace(11.5),
            text_color,
        );

        // Close button (×)
        if tabs.len() > 1 || tab.is_dirty {
            let close_color = if is_close_hovered {
                Color32::from_rgb(245, 105, 105)
            } else if tab.is_active {
                Color32::from_gray(140)
            } else if is_tab_hovered {
                Color32::from_gray(115)
            } else {
                Color32::from_gray(75)
            };

            if is_close_hovered {
                // Slightly bolder background on close button hover than the tab hover
                clip_painter.rect_filled(
                    close_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                );
            }

            clip_painter.text(
                close_rect.center(),
                Align2::CENTER_CENTER,
                "×",
                FontId::monospace(13.0),
                close_color,
            );
        }

        // Mouse interaction
        if primary_clicked && mouse_in_bar {
            if let Some(pos) = mouse_pos {
                if tab_bar_rect.contains(pos) {
                    if close_rect.contains(pos) && (tabs.len() > 1 || tab.is_dirty) {
                        action = Some(TabAction::Close(idx));
                    } else if tab_rect.contains(pos) {
                        action = Some(TabAction::Select(idx));
                    }
                }
            }
        }

        // Small divider tick between tabs
        if !tab.is_active && idx + 1 < tabs.len() && !tabs[idx + 1].is_active {
            let tick_x = tab_rect.max.x + 1.0;
            let tick_mid_y = tab_bar_rect.center().y;
            clip_painter.line_segment(
                [pos2(tick_x, tick_mid_y - 6.0), pos2(tick_x, tick_mid_y + 6.0)],
                Stroke::new(1.0, Color32::from_gray(38)),
            );
        }
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_row_height_constant() {
        assert_eq!(TAB_ROW_H, 35.0);
    }

    #[test]
    fn test_tab_item_properties() {
        let item = TabItem {
            title: "Rust Architecture",
            is_dirty: true,
            is_active: false,
        };
        assert_eq!(item.title, "Rust Architecture");
        assert!(item.is_dirty);
        assert!(!item.is_active);
    }

    #[test]
    fn test_tab_auto_scroll_clamps() {
        let mut scroll = 100.0f32;
        let viewport_w = 400.0f32;
        let total_w = 300.0f32;
        let max_scroll = (total_w - viewport_w).max(0.0);
        scroll = scroll.clamp(0.0, max_scroll);
        assert_eq!(scroll, 0.0);
    }
}
