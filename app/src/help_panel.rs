//! Interactive Help & Guidance Center Panel.
//! Provides categorized guidance for Vim motions, command-line operations,
//! keyboard shortcuts, and core app workflows.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

pub struct HelpPanelAction {
    pub open_docs: bool,
    pub open_settings: bool,
    pub should_close: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HelpCategory {
    QuickStart = 0,
    VimMotions = 1,
    Commands = 2,
    Shortcuts = 3,
}

impl HelpCategory {
    pub const ALL: &'static [HelpCategory] = &[
        HelpCategory::QuickStart,
        HelpCategory::VimMotions,
        HelpCategory::Commands,
        HelpCategory::Shortcuts,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::QuickStart => "⚡ Quick Start",
            Self::VimMotions => "⌨ Vim Motions",
            Self::Commands => "▶ Commands (:)",
            Self::Shortcuts => "✨ Shortcuts",
        }
    }
}

pub fn render_help_panel(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    bounds: Rect,
    active_tab: &mut usize,
    scroll_y: &mut f32,
    accent: Color32,
    _just_opened: bool,
) -> HelpPanelAction {
    let mut action = HelpPanelAction {
        open_docs: false,
        open_settings: false,
        should_close: false,
    };

    // Dimmed backdrop with deep focus overlay
    painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(195));

    // Centered modal dimensions
    let modal_w = 740.0f32.min(bounds.width() - 32.0);
    let modal_h = 530.0f32.min(bounds.height() - 40.0);
    let modal_rect = Rect::from_center_size(bounds.center(), vec2(modal_w, modal_h));

    // Click outside to dismiss
    if ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !modal_rect.contains(pos) {
                action.should_close = true;
            }
        }
    }

    // Modal frame: Deep frosted obsidian surface with subtle accent glow border
    painter.rect(
        modal_rect,
        8.0,
        Color32::from_rgb(13, 15, 20),
        Stroke::new(1.2, Color32::from_rgb(42, 46, 62)),
        egui::StrokeKind::Inside,
    );

    // ── Header Bar ──────────────────────────────────────────────────────────
    let header_h = 58.0;
    let header_rect = Rect::from_min_size(modal_rect.min, vec2(modal_w, header_h));

    // Accent mini badge
    let badge_rect = Rect::from_min_size(
        pos2(header_rect.min.x + 20.0, header_rect.min.y + 14.0),
        vec2(132.0, 16.0),
    );
    painter.rect_filled(
        badge_rect,
        3.0,
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 28),
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "✦ HELP & GUIDANCE",
        FontId::monospace(9.5),
        accent,
    );

    // Header title & subtitle
    painter.text(
        pos2(header_rect.min.x + 160.0, header_rect.min.y + 13.0),
        Align2::LEFT_TOP,
        "MINDFORGE HELP CENTER",
        FontId::monospace(14.0),
        Color32::WHITE,
    );
    painter.text(
        pos2(header_rect.min.x + 20.0, header_rect.min.y + 36.0),
        Align2::LEFT_TOP,
        "Home-row Vim navigation • Essential commands • Workflows & shortcuts",
        FontId::monospace(10.5),
        Color32::from_gray(140),
    );

    // Top-right close button [✕ Esc]
    let close_rect = Rect::from_min_size(
        pos2(header_rect.max.x - 72.0, header_rect.min.y + 16.0),
        vec2(52.0, 24.0),
    );
    let close_hover = ui.rect_contains_pointer(close_rect);
    painter.rect(
        close_rect,
        4.0,
        if close_hover { Color32::from_rgb(52, 24, 28) } else { Color32::from_rgb(22, 24, 32) },
        Stroke::new(1.0, if close_hover { Color32::from_rgb(230, 80, 90) } else { Color32::from_rgb(44, 48, 62) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        close_rect.center(),
        Align2::CENTER_CENTER,
        "✕ Esc",
        FontId::monospace(10.5),
        if close_hover { Color32::WHITE } else { Color32::from_gray(160) },
    );
    if close_hover && ui.input(|i| i.pointer.primary_clicked()) {
        action.should_close = true;
    }

    // Divider under header
    painter.line_segment(
        [pos2(modal_rect.min.x, modal_rect.min.y + header_h), pos2(modal_rect.max.x, modal_rect.min.y + header_h)],
        Stroke::new(1.0, Color32::from_rgb(28, 30, 40)),
    );

    // ── Category Navigation Tabs ────────────────────────────────────────────
    let nav_y = modal_rect.min.y + header_h + 8.0;
    let tab_h = 28.0;
    let tab_gap = 8.0;
    let mut tab_x = modal_rect.min.x + 20.0;

    for (idx, cat) in HelpCategory::ALL.iter().enumerate() {
        let text_w = cat.title().len() as f32 * 7.5 + 20.0;
        let tab_rect = Rect::from_min_size(pos2(tab_x, nav_y), vec2(text_w, tab_h));
        let is_selected = *active_tab == idx;
        let tab_hover = ui.rect_contains_pointer(tab_rect);

        // Sleek active underline indicator (no chunky background box or borders)
        if is_selected {
            painter.line_segment(
                [
                    pos2(tab_rect.min.x + 4.0, tab_rect.max.y),
                    pos2(tab_rect.max.x - 4.0, tab_rect.max.y),
                ],
                Stroke::new(2.0, accent),
            );
        }

        painter.text(
            tab_rect.center(),
            Align2::CENTER_CENTER,
            cat.title(),
            FontId::monospace(11.5),
            if is_selected {
                accent
            } else if tab_hover {
                Color32::WHITE
            } else {
                Color32::from_gray(155)
            },
        );

        if tab_hover && ui.input(|i| i.pointer.primary_clicked()) {
            *active_tab = idx;
            *scroll_y = 0.0;
        }

        tab_x += text_w + tab_gap;
    }

    // ── Action Buttons on Far Right of Tab Row ──────────────────────────────
    let doc_btn_rect = Rect::from_min_size(pos2(modal_rect.max.x - 146.0, nav_y), vec2(126.0, tab_h));
    let doc_hover = ui.rect_contains_pointer(doc_btn_rect);
    painter.rect(
        doc_btn_rect,
        5.0,
        if doc_hover { Color32::from_rgb(26, 34, 48) } else { Color32::from_rgb(18, 22, 32) },
        Stroke::new(1.0, if doc_hover { Color32::from_rgb(100, 200, 255) } else { Color32::from_rgb(40, 52, 70) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        doc_btn_rect.center(),
        Align2::CENTER_CENTER,
        "📖 Open Docs (:doc)",
        FontId::monospace(10.5),
        if doc_hover { Color32::WHITE } else { Color32::from_rgb(120, 200, 255) },
    );
    if doc_hover && ui.input(|i| i.pointer.primary_clicked()) {
        action.open_docs = true;
    }

    // ── Content Area ────────────────────────────────────────────────────────
    let content_top = nav_y + tab_h + 12.0;
    let content_h = modal_rect.max.y - content_top - 46.0;
    let content_rect = Rect::from_min_size(pos2(modal_rect.min.x + 20.0, content_top), vec2(modal_w - 40.0, content_h));

    // Handle scroll wheel
    if ui.rect_contains_pointer(modal_rect) {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            *scroll_y = (*scroll_y - scroll_delta * 0.8).clamp(0.0, 600.0);
        }
    }

    // Clip content inside content_rect
    let child_painter = painter.with_clip_rect(content_rect);

    let start_y = content_top - *scroll_y;
    match *active_tab {
        0 => render_quickstart_content(ui, &child_painter, content_rect, start_y, accent),
        1 => render_vim_content(ui, &child_painter, content_rect, start_y, accent),
        2 => render_commands_content(ui, &child_painter, content_rect, start_y, accent),
        _ => render_shortcuts_content(ui, &child_painter, content_rect, start_y, accent),
    }

    // ── Footer Bar ──────────────────────────────────────────────────────────
    let footer_y = modal_rect.max.y - 42.0;
    painter.line_segment(
        [pos2(modal_rect.min.x, footer_y), pos2(modal_rect.max.x, footer_y)],
        Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
    );

    // Keyboard guidance pills
    painter.text(
        pos2(modal_rect.min.x + 20.0, footer_y + 13.0),
        Align2::LEFT_TOP,
        "Tab / h,l: Switch Tab  •  j/k: Scroll  •  1-4: Jump  •  Esc: Close",
        FontId::monospace(10.5),
        Color32::from_gray(140),
    );

    let pref_btn = Rect::from_min_size(pos2(modal_rect.max.x - 146.0, footer_y + 8.0), vec2(126.0, 26.0));
    let pref_hover = ui.rect_contains_pointer(pref_btn);
    painter.rect(
        pref_btn,
        4.0,
        if pref_hover { Color32::from_rgb(30, 32, 42) } else { Color32::from_rgb(20, 22, 28) },
        Stroke::new(1.0, if pref_hover { accent } else { Color32::from_rgb(38, 42, 54) }),
        egui::StrokeKind::Inside,
    );
    painter.text(
        pref_btn.center(),
        Align2::CENTER_CENTER,
        "⚙ Settings (Ctrl+,)",
        FontId::monospace(10.5),
        if pref_hover { Color32::WHITE } else { Color32::from_gray(170) },
    );
    if pref_hover && ui.input(|i| i.pointer.primary_clicked()) {
        action.open_settings = true;
    }

    action
}

// ── Tab 0: Quick Start & Core Concepts ─────────────────────────────────────────

fn render_quickstart_content(ui: &egui::Ui, p: &egui::Painter, rect: Rect, mut y: f32, accent: Color32) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "1. WRITING & MANAGING NOTES",
            &[
                ("Auto-Save", "Notes are continuously saved to a high-speed SQLite database without manual effort."),
                ("Ctrl + N", "Instantly creates a new blank document ready for typing."),
                ("Ctrl + S", "Executes an immediate manual database sync and triggers dirty-state reset."),
                ("Ctrl + R", "Opens the modal to rename the current active document title."),
                ("Ctrl + B", "Toggles the left-hand navigation sidebar showing your notes list and writing stories."),
            ],
        ),
        (
            "2. HYBRID VS VIM MODES",
            &[
                ("Hybrid Mode", "Modern editor experience with intuitive hotkeys, smooth cursor glide, and auto-pairing."),
                ("Vim Mode", "Pure home-row modal editing with Normal, Insert, Visual, and Command lines."),
                ("Switching", "Type :vim or :mode vim/hybrid in the command bar to toggle anytime."),
                ("Uniform Caret", "Your custom animated caret stays uniform and visible across all Vim submodes."),
            ],
        ),
        (
            "3. LIVING ANIMATED CARETS & SOUNDS",
            &[
                ("Living Carets", "Choose from 9 animated carets: Candle, Fire, Water, Snow, Neon, Rainbow, Block, Beam, Underline."),
                ("Ambient Life", "Water drips into baseline ripples, candle flickers gently, fire micro-embers dance."),
                ("Typing Sounds", "Authentic synthesized mechanical switch audio: Thocky, Clacky, Creamy, Marbly, Poppy."),
            ],
        ),
    ];

    render_card_sections(ui, p, rect, &mut y, sections, accent);
}

