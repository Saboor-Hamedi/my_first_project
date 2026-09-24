//! Neovim-style minimalist welcome dashboard and quick-action launcher.

use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Key, Rect, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardAction {
    NewNote,
    FindNote,
    OpenRecent(i64),
    OpenTerminal,
    OpenAi,
    OpenDocs,
    OpenSettings,
    ToggleZen,
    Quit,
}

const ASCII_LOGO: [&str; 6] = [
    r" __  __ _           _ _____                    ",
    r"|  \/  (_)_ __   __| |  ___|__  _ __ __ _  ___ ",
    r"| |\/| | | '_ \ / _` | |_ / _ \| '__/ _` |/ _ \",
    r"| |  | | | | | | (_| |  _| (_) | | | (_| |  __/",
    r"|_|  |_|_|_| |_|\__,_|_|  \___/|_|  \__, |\___|",
    r"                                    |___/      ",
];

pub fn render_welcome_dashboard(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    theme: &Theme,
    total_notes: usize,
) -> Option<DashboardAction> {
    let mut action = None;
    let center = rect.center();

    // ── 1. Keyboard shortcuts dispatch for dashboard ────────────────────────
    ui.input(|i| {
        if i.key_pressed(Key::N) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::NewNote);
        } else if i.key_pressed(Key::F) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::FindNote);
        } else if i.key_pressed(Key::T) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::OpenTerminal);
        } else if i.key_pressed(Key::A) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::OpenAi);
        } else if i.key_pressed(Key::D) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::OpenDocs);
        } else if i.key_pressed(Key::S) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::OpenSettings);
        } else if i.key_pressed(Key::Z) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::ToggleZen);
        } else if i.key_pressed(Key::Q) && !i.modifiers.ctrl && !i.modifiers.alt {
            action = Some(DashboardAction::Quit);
        }
    });

    if action.is_some() {
        return action;
    }

    // ── 2. ASCII Logo Header (Bigger, Bold, Clean) ─────────────────────────
    let logo_line_h = 16.5;
    let total_logo_h = (ASCII_LOGO.len() as f32) * logo_line_h;
    let btn_h = 28.0;
    let btn_gap = 6.0;
    let total_buttons_h = 7.0 * (btn_h + btn_gap);
    let total_content_h = total_logo_h + 24.0 + total_buttons_h;

    let logo_start_y = (center.y - total_content_h * 0.48).max(rect.min.y + 12.0);

    for (idx, line) in ASCII_LOGO.iter().enumerate() {
        let y = logo_start_y + idx as f32 * logo_line_h;
        painter.text(
            pos2(center.x, y),
            Align2::CENTER_CENTER,
            *line,
            FontId::monospace(14.0),
            theme.accent,
        );
    }

    // ── 3. Subtitle / Metadata ──────────────────────────────────────────────
    let sub_y = logo_start_y + total_logo_h + 8.0;
    let subtitle = format!("MINDFORGE  •  Fast, Minimalist, Local-First Notes  •  {} notes", total_notes);
    painter.text(
        pos2(center.x, sub_y),
        Align2::CENTER_CENTER,
        subtitle,
        FontId::monospace(11.5),
        theme.muted,
    );

    // ── 4. Action Buttons (Neovim-style bracket shortcuts) ───────────────────
    let actions_start_y = sub_y + 24.0;
    let btn_w = 340.0f32.min(rect.width() - 40.0);

    let action_items: [(&str, &str, &str, DashboardAction); 7] = [
        ("n", "New Note", "Ctrl+N", DashboardAction::NewNote),
        ("f", "Find Note", "Ctrl+P", DashboardAction::FindNote),
        ("t", "Embedded Terminal", ":term", DashboardAction::OpenTerminal),
        ("a", "AI Assistant", "Ctrl+J", DashboardAction::OpenAi),
        ("d", "Documentation Guides", ":doc", DashboardAction::OpenDocs),
        ("s", "Preferences & Themes", "Ctrl+,", DashboardAction::OpenSettings),
        ("z", "Zen Focus Mode", "Ctrl+.", DashboardAction::ToggleZen),
    ];

    for (idx, (key, label, hint, act)) in action_items.iter().enumerate() {
        let btn_rect = Rect::from_center_size(
            pos2(center.x, actions_start_y + idx as f32 * (btn_h + btn_gap)),
            vec2(btn_w, btn_h),
        );
        let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
        let hovered = resp.hovered() || ui.rect_contains_pointer(btn_rect);

        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            painter.rect(
                btn_rect,
                4.0,
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 20),
                Stroke::new(1.0, theme.border()),
                egui::StrokeKind::Inside,
            );
        }

        // Bracket key trigger
        painter.text(
            pos2(btn_rect.min.x + 12.0, btn_rect.center().y),
            Align2::LEFT_CENTER,
            format!("[{}]", key),
            FontId::monospace(12.0),
            theme.accent,
        );

        // Action label
        painter.text(
            pos2(btn_rect.min.x + 48.0, btn_rect.center().y),
            Align2::LEFT_CENTER,
            *label,
            FontId::monospace(12.0),
            if hovered { theme.text } else { theme.muted },
        );

        // Shortcut hint
        painter.text(
            pos2(btn_rect.max.x - 12.0, btn_rect.center().y),
            Align2::RIGHT_CENTER,
            *hint,
            FontId::monospace(11.0),
            theme.border(),
        );

        if resp.clicked() || (hovered && ui.input(|i| i.pointer.primary_clicked())) {
            action = Some(*act);
        }
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_logo_non_empty() {
        assert_eq!(ASCII_LOGO.len(), 6);
        for line in ASCII_LOGO {
            assert!(!line.trim().is_empty());
        }
    }

    #[test]
    fn test_dashboard_action_variants() {
        let act = DashboardAction::NewNote;
        assert_eq!(act, DashboardAction::NewNote);
        assert_ne!(act, DashboardAction::FindNote);
    }
}
