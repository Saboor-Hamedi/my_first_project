#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod bottom_bar;
mod caret;
mod commands;
mod db_worker;
mod editor;
mod fuzzy;
mod hybrid;
mod input;
mod modals;
mod mode;
mod notes;
mod settingpanel;
mod settingtabs;
mod sidebar;
mod sound;
mod theme;
mod types;
mod view_editor;
mod view_stats;
mod vim;
mod showcmd;
mod updater;

pub use types::{snapshot, visual_line};

use app::App;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "mono".into());
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "mono".into());
    ctx.set_fonts(fonts);
}