// ── Tab 1: Vim Motions & Text Objects ──────────────────────────────────────────

fn render_vim_content(ui: &egui::Ui, p: &egui::Painter, rect: Rect, mut y: f32, accent: Color32) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "HOME-ROW MOTIONS & OPERATORS",
            &[
                ("h / j / k / l", "Precision cursor movement: Left, Down, Up, and Right."),
                ("w / b / e", "Jump forward to next word (w), backward to word start (b), or end of word (e)."),
                ("0 / $", "Jump instantly to line beginning (0) or visual line end ($)."),
                ("gg / G", "Jump to the very beginning (gg) or very end (G) of the active document."),
                ("40j / 10k", "Motion multipliers: jump 40 lines down or 10 lines up instantly."),
                ("dd / dw", "Delete entire current line (dd) or delete from cursor to next word (dw)."),
            ],
        ),
        (
            "TEXT OBJECTS & EDITING CHORDS",
            &[
                ("ci\" / ca\"", "Change inside quotes (ci\") or around quotes (ca\"). Deletes text and enters Insert."),
                ("da( / di(", "Delete around parentheses (da() or delete content inside parens (di()."),
                ("vi[ / va[", "Visually select inside brackets (vi[) or select enclosing brackets as well (va[)."),
                ("c / d / y", "Operator pending chords: change (c), delete (d), or yank/copy (y)."),
            ],
        ),
        (
            "SHOWCMD HUD & SEARCH",
            &[
                ("ShowCmd HUD", "Live floating pill in bottom-right tracks pending keys (VIM), visual range (VIS), search (FIND), and commands (CMD)."),
                (":set showcmd", "Enables the floating keystroke & operator HUD card."),
                (":set noshowcmd", "Disables the floating HUD (aliases: :set nonshowcmd, :noshowcmd)."),
                ("/pattern / ?pattern", "Buffer search forward (/) or backward (?). Press n for next match, N for previous."),
            ],
        ),
    ];

    render_card_sections(ui, p, rect, &mut y, sections, accent);
}

