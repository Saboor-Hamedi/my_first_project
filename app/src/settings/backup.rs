//! Database backup and snapshot settings tab.

use super::SettingPanelAction;
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};

pub fn render_backup_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    backup_dir: &mut String,
    last_backup_status: Option<&str>,
    theme: &Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) -> Option<SettingPanelAction> {
    let mut action = None;

    // Header
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "Auto-Backup & Snapshots",
        FontId::monospace(15.0),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Atomic SQLite database snapshots with WAL write-protection",
        FontId::monospace(11.0),
        theme.muted,
    );

    let mut cur_y = p_origin.y + 60.0;
    let card_w = panel_rect.width() - 56.0;

    // Card 1: Backup Directory Path
    let path_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 90.0));
    painter.rect(
        path_card,
        5.0,
        Color32::from_rgb(18, 20, 26),
        Stroke::new(1.0, Color32::from_rgb(34, 37, 48)),
        egui::StrokeKind::Inside,
    );

    painter.text(
        path_card.min + vec2(14.0, 12.0),
        Align2::LEFT_TOP,
        "Backup Destination Directory",
        FontId::monospace(12.5),
        Color32::from_gray(220),
    );

    // Display current path in a pill box
    let path_display = if backup_dir.is_empty() {
        "Default: mindforge_backup/".to_string()
    } else if backup_dir.len() > 42 {
        format!("...{}", &backup_dir[backup_dir.len() - 39..])
    } else {
        backup_dir.clone()
    };

    let pill_rect = Rect::from_min_size(
        path_card.min + vec2(14.0, 42.0),
        vec2(card_w - 150.0, 32.0),
    );
    painter.rect(
        pill_rect,
        4.0,
        Color32::from_rgb(12, 13, 17),
        Stroke::new(1.0, Color32::from_rgb(28, 30, 40)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        pill_rect.min + vec2(10.0, 8.0),
        Align2::LEFT_TOP,
        path_display,
        FontId::monospace(11.0),
        theme.accent,
    );

    // Browse button
    let browse_rect = Rect::from_min_size(
        path_card.min + vec2(card_w - 126.0, 42.0),
        vec2(112.0, 32.0),
    );
    let browse_hover = ui.rect_contains_pointer(browse_rect);
    let browse_bg = if browse_hover {
        Color32::from_rgb(36, 40, 54)
    } else {
        Color32::from_rgb(26, 28, 38)
    };
    painter.rect(
        browse_rect,
        4.0,
        browse_bg,
        Stroke::new(1.0, if browse_hover { theme.accent } else { Color32::from_rgb(48, 52, 68) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        browse_rect.center(),
        Align2::CENTER_CENTER,
        "📁 Browse...",
        FontId::monospace(11.5),
        if browse_hover { Color32::WHITE } else { Color32::from_gray(210) },
    );

    if browse_hover && ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let p = folder.to_string_lossy().to_string();
            *backup_dir = p.clone();
            on_save_setting("backup_dir", &p);
        }
    }

    cur_y += 105.0;

    // Card 2: Instant Snapshot Action (:backup)
    let action_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 82.0));
    painter.rect(
        action_card,
        5.0,
        Color32::from_rgb(18, 20, 26),
        Stroke::new(1.0, Color32::from_rgb(34, 37, 48)),
        egui::StrokeKind::Inside,
    );

    painter.text(
        action_card.min + vec2(14.0, 12.0),
        Align2::LEFT_TOP,
        "1-Click Snapshot",
        FontId::monospace(12.5),
        Color32::from_gray(220),
    );

    let status_str = match last_backup_status {
        Some(s) if s.len() > 32 => format!("...{}", &s[s.len() - 29..]),
        Some(s) => s.to_string(),
        None => "Ready — or type :backup anytime".to_string(),
    };
    painter.text(
        action_card.min + vec2(14.0, 38.0),
        Align2::LEFT_TOP,
        status_str,
        FontId::monospace(10.5),
        theme.muted,
    );

    let run_btn_rect = Rect::from_min_size(
        action_card.min + vec2(card_w - 180.0, 24.0),
        vec2(166.0, 34.0),
    );
    let run_hover = ui.rect_contains_pointer(run_btn_rect);
    let run_bg = if run_hover {
        Color32::from_rgb(28, 48, 38)
    } else {
        Color32::from_rgb(20, 32, 26)
    };
    painter.rect(
        run_btn_rect,
        4.0,
        run_bg,
        Stroke::new(1.0, if run_hover { theme.accent } else { Color32::from_rgb(36, 68, 50) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        run_btn_rect.center(),
        Align2::CENTER_CENTER,
        "⚡ Snapshot (:backup)",
        FontId::monospace(11.5),
        theme.accent,
    );

    if run_hover && ui.input(|i| i.pointer.primary_clicked()) {
        action = Some(SettingPanelAction::TriggerBackup);
    }

    cur_y += 98.0;

    // Card 3: Safety info - clean multi-line layout with zero horizontal overflow
    let info_card = Rect::from_min_size(pos2(p_origin.x, cur_y), vec2(card_w, 132.0));
    painter.rect(
        info_card,
        5.0,
        Color32::from_rgb(14, 15, 20),
        Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
        egui::StrokeKind::Inside,
    );

    let features = [
        ("• WAL Architecture", "Guarantees high write speed and transactional safety."),
        ("• Automated Snapshots", "Timestamped as mindforge_backup_YYYYMMDD_HHMMSS.db"),
        ("• Zero Lock Contention", "Atomic snapshots execute without freezing the editor."),
    ];
    for (idx, (label, desc)) in features.iter().enumerate() {
        let y = info_card.min.y + 12.0 + idx as f32 * 38.0;
        painter.text(
            pos2(info_card.min.x + 14.0, y),
            Align2::LEFT_TOP,
            *label,
            FontId::monospace(11.0),
            theme.accent,
        );
        painter.text(
            pos2(info_card.min.x + 14.0, y + 16.0),
            Align2::LEFT_TOP,
            *desc,
            FontId::monospace(10.0),
            Color32::from_gray(160),
        );
    }

    action
}
