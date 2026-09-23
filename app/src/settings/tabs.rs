//! Settings modal tab navigation.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingTab {
    Carets,
    EditorMode,
    Sounds,
    Theme,
    Shortcuts,
    Backup,
    Updates,
}

impl SettingTab {
    pub const ALL: [SettingTab; 7] = [
        SettingTab::Carets,
        SettingTab::EditorMode,
        SettingTab::Sounds,
        SettingTab::Theme,
        SettingTab::Shortcuts,
        SettingTab::Backup,
        SettingTab::Updates,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            SettingTab::Carets => "Carets",
            SettingTab::EditorMode => "Editor Mode",
            SettingTab::Sounds => "Sounds",
            SettingTab::Theme => "Theme",
            SettingTab::Shortcuts => "Shortcuts",
            SettingTab::Backup => "Backup",
            SettingTab::Updates => "Updates",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingTab::Carets => "✨",
            SettingTab::EditorMode => "⚡",
            SettingTab::Sounds => "🔔",
            SettingTab::Theme => "🎨",
            SettingTab::Shortcuts => "⌨",
            SettingTab::Backup => "💾",
            SettingTab::Updates => "🔄",
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
    let tab_h = 42.0;

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
            pos2(tabs_rect.min.x + 8.0, start_y + 30.0 + i as f32 * (tab_h + 4.0)),
            vec2(tab_w, tab_h),
        );
        let is_sel = *active_tab == tab;
        let hovered = ui.rect_contains_pointer(item_rect);

        // Background card
        if is_sel {
            painter.rect_filled(
                item_rect,
                6.0,
                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 26),
            );
            // Left accent border bar
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
                    Color32::from_rgba_unmultiplied(0, 0, 0, 10)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12)
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

        // Icon — larger, in accent color when selected
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