// ── Tab 2: Command Palette Reference ──────────────────────────────────────────

fn render_commands_content(ui: &egui::Ui, p: &egui::Painter, rect: Rect, mut y: f32, accent: Color32) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "ESSENTIAL APP COMMANDS",
            &[
                (":w / :save", "Save active note immediately to SQLite storage."),
                (":r <title>", "Rename the active document to a new title directly from the command bar."),
                (":d / :delete", "Open delete confirmation modal to remove active note."),
                (":doc / :docs", "Open built-in documentation and interactive guide reader."),
                (":editor", "Return from documentation viewer or stats back to active notes editor."),
                (":stats", "Open daily writing story, activity heatmap, and lifetime statistics."),
                (":clear", "Clear all text in active document editor."),
                (":q / :quit", "Exit the MindForge application."),
            ],
        ),
        (
            "CONFIGURATION & HUD CONTROLS",
            &[
                (":set showcmd", "Turn on the floating keystroke and command HUD capsule."),
                (":set noshowcmd", "Turn off the floating HUD (also :set nonshowcmd, :noshowcmd)."),
                (":vim on / off", "Toggle between Vim modal engine and modern Hybrid IDE input."),
                (":theme <name>", "Switch theme palette: green, amber, blue, monokai, rose, purple."),
                (":sound <type>", "Change switch sounds: thocky, clacky, creamy, marbly, poppy, clicky, off."),
                (":caret <kind>", "Set caret style: candle, fire, water, snow, neon, rainbow, beam, block."),
                (":backup", "Trigger instant atomic backup snapshot of the SQLite database."),
                (":export / :import", "Export note to markdown file, or import external .md/.txt into notes."),
            ],
        ),
    ];

    render_card_sections(ui, p, rect, &mut y, sections, accent);
}

