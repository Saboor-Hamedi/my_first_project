//! Settings modal tab navigation.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingTab {
    Carets,
    EditorMode,
    Sounds,
    Theme,
    Shortcuts,
    Keybindings,
    Backup,
    Updates,
    Ai,
}

impl SettingTab {
    pub const ALL: [SettingTab; 9] = [
        SettingTab::Carets,
        SettingTab::EditorMode,
        SettingTab::Sounds,
        SettingTab::Theme,
        SettingTab::Shortcuts,
        SettingTab::Keybindings,
        SettingTab::Backup,
        SettingTab::Updates,
        SettingTab::Ai,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            SettingTab::Carets => "Carets",
            SettingTab::EditorMode => "Editor Mode",
            SettingTab::Sounds => "Sounds",
            SettingTab::Theme => "Theme",
            SettingTab::Shortcuts => "Shortcuts",
            SettingTab::Keybindings => "Keybindings",
            SettingTab::Backup => "Backup",
            SettingTab::Updates => "Updates",
            SettingTab::Ai => "AI Agent",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingTab::Carets => "✨",
            SettingTab::EditorMode => "⚡",
            SettingTab::Sounds => "🔔",
            SettingTab::Theme => "🎨",
            SettingTab::Shortcuts => "⌨",
            SettingTab::Keybindings => "🔧",
            SettingTab::Backup => "💾",
            SettingTab::Updates => "🔄",
            SettingTab::Ai => "◈",
        }
    }
}

pub fn render_setting_tabs(
    ui: &egui::Ui,
    painter: &egui::Painter,
    tabs_rect: Rect,
    active_tab: &mut SettingTab,
    theme: &crate::theme::Theme,
) {
    // Left tab sidebar background with 8px rounded left corners
    painter.rect(
        tabs_rect,
        egui::CornerRadius {
            nw: 8,
            sw: 8,
            ne: 0,
            se: 0,
        },
        theme.sidebar_bg(),
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [tabs_rect.right_top(), tabs_rect.right_bottom()],
        Stroke::new(1.0, theme.border()),
    );

    let start_y = tabs_rect.min.y + 24.0;
    let tab_w = tabs_rect.width() - 16.0;
    let tab_h = 36.0;

    // "PREFERENCES" header
    painter.text(
        pos2(tabs_rect.min.x + 16.0, start_y),
        Align2::LEFT_TOP,
        "PREFERENCES",
        FontId::proportional(11.0),
        theme.muted,
    );

    for (i, &tab) in SettingTab::ALL.iter().enumerate() {
        let item_rect = Rect::from_min_size(
            pos2(tabs_rect.min.x + 8.0, start_y + 26.0 + i as f32 * (tab_h + 3.0)),
            vec2(tab_w, tab_h),
        );
        let is_sel = *active_tab == tab;
        let hovered = ui.rect_contains_pointer(item_rect);

        // Clean selection: left accent bar only, no filled background card
        if is_sel {
            let bar = Rect::from_min_size(
                pos2(item_rect.min.x + 2.0, item_rect.min.y + 7.0),
                vec2(3.5, item_rect.height() - 14.0),
            );
            painter.rect_filled(bar, 2.0, theme.accent);
        } else if hovered {
            painter.rect_filled(
                item_rect,
                6.0,
                if theme.is_light() {
                    Color32::from_rgba_unmultiplied(0, 0, 0, 8)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 8)
                },
            );
        }

        if hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            *active_tab = tab;
        }

        let (icon_color, label_color) = if is_sel {
            (theme.accent, theme.text)
        } else if hovered {
            (theme.accent, theme.text)
        } else {
            (theme.muted, theme.muted)
        };

        // Icon — clean glyph in accent color when selected (no boxes/badges)
        painter.text(
            pos2(item_rect.min.x + 16.0, item_rect.center().y - 1.0),
            Align2::LEFT_CENTER,
            tab.icon(),
            FontId::proportional(15.0),
            icon_color,
        );

        // Label — slightly right of icon
        painter.text(
            pos2(item_rect.min.x + 38.0, item_rect.center().y),
            Align2::LEFT_CENTER,
            tab.title(),
            FontId::proportional(13.0),
            label_color,
        );
    }
}
