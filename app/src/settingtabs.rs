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
            SettingTab::Carets => "✦",
            SettingTab::EditorMode => "⚡",
            SettingTab::Sounds => "♪",
            SettingTab::Theme => "◐",
            SettingTab::Shortcuts => "⌨",
            SettingTab::Backup => "💾",
            SettingTab::Updates => "⟳",
        }
    }
}


pub fn render_setting_tabs(
    ui: &egui::Ui,
    painter: &egui::Painter,
    tabs_rect: Rect,
    active_tab: &mut SettingTab,
    accent: Color32,
) {
    // Left tab sidebar background with 5px rounded left corners
    painter.rect(
        tabs_rect,
        egui::CornerRadius {
            nw: 5,
            sw: 5,
            ne: 0,
            se: 0,
        },
        Color32::from_rgb(11, 12, 15),
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [tabs_rect.right_top(), tabs_rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(30, 32, 40)),
    );

    let start_y = tabs_rect.min.y + 24.0;
    let tab_w = tabs_rect.width() - 16.0;
    let tab_h = 42.0;

    // "PREFERENCES" header — higher contrast
    painter.text(
        pos2(tabs_rect.min.x + 16.0, start_y),
        Align2::LEFT_TOP,
        "PREFERENCES",
        FontId::monospace(10.5),
        Color32::from_gray(155),
    );

    for (i, &tab) in SettingTab::ALL.iter().enumerate() {
        let item_rect = Rect::from_min_size(
            pos2(tabs_rect.min.x + 8.0, start_y + 30.0 + i as f32 * (tab_h + 4.0)),
            vec2(tab_w, tab_h),
        );
        let is_sel = *active_tab == tab;
        let hovered = ui.rect_contains_pointer(item_rect);

        // Background — hover only, no filled card on selected
        if hovered && !is_sel {
            painter.rect_filled(
                item_rect,
                6.0,
                Color32::from_rgb(18, 20, 26),
            );
        }

        if is_sel {
            // Left accent border only (no background card)
            let bar = Rect::from_min_size(
                pos2(item_rect.min.x + 2.0, item_rect.min.y + 8.0),
                vec2(3.5, item_rect.height() - 16.0),
            );
            painter.rect_filled(bar, 2.0, accent);
        }

        if hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            *active_tab = tab;
        }

        let (icon_color, label_color) = if is_sel {
            (accent, Color32::WHITE)
        } else if hovered {
            (Color32::from_gray(200), Color32::from_gray(210))
        } else {
            (Color32::from_gray(100), Color32::from_gray(145))
        };

        // Icon — larger, in accent color when selected
        painter.text(
            pos2(item_rect.min.x + 16.0, item_rect.center().y - 1.0),
            Align2::LEFT_CENTER,
            tab.icon(),
            FontId::monospace(15.0),
            icon_color,
        );

        // Label — slightly right of icon
        painter.text(
            pos2(item_rect.min.x + 36.0, item_rect.center().y),
            Align2::LEFT_CENTER,
            tab.title(),
            FontId::monospace(12.5),
            label_color,
        );
    }
}