// ── Tab 3: Complete Keyboard Shortcuts ────────────────────────────────────────

fn render_shortcuts_content(ui: &egui::Ui, p: &egui::Painter, rect: Rect, mut y: f32, accent: Color32) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "GLOBAL SHORTCUTS",
            &[
                ("Ctrl + N", "Create new blank note."),
                ("Ctrl + S", "Save active note immediately."),
                ("Ctrl + P", "Open fuzzy search palette across all notes."),
                ("Ctrl + B", "Toggle notes sidebar."),
                ("Ctrl + ,", "Open Preferences & Settings modal."),
                ("Ctrl + R", "Rename active note title."),
                ("Ctrl + Shift + D", "Delete active note confirmation modal."),
                (":help / F1", "Open this Guidance & Help Center."),
                ("Esc", "Close active modal or return to Normal mode."),
            ],
        ),
        (
            "EDITING & LINE MANIPULATION",
            &[
                ("Ctrl + ]", "Shift line or selected block 4 spaces right (indent)."),
                ("Ctrl + [", "Shift line or selected block 4 spaces left (dedent)."),
                ("Ctrl + D", "Duplicate current line or active selection directly below."),
                ("Ctrl + Z", "Undo last edit action."),
                ("Ctrl + Y", "Redo last undone edit action."),
                ("Ctrl + C / V / X", "Standard clipboard Copy, Paste, and Cut."),
                ("Tab / Shift + Tab", "Indent or dedent active line."),
            ],
        ),
    ];

    render_card_sections(ui, p, rect, &mut y, sections, accent);
}

// ── Helper: Section and Row Renderer ──────────────────────────────────────────

fn render_card_sections(
    ui: &egui::Ui,
    p: &egui::Painter,
    rect: Rect,
    y: &mut f32,
    sections: &[(&str, &[(&str, &str)])],
    accent: Color32,
) {
    let row_w = rect.width();
    let row_h = 24.0;
    let key_col_w = 160.0;
    let gap = 2.0;

    for (header, items) in sections.iter() {
        // Section header
        p.text(
            pos2(rect.min.x, *y + 2.0),
            Align2::LEFT_TOP,
            *header,
            FontId::monospace(11.0),
            accent,
        );
        let line_x = rect.min.x + header.len() as f32 * 7.0 + 12.0;
        p.line_segment(
            [pos2(line_x, *y + 8.0), pos2(rect.max.x, *y + 8.0)],
            Stroke::new(1.0, Color32::from_rgb(32, 36, 46)),
        );
        *y += 20.0;

        for (key, desc) in items.iter() {
            let row_rect = Rect::from_min_size(pos2(rect.min.x, *y), vec2(row_w, row_h));
            let is_row_hover = ui.rect_contains_pointer(row_rect);

            // Subtle hover tint only (no permanent chunky row background)
            if is_row_hover {
                p.rect_filled(row_rect, 3.0, Color32::from_rgb(22, 25, 34));
            }

            // Keycap text: crisp, clean monospace (no bulky boxed badges)
            p.text(
                pos2(row_rect.min.x + 8.0, row_rect.center().y),
                Align2::LEFT_CENTER,
                *key,
                FontId::monospace(11.5),
                if is_row_hover { accent } else { Color32::from_rgb(225, 230, 242) },
            );

            // Subtle vertical separator
            p.line_segment(
                [
                    pos2(row_rect.min.x + key_col_w, row_rect.min.y + 4.0),
                    pos2(row_rect.min.x + key_col_w, row_rect.max.y - 4.0),
                ],
                Stroke::new(1.0, Color32::from_rgb(34, 38, 50)),
            );

            // Description text
            p.text(
                pos2(row_rect.min.x + key_col_w + 14.0, row_rect.center().y),
                Align2::LEFT_CENTER,
                *desc,
                FontId::monospace(11.0),
                if is_row_hover { Color32::WHITE } else { Color32::from_gray(175) },
            );

            *y += row_h + gap;
        }

        *y += 10.0;
    }
}
