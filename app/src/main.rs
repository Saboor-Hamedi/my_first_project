#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod accent;
mod statusbar;
mod caret;
mod commands;
mod db_worker;
mod editor;
mod fuzzy;
mod hybrid;
mod input;
pub mod layout;
mod modals;
mod mode;
mod notes;
pub mod settings;
#[path = "sidebar/sidebar.rs"]
mod sidebar;
mod sound;
mod theme;
mod types;
mod view_editor;
mod view_stats;
mod vim;
mod showcmd;
mod updater;
mod docs;
mod help_panel;
mod scan_history_view;
mod scan_view;
pub mod terminal_pane;
pub mod ui_components;
pub mod zoom;
pub mod agent;

pub use types::{snapshot, visual_line};

use app::App;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/icon_64.rgba");
    let icon_data = egui::IconData {
        rgba: icon_bytes.to_vec(),
        width: 64,
        height: 64,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon_data)
            .with_app_id("app.mindforge.MindForge")
            .with_decorations(false) // Borderless: NO title bar, NO borders
            .with_transparent(true)  // Enables opacity and rounded edges
            .with_inner_size([1120.0, 740.0])
            .with_min_inner_size([700.0, 500.0])
            .with_max_inner_size([2560.0, 1440.0])
            .with_resizable(true),
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "mindforge",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "mono".into(),
        FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf")).into(),
    );

    #[cfg(target_os = "windows")]
    {
        if let Ok(emoji_bytes) = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf") {
            fonts.font_data.insert(
                "win_emoji".into(),
                FontData::from_owned(emoji_bytes).into(),
            );
        }
    }

    let mono = fonts.families.get_mut(&FontFamily::Monospace).unwrap();
    mono.insert(0, "mono".into());
    #[cfg(target_os = "windows")]
    if fonts.font_data.contains_key("win_emoji") {
        mono.push("win_emoji".into());
    }

    let prop = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
    prop.insert(0, "mono".into());
    #[cfg(target_os = "windows")]
    if fonts.font_data.contains_key("win_emoji") {
        prop.push("win_emoji".into());
    }

    ctx.set_fonts(fonts);
}
