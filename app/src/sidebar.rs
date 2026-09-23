//! Sleek floating sidebar with navigation and user notes library.

pub mod body;
pub mod footer;
pub mod header;

#[allow(unused_imports)]
pub use body::render_sidebar_body;
#[allow(unused_imports)]
pub use footer::render_sidebar_footer;
#[allow(unused_imports)]
pub use header::render_sidebar_header;

use core::Note;
use eframe::egui::{self, vec2, Color32, Rect, Stroke};

pub enum SidebarAction {
    SwitchMode(usize),
    LoadNote { id: i64, topic: String, body: String, index: usize },
    DeleteNote(i64),
    NewNote,
    OpenSettings,
    ToggleNotesLimit,
}

pub fn render_sidebar(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    sb_rect: Rect,
    active_mode_idx: usize,
    active_note_id: Option<i64>,
    notes: &[Note],
    notes_limit: usize,
    total_notes_count: usize,
    is_dirty: bool,
    accent: Color32,
    text_color: Color32,
    muted_color: Color32,
    sidebar_selected_idx: usize,
    sidebar_focused: bool,
) -> Option<SidebarAction> {
    let sidebar_w = sb_rect.width();

    // Floating panel background with subtle minimal border
    painter.rect(
        sb_rect,
        5.0,
        Color32::from_rgb(12, 12, 14),
        Stroke::new(1.0, Color32::from_rgb(32, 34, 40)),
        egui::StrokeKind::Inside,
    );

    let sb_origin = sb_rect.min + vec2(16.0, 18.0);

    // 1. Sidebar Header (Branding & Stats view)
    let header_action = header::render_sidebar_header(
        ui,
        painter,
        sb_origin,
        sidebar_w,
        active_mode_idx,
        accent,
        text_color,
    );

    // 2. Sidebar Body (File explorer / SQLite documents)
    let body_action = body::render_sidebar_body(
        ui,
        painter,
        sb_rect,
        sb_origin,
        active_note_id,
        notes,
        notes_limit,
        total_notes_count,
        is_dirty,
        accent,
        muted_color,
        sidebar_selected_idx,
        sidebar_focused,
    );

    // 3. Sidebar Footer (Round settings button with tooltip)
    let footer_action = footer::render_sidebar_footer(
        ui,
        painter,
        sb_rect,
        accent,
        muted_color,
    );

    header_action.or(body_action).or(footer_action)
}
