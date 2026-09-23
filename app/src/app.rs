//! Main application controller, state management, and egui frame loop.

use crate::statusbar::render_bottom_dock;
use crate::caret::{Caret, CaretKind};
use crate::db_worker::{spawn_db_worker, DbMsg};
use crate::editor::{Editor, VisualLine};
use crate::fuzzy::SearchItem;
use crate::input::{handle_input, window_shortcuts};
use crate::modals::{render_delete_confirm_modal, render_rename_modal, render_search_modal};
use crate::help_panel::render_help_panel;
use crate::hybrid::HybridEngine;
use crate::mode::Mode;
use crate::notes::{delete_active_note, quick_save_active_note, rename_active_note, update_search_results};
use crate::settings::{render_setting_panel, render_setting_tabs, SettingPanelAction, SettingTab};
use crate::sidebar::{render_sidebar, SidebarAction};
use crate::sound::{SoundEngine, SoundProfile};
use crate::theme::{Theme, ThemeKind};
use crate::view_editor::{render_editor_body, render_markdown_preview};
use crate::view_stats::render_stats;
use crate::vim::VimEngine;
use crate::updater::UpdateManager;

use chrono::Local;
use core::{DailyActivity, Database, Note};
use eframe::egui::{self, pos2, vec2, Color32, FontId, Rect, Stroke};
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorInputMode {
    Hybrid,
    Vim,
}

pub fn default_backup_dir() -> std::path::PathBuf {
    if let Some(proj) = directories::ProjectDirs::from("com", "mindforge", "mindforge") {
        proj.data_local_dir().join("mindforge_backup")
    } else {
        std::path::PathBuf::from("mindforge_backup")
    }
}

#[derive(Clone)]
pub struct OpenNote {
    pub id: i64,
    pub title: String,
    pub editor: Editor,
    pub scroll_y: f32,
    pub is_dirty: bool,
}

pub struct App {
    pub ed: Editor,
    pub doc_ed: Editor,
    pub cmd_ed: Editor,
    pub in_command: bool,
    pub caret: Caret,
    pub theme: Theme,
    pub sound: SoundEngine,
    pub font_size: f32,
    pub opacity: f32,
    pub last_char_time: f64,
    pub cell: Option<(f32, f32)>,
    pub visual_lines: Vec<VisualLine>,
    pub mode: Mode,
    pub status_msg: String,
    pub status_time: f64,

    pub first_frame: bool,

    // Sleek floating sidebar (Ctrl+B)
    pub sidebar_open: bool,
    pub sidebar_focused: bool,
    pub sidebar_selected_idx: usize,
    pub sidebar_width: f32,
    pub is_dragging_sidebar_splitter: bool,

    // Two-column Preferences modal (Ctrl+,)
    pub settings_open: bool,
    pub settings_just_opened: bool,
    pub active_setting_tab: SettingTab,
    pub backup_dir: String,
    pub last_backup_status: Option<String>,

    // Fuzzy search modal (Ctrl+P)
    pub search_open: bool,
    pub search_query: String,
    pub search_results: Vec<SearchItem>,
    pub search_selected: usize,
    pub search_just_opened: bool,

    // Rename modal (Ctrl+R)
    pub rename_open: bool,
    pub rename_input: String,
    pub rename_just_opened: bool,

    // Delete confirmation modal (Ctrl+Shift+D or :d)
    pub delete_confirm_open: bool,
    pub delete_just_opened: bool,
    pub pending_delete_note_id: Option<i64>,

    // Guidance & Help modal (:help or F1)
    pub help_open: bool,
    pub help_just_opened: bool,
    pub help_tab: usize,
    pub help_scroll_y: f32,

    // Active document & sidebar limits
    pub active_note_id: Option<i64>,
    pub active_note_title: String,
    pub notes_list: Vec<Note>,
    pub sidebar_notes_limit: usize,
    pub total_notes_count: usize,

    // Multi-note & Documentation Tabs
    pub open_notes: Vec<OpenNote>,
    pub active_tab: usize,
    pub last_active_tab: usize,
    pub open_doc_tabs: Vec<usize>,
    pub active_doc_tab: usize,
    pub last_active_doc_tab: usize,
    pub tab_scroll_offset: f32,
    pub doc_tab_scroll_offset: f32,

    // Scrolling & auto-save
    pub scroll_y: f32,
    pub doc_scroll_y: f32,
    pub preview_scroll_y: f32,
    pub preview_open: bool,
    pub split_ratio: f32,
    pub is_dragging_splitter: bool,
    pub show_line_numbers: bool,
    pub is_dirty: bool,
    pub last_saved_time: f64,

    // Database & worker channel
    pub db: Option<Database>,
    pub db_tx: Sender<DbMsg>,

    // Active editing mode (Hybrid vs Vim)
    pub editor_input_mode: EditorInputMode,
    pub hybrid: HybridEngine,
    pub vim: VimEngine,
    pub showcmd: crate::showcmd::ShowCmdState,

    // Daily Activity & Writing Story tracking
    pub pending_secs: f32,
    pub pending_keys: u32,
    pub pending_words: u32,
    pub pending_created: u32,
    pub pending_edited: u32,
    pub last_flush_time: f64,
    pub today_activity: DailyActivity,
    pub activity_history: Vec<DailyActivity>,
    pub lifetime_activity: (u64, u64, u64, usize),

    // In-app auto-updater
    pub updater: UpdateManager,
    pub active_doc_idx: usize,
    pub doc_sidebar_focused: bool,
    pub doc_selected_idx: usize,

    // Webscan (:scan & :scans) state
    pub prev_mode_before_scan: Mode,
    pub scan_in_progress: Option<String>,
    pub scan_rx: Option<std::sync::mpsc::Receiver<Result<webscan::ScanResult, String>>>,
    pub active_scan_result: Option<webscan::ScanResult>,
    pub active_scan_error: Option<(String, String)>,
    pub scan_report_scroll_y: f32,
    pub past_scans: Vec<core::ScanRecord>,
    pub scan_history_selected: usize,
    pub scan_history_scroll_y: f32,
    pub clipboard_text: Option<String>,
    pub accent_overrides: crate::accent::AccentOverrides,
    pub accent_dropdown_open: bool,
}

impl App {
    pub fn new() -> Self {
        // Open SQLite connection on the main thread first to safely apply any pending migrations
        let db = Database::open_default().ok();
        let tx = spawn_db_worker();

        let mut app = Self {
            ed: Editor::new(),
            doc_ed: Editor::new(),
            cmd_ed: Editor::new(),
            in_command: false,
            caret: Caret::new(CaretKind::Beam),
            theme: Theme::from_kind(ThemeKind::Green),
            sound: SoundEngine::new(SoundProfile::Thocky), // Mechanical keyboard enabled by default
            font_size: 16.0,
            opacity: 1.0,
            last_char_time: -10.0,
            cell: None,
            visual_lines: vec![VisualLine { char_start: 0, char_end: 0 }],
            mode: Mode::Normal,
            status_msg: String::new(),
            status_time: 0.0,
            first_frame: true,
            sidebar_open: false,
            sidebar_focused: false,
            sidebar_selected_idx: 0,
            sidebar_width: crate::layout::DEFAULT_SIDEBAR_W,
            is_dragging_sidebar_splitter: false,
            settings_open: false,
            settings_just_opened: false,
            active_setting_tab: SettingTab::Carets,
            backup_dir: default_backup_dir().to_string_lossy().to_string(),
            last_backup_status: None,
            search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_just_opened: false,
            rename_open: false,
            rename_input: String::new(),
            rename_just_opened: false,
            delete_confirm_open: false,
            delete_just_opened: false,
            pending_delete_note_id: None,
            help_open: false,
            help_just_opened: false,
            help_tab: 0,
            help_scroll_y: 0.0,
            active_note_id: None,
            active_note_title: "Untitled Note".to_string(),
            notes_list: Vec::new(),
            sidebar_notes_limit: 50,
            total_notes_count: 0,
            open_notes: Vec::new(),
            active_tab: 0,
            last_active_tab: 0,
            open_doc_tabs: vec![0],
            active_doc_tab: 0,
            last_active_doc_tab: 0,
            tab_scroll_offset: 0.0,
            doc_tab_scroll_offset: 0.0,
            scroll_y: 0.0,
            doc_scroll_y: 0.0,
            preview_scroll_y: 0.0,
            preview_open: false,
            split_ratio: 0.5,
            is_dragging_splitter: false,
            show_line_numbers: true,
            is_dirty: false,
            last_saved_time: 0.0,
            db,
            db_tx: tx,
            editor_input_mode: EditorInputMode::Hybrid,
            hybrid: HybridEngine::new(),
            vim: VimEngine::new(),
            showcmd: crate::showcmd::ShowCmdState::new(true),
            pending_secs: 0.0,
            pending_keys: 0,
            pending_words: 0,
            pending_created: 0,
            pending_edited: 0,
            last_flush_time: 0.0,
            today_activity: DailyActivity::default(),
            activity_history: Vec::new(),
            lifetime_activity: (0, 0, 0, 0),
            updater: UpdateManager::new(),
            active_doc_idx: 0,
            doc_sidebar_focused: true,
            doc_selected_idx: 0,
            prev_mode_before_scan: Mode::Normal,
            scan_in_progress: None,
            scan_rx: None,
            active_scan_result: None,
            active_scan_error: None,
            scan_report_scroll_y: 0.0,
            past_scans: Vec::new(),
            scan_history_selected: 0,
            scan_history_scroll_y: 0.0,
            clipboard_text: None,
            accent_overrides: crate::accent::AccentOverrides::default(),
            accent_dropdown_open: false,
        };

        app.load_settings();
        app.reload_db_state();

        let mut restored = false;
        if let Some(ref db) = app.db {
            if let Ok(Some(saved_json)) = db.get_setting("open_note_tab_ids") {
                if let Ok(saved_ids) = serde_json::from_str::<Vec<i64>>(&saved_json) {
                    if !saved_ids.is_empty() {
                        for id in saved_ids {
                            let note_opt = app.notes_list.iter().find(|n| n.id == id).cloned()
                                .or_else(|| db.get_note(id).ok().flatten());
                            if let Some(note) = note_opt {
                                let clean = note.body.replace("\r\n", "\n").replace('\r', "\n");
                                let mut ed = Editor::new();
                                ed.set_text(&clean);
                                let saved_cur = db.get_setting(&format!("note_caret_{}", note.id))
                                    .ok().flatten()
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(0);
                                ed.cur = saved_cur.min(ed.buf.len());
                                let saved_scroll = db.get_setting(&format!("note_scroll_{}", note.id))
                                    .ok().flatten()
                                    .and_then(|s| s.parse::<f32>().ok())
                                    .unwrap_or(0.0);

                                app.open_notes.push(OpenNote {
                                    id: note.id,
                                    title: note.topic.clone(),
                                    editor: ed,
                                    scroll_y: saved_scroll,
                                    is_dirty: false,
                                });
                            }
                        }
                        if !app.open_notes.is_empty() {
                            restored = true;
                            let saved_active_idx = db.get_setting("open_note_active_tab")
                                .ok().flatten()
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0);

                            let tab_idx = if let Some(last_id) = app.active_note_id {
                                app.open_notes.iter().position(|n| n.id == last_id).unwrap_or(saved_active_idx)
                            } else {
                                saved_active_idx
                            };
                            let active_idx = tab_idx.min(app.open_notes.len() - 1);
                            app.active_tab = active_idx;
                            let cur_tab = &app.open_notes[active_idx];
                            app.active_note_id = Some(cur_tab.id);
                            app.active_note_title = cur_tab.title.clone();
                            app.ed = cur_tab.editor.clone();
                            app.scroll_y = cur_tab.scroll_y;
                        }
                    }
                }
            }
        }

        if !restored && app.open_notes.is_empty() {
            for (idx, note) in app.notes_list.iter().take(3).enumerate() {
                let clean = note.body.replace("\r\n", "\n").replace('\r', "\n");
                let mut ed = Editor::new();
                ed.set_text(&clean);
                if Some(note.id) == app.active_note_id {
                    ed = app.ed.clone();
                    app.active_tab = idx;
                }
                app.open_notes.push(OpenNote {
                    id: note.id,
                    title: note.topic.clone(),
                    editor: ed,
                    scroll_y: 0.0,
                    is_dirty: false,
                });
            }
            if app.open_notes.is_empty() {
                app.open_notes.push(OpenNote {
                    id: app.active_note_id.unwrap_or(0),
                    title: app.active_note_title.clone(),
                    editor: app.ed.clone(),
                    scroll_y: app.scroll_y,
                    is_dirty: app.is_dirty,
                });
                app.active_tab = 0;
            }
        }
        app
    }

    pub fn load_settings(&mut self) {
        if let Some(db) = &self.db {
            if let Ok(Some(c)) = db.get_setting("caret") {
                if let Some(kind) = CaretKind::parse(&c) {
                    self.caret.kind = kind;
                }
            }
            if let Ok(Some(t)) = db.get_setting("theme") {
                if let Some(kind) = ThemeKind::parse(&t) {
                    self.theme = Theme::from_kind(kind);
                }
            }
            self.accent_overrides = crate::accent::AccentOverrides::load_from_db(db);
            self.accent_overrides.apply(&mut self.theme);
            if let Ok(Some(s)) = db.get_setting("sound") {
                if let Some(profile) = SoundProfile::parse(&s) {
                    self.sound.profile = profile;
                }
            }
            if let Ok(Some(w)) = db.get_setting("caret_width") {
                if let Ok(val) = w.parse::<f32>() {
                    self.caret.width = val.clamp(1.0, 10.0);
                }
            }
            if let Ok(Some(anim)) = db.get_setting("caret_animations") {
                self.caret.animations_enabled = anim != "off" && anim != "false";
            }
            if let Ok(Some(op)) = db.get_setting("opacity") {
                if let Ok(val) = op.parse::<f32>() {
                    self.opacity = val.clamp(0.2, 1.0);
                }
            }
            if let Ok(Some(f)) = db.get_setting("font") {
                if let Ok(val) = f.parse::<f32>() {
                    self.font_size = val.clamp(12.0, 48.0);
                    self.cell = None;
                }
            }
            if let Ok(Some(sw)) = db.get_setting("sidebar_w") {
                if let Ok(val) = sw.parse::<f32>() {
                    self.sidebar_width = val.clamp(crate::layout::MIN_SIDEBAR_W, crate::layout::MAX_SIDEBAR_W);
                }
            }
            if let Ok(Some(b)) = db.get_setting("backup_dir") {
                self.backup_dir = b;
            }
            if let Ok(Some(m)) = db.get_setting("editor_mode") {
                if m == "vim" {
                    self.editor_input_mode = EditorInputMode::Vim;
                } else {
                    self.editor_input_mode = EditorInputMode::Hybrid;
                }
            }
            if let Ok(Some(s)) = db.get_setting("showcmd") {
                self.showcmd.enabled = s != "off" && s != "false";
            }
            if let Ok(Some(ln)) = db.get_setting("line_numbers") {
                self.show_line_numbers = ln != "off" && ln != "false";
            }
            if let Ok(Some(p)) = db.get_setting("preview") {
                self.preview_open = p == "on" || p == "true";
            }
            if let Ok(Some(sb)) = db.get_setting("sidebar") {
                self.sidebar_open = sb == "on" || sb == "true";
            }
        }
    }

    pub fn save_active_note_id(&self) {
        let val = self.active_note_id.map(|id| id.to_string()).unwrap_or_default();
        let _ = self.db_tx.send(DbMsg::SaveSetting {
            key: "last_active_note_id".to_string(),
            val,
        });
        self.save_caret_position();
    }

    pub fn save_caret_position(&self) {
        let _ = self.db_tx.send(DbMsg::SaveSetting {
            key: "last_caret_pos".to_string(),
            val: self.ed.cur.to_string(),
        });
        let _ = self.db_tx.send(DbMsg::SaveSetting {
            key: "last_scroll_y".to_string(),
            val: self.scroll_y.to_string(),
        });
        if let Some(id) = self.active_note_id {
            let _ = self.db_tx.send(DbMsg::SaveSetting {
                key: format!("note_caret_{}", id),
                val: self.ed.cur.to_string(),
            });
            let _ = self.db_tx.send(DbMsg::SaveSetting {
                key: format!("note_scroll_{}", id),
                val: self.scroll_y.to_string(),
            });
        }
    }

    pub fn save_open_tabs(&self) {
        if let Some(ref db) = self.db {
            let tab_ids: Vec<i64> = self.open_notes.iter()
                .filter_map(|n| if n.id > 0 { Some(n.id) } else { None })
                .collect();
            if let Ok(json) = serde_json::to_string(&tab_ids) {
                let _ = db.set_setting("open_note_tab_ids", &json);
            }
            let _ = db.set_setting("open_note_active_tab", &self.active_tab.to_string());
        }
    }

    pub fn sync_save_session(&mut self) {
        let mut created_id = None;
        if let Some(ref db) = self.db {
            let _ = db.set_setting("last_caret_pos", &self.ed.cur.to_string());
            let _ = db.set_setting("last_scroll_y", &self.scroll_y.to_string());
            if let Some(id) = self.active_note_id {
                let _ = db.set_setting("last_active_note_id", &id.to_string());
                let _ = db.set_setting(&format!("note_caret_{}", id), &self.ed.cur.to_string());
                let _ = db.set_setting(&format!("note_scroll_{}", id), &self.scroll_y.to_string());
                if self.is_dirty {
                    let _ = db.update_note(id, &self.ed.text());
                    self.is_dirty = false;
                }
            } else if self.is_dirty || !self.ed.text().trim().is_empty() {
                let topic = if self.active_note_title.trim().is_empty() {
                    "Untitled Note".to_string()
                } else {
                    self.active_note_title.clone()
                };
                let dt = Local::now().naive_local();
                let content = self.ed.text();
                if let Ok(new_id) = db.add_note(&topic, &content, None, dt) {
                    created_id = Some(new_id);
                    let _ = db.set_setting("last_active_note_id", &new_id.to_string());
                }
            }
        }
        if let Some(new_id) = created_id {
            self.active_note_id = Some(new_id);
            self.is_dirty = false;
            self.sync_active_tab();
        }
        self.save_open_tabs();
    }

    pub fn sync_active_tab(&mut self) {
        if let Some(tab) = self.open_notes.get_mut(self.active_tab) {
            tab.title = self.active_note_title.clone();
            tab.is_dirty = self.is_dirty;
            tab.scroll_y = self.scroll_y;
            if let Some(id) = self.active_note_id {
                tab.id = id;
            }
        }
    }

    pub fn switch_tab(&mut self, new_idx: usize, now: f64) {
        if self.open_notes.is_empty() {
            return;
        }
        let new_idx = new_idx.min(self.open_notes.len() - 1);
        if new_idx == self.active_tab {
            return;
        }

        // 1. Sync current state into the active tab before switching
        if let Some(cur) = self.open_notes.get_mut(self.active_tab) {
            cur.editor = self.ed.clone();
            cur.title = self.active_note_title.clone();
            cur.scroll_y = self.scroll_y;
            cur.is_dirty = self.is_dirty;
            if let Some(cur_id) = self.active_note_id {
                cur.id = cur_id;
                if let Some(ref db) = self.db {
                    let _ = db.set_setting(&format!("note_caret_{}", cur_id), &self.ed.cur.to_string());
                    let _ = db.set_setting(&format!("note_scroll_{}", cur_id), &self.scroll_y.to_string());
                }
            }
        }

        // 2. Set new active tab index
        self.active_tab = new_idx;

        // 3. Load target tab state
        let target = &self.open_notes[self.active_tab];
        self.active_note_id = if target.id > 0 { Some(target.id) } else { None };
        self.active_note_title = target.title.clone();
        self.ed = target.editor.clone();
        self.scroll_y = target.scroll_y;
        self.is_dirty = target.is_dirty;
        self.save_active_note_id();
        self.save_open_tabs();
        self.mode = Mode::Normal;
        self.vim.set_mode(crate::vim::VimSubMode::Normal, &mut self.ed);
        let msg = format!("Switched to {}", self.active_note_title);
        self.set_status(&msg, now);
    }

    pub fn close_tab(&mut self, idx: usize, now: f64) {
        if idx >= self.open_notes.len() {
            return;
        }
        if idx == self.active_tab && self.is_dirty {
            self.quick_save_active_note(now);
        }

        self.open_notes.remove(idx);

        if self.open_notes.is_empty() {
            self.active_note_id = None;
            self.active_note_title = "Untitled Note".to_string();
            self.ed.clear();
            self.is_dirty = false;
            self.scroll_y = 0.0;
            self.open_notes.push(OpenNote {
                id: 0,
                title: "Untitled Note".to_string(),
                editor: self.ed.clone(),
                scroll_y: 0.0,
                is_dirty: false,
            });
            self.active_tab = 0;
            self.save_active_note_id();
        } else {
            if self.active_tab >= self.open_notes.len() {
                self.active_tab = self.open_notes.len() - 1;
            } else if idx < self.active_tab {
                self.active_tab = self.active_tab.saturating_sub(1);
            }
            let target = &self.open_notes[self.active_tab];
            self.active_note_id = if target.id > 0 { Some(target.id) } else { None };
            self.active_note_title = target.title.clone();
            self.ed = target.editor.clone();
            self.scroll_y = target.scroll_y;
            self.is_dirty = target.is_dirty;
            self.save_active_note_id();
        }
        self.save_open_tabs();
    }

    pub fn switch_doc_tab(&mut self, new_idx: usize, now: f64) {
        if self.open_doc_tabs.is_empty() {
            return;
        }
        let new_idx = new_idx.min(self.open_doc_tabs.len() - 1);
        self.active_doc_tab = new_idx;
        let doc_idx = self.open_doc_tabs[self.active_doc_tab];
        self.load_doc_by_index(doc_idx, now);
    }

    pub fn close_doc_tab(&mut self, idx: usize, now: f64) {
        if self.open_doc_tabs.len() <= 1 || idx >= self.open_doc_tabs.len() {
            return;
        }
        self.open_doc_tabs.remove(idx);
        if self.active_doc_tab >= self.open_doc_tabs.len() {
            self.active_doc_tab = self.open_doc_tabs.len() - 1;
        } else if idx < self.active_doc_tab {
            self.active_doc_tab = self.active_doc_tab.saturating_sub(1);
        }
        let doc_idx = self.open_doc_tabs[self.active_doc_tab];
        self.load_doc_by_index(doc_idx, now);
    }

    /// Loads a note by id into the editor, switching tab if open or adding a new tab.
    pub fn load_note(&mut self, id: i64, topic: String, body: String, now: f64) {
        if let Some(idx) = self.open_notes.iter().position(|n| n.id == id) {
            self.switch_tab(idx, now);
            return;
        }

        // Save current active tab before switching
        if let Some(cur) = self.open_notes.get_mut(self.active_tab) {
            cur.editor = self.ed.clone();
            cur.title = self.active_note_title.clone();
            cur.scroll_y = self.scroll_y;
            cur.is_dirty = self.is_dirty;
            if let Some(cur_id) = self.active_note_id {
                cur.id = cur_id;
                if let Some(ref db) = self.db {
                    let _ = db.set_setting(&format!("note_caret_{}", cur_id), &self.ed.cur.to_string());
                    let _ = db.set_setting(&format!("note_scroll_{}", cur_id), &self.scroll_y.to_string());
                }
            }
        }

        let clean = body.replace("\r\n", "\n").replace('\r', "\n");
        self.active_note_id = Some(id);
        self.save_active_note_id();
        self.active_note_title = topic.clone();
        self.ed.set_text(&clean);
        let saved_cur = self.db.as_ref()
            .and_then(|db| db.get_setting(&format!("note_caret_{}", id)).ok().flatten())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        self.ed.cur = saved_cur.min(self.ed.buf.len());
        let saved_scroll = self.db.as_ref()
            .and_then(|db| db.get_setting(&format!("note_scroll_{}", id)).ok().flatten())
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        self.scroll_y = saved_scroll;
        self.mode = Mode::Normal;
        self.vim.set_mode(crate::vim::VimSubMode::Normal, &mut self.ed);
        self.is_dirty = false;

        // If existing single tab was an untouched empty Untitled Note, replace it
        if self.open_notes.len() == 1 && self.open_notes[0].id == 0 && !self.open_notes[0].is_dirty {
            self.open_notes[0] = OpenNote {
                id,
                title: topic,
                editor: self.ed.clone(),
                scroll_y: saved_scroll,
                is_dirty: false,
            };
            self.active_tab = 0;
        } else {
            self.open_notes.push(OpenNote {
                id,
                title: topic,
                editor: self.ed.clone(),
                scroll_y: saved_scroll,
                is_dirty: false,
            });
            self.active_tab = self.open_notes.len() - 1;
        }
        self.save_open_tabs();
        self.set_status("Opened note", now);
    }

    pub fn reload_db_state(&mut self) {
        if self.db.is_none() {
            self.db = Database::open_default().ok();
        }
        if let Some(db) = &self.db {
            if let Ok(count) = db.get_notes_count() {
                self.total_notes_count = count;
            }
            if let Ok(notes) = db.get_recent_notes(self.sidebar_notes_limit) {
                self.notes_list = notes;
                if self.active_note_id.is_none() {
                    let saved_note_id = db
                        .get_setting("last_active_note_id")
                        .ok()
                        .flatten()
                        .and_then(|s| s.parse::<i64>().ok());

                    let target_note = saved_note_id.and_then(|id| {
                        self.notes_list
                            .iter()
                            .find(|n| n.id == id)
                            .cloned()
                            .or_else(|| db.get_note(id).ok().flatten())
                    });

                    if let Some(target) = target_note.or_else(|| self.notes_list.first().cloned()) {
                        let clean = target.body.replace("\r\n", "\n").replace('\r', "\n");
                        self.active_note_id = Some(target.id);
                        self.active_note_title = target.topic.clone();
                        self.ed.set_text(&clean);

                        let note_caret_key = format!("note_caret_{}", target.id);
                        let saved_cur = db
                            .get_setting(&note_caret_key)
                            .ok()
                            .flatten()
                            .or_else(|| db.get_setting("last_caret_pos").ok().flatten())
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        self.ed.cur = saved_cur.min(self.ed.buf.len());
                        self.is_dirty = false;

                        let saved_scroll = db
                            .get_setting(&format!("note_scroll_{}", target.id))
                            .ok()
                            .flatten()
                            .or_else(|| db.get_setting("last_scroll_y").ok().flatten())
                            .and_then(|s| s.parse::<f32>().ok())
                            .unwrap_or(0.0);
                        self.scroll_y = saved_scroll;
                    }
                }
            }
            if let Ok(recent) = db.get_recent_activity(14) {
                let today_str = Local::now().date_naive().format("%Y-%m-%d").to_string();
                if let Some(t) = recent.iter().find(|a| a.date == today_str) {
                    self.today_activity = t.clone();
                } else {
                    self.today_activity = DailyActivity {
                        date: today_str,
                        ..Default::default()
                    };
                }
                self.activity_history = recent;
            }
            if let Ok(lifetime) = db.get_lifetime_activity() {
                self.lifetime_activity = lifetime;
            }
        }
    }

    pub fn flush_activity(&mut self, now: f64) {
        let secs = self.pending_secs as u32;
        let keys = self.pending_keys;
        let words = self.pending_words;
        let created = self.pending_created;
        let edited = self.pending_edited;

        if secs > 0 || keys > 0 || words > 0 || created > 0 || edited > 0 {
            let today_str = Local::now().date_naive().format("%Y-%m-%d").to_string();
            let _ = self.db_tx.send(DbMsg::FlushActivity {
                date: today_str,
                delta_secs: secs,
                delta_keys: keys,
                delta_words: words,
                delta_created: created,
                delta_edited: edited,
            });
            self.today_activity.active_seconds += secs;
            self.today_activity.keystrokes += keys;
            self.today_activity.words_written += words;
            self.today_activity.notes_created += created;
            self.today_activity.notes_edited += edited;

            self.lifetime_activity.0 += secs as u64;
            self.lifetime_activity.1 += keys as u64;
            self.lifetime_activity.2 += words as u64;

            self.pending_secs = 0.0;
            self.pending_keys = 0;
            self.pending_words = 0;
            self.pending_created = 0;
            self.pending_edited = 0;
            self.last_flush_time = now;
        }
    }

    pub fn quick_save_active_note(&mut self, now: f64) {
        quick_save_active_note(self, now);
    }

    pub fn delete_active_note(&mut self, now: f64) {
        delete_active_note(self, now);
    }

    pub fn rename_active_note(&mut self, new_title: &str, now: f64) {
        rename_active_note(self, new_title, now);
    }

    pub fn update_search_results(&mut self) {
        update_search_results(self);
    }

    pub fn open_docs_mode(&mut self, now: f64) {
        self.mode = Mode::Doc;
        self.sidebar_open = true;
        self.doc_sidebar_focused = true;
        self.doc_selected_idx = self.active_doc_idx;
        let docs = crate::docs::get_docs();
        let idx = self.active_doc_idx.min(docs.len().saturating_sub(1));
        if let Some(doc) = docs.get(idx) {
            let clean = crate::docs::format_doc_for_reader(doc.content);
            self.doc_ed.set_text(&clean);
            self.doc_ed.cur = 0;
            self.doc_scroll_y = 0.0;
            self.vim.set_mode(crate::vim::VimSubMode::Normal, &mut self.doc_ed);
            self.set_status(format!("Opened Guide: {}", doc.title), now);
        }
    }

    pub fn load_doc_by_index(&mut self, idx: usize, now: f64) {
        self.active_doc_idx = idx;
        self.doc_selected_idx = idx;
        if !self.open_doc_tabs.contains(&idx) {
            self.open_doc_tabs.push(idx);
            self.active_doc_tab = self.open_doc_tabs.len() - 1;
        } else {
            self.active_doc_tab = self.open_doc_tabs.iter().position(|&d| d == idx).unwrap_or(0);
        }
        self.open_docs_mode(now);
    }

    pub fn set_status(&mut self, msg: impl Into<String>, now: f64) {
        self.status_msg = msg.into();
        self.status_time = now;
    }

    pub fn trigger_backup(&mut self, now: f64) {
        let target = if !self.backup_dir.is_empty() {
            let p = std::path::PathBuf::from(&self.backup_dir);
            if p.file_name().and_then(|s| s.to_str()) == Some("mindforge_backup") {
                p
            } else {
                p.join("mindforge_backup")
            }
        } else {
            default_backup_dir()
        };
        if let Some(ref db) = self.db {
            match db.backup(&target) {
                Ok(p) => {
                    self.last_backup_status = Some(format!(
                        "Success ({})",
                        chrono::Local::now().format("%H:%M:%S")
                    ));
                    self.set_status(format!("Backup saved: {}", p.display()), now);
                }
                Err(e) => {
                    self.last_backup_status = Some(format!("Failed: {}", e));
                    self.set_status(format!("Backup failed: {}", e), now);
                }
            }
        } else {
            self.set_status("Database not available for backup", now);
        }
    }

    /// Computes the exact monospace character advance width and line height.
    /// Measuring 100 characters eliminates single-glyph bounding box ink discrepancies,
    /// guaranteeing that caret placement at `col * cw` aligns with rendered text at any line length.
    pub fn cell_size(&mut self, ctx: &egui::Context) -> (f32, f32) {
        *self.cell.get_or_insert_with(|| {
            let font = FontId::monospace(self.font_size);
            let sample_100 = "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM";
            let g100 = ctx.fonts(|f| f.layout_no_wrap(sample_100.to_owned(), font.clone(), Color32::WHITE));
            let g1 = ctx.fonts(|f| f.layout_no_wrap("M".to_owned(), font, Color32::WHITE));
            let cw = (g100.size().x - g1.size().x) / 99.0;
            let lh = (g1.size().y * 1.30).round();
            (cw, lh)
        })
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, dt: f32, now: f64, typed: bool) {
        let (cw, lh) = self.cell_size(ui.ctx());
        let painter = ui.painter().clone();

        let bounds = ui.max_rect();

        // 5px Rounded window background frame
        let win_bg = Color32::from_rgba_unmultiplied(
            self.theme.bg.r(),
            self.theme.bg.g(),
            self.theme.bg.b(),
            (self.opacity * 255.0) as u8,
        );
        painter.rect(
            bounds,
            5.0,
            win_bg,
            eframe::egui::Stroke::new(1.0, Color32::from_rgb(32, 34, 40)),
            egui::StrokeKind::Inside,
        );

        let layout = crate::layout::compute_app_layout(bounds, self.sidebar_open, self.sidebar_width);
        let titlebar_rect = layout.titlebar_rect;
        let cmd_bar_rect = layout.cmd_bar_rect;
        let editor_panel_rect = layout.editor_panel_rect;

        // Full-width modern Titlebar spanning entire top
        let (header_title, header_dirty) = match self.mode {
            Mode::Doc => {
                let doc_title = crate::docs::get_docs()
                    .get(self.active_doc_idx)
                    .map(|d| d.title)
                    .unwrap_or("Documentation");
                (format!("📖 {}", doc_title), false)
            }
            Mode::Normal => (self.active_note_title.clone(), self.is_dirty),
            Mode::Stats => ("📊 Daily Story & Statistics".to_string(), false),
            Mode::ScanReport | Mode::ScanHistory => ("🌐 Security Scanner".to_string(), false),
        };

        let (titlebar_action, accent_anchor_rect) = crate::view_editor::render_full_titlebar(
            ui,
            &painter,
            titlebar_rect,
            &header_title,
            header_dirty,
            &self.theme,
            self.accent_dropdown_open,
        );

        if let Some(crate::view_editor::TitlebarAction::ToggleAccentDropdown) = titlebar_action {
            self.accent_dropdown_open = !self.accent_dropdown_open;
        }

        // Sidebar Splitter Divider & Knob (when sidebar is open in Notes or Docs)
        if let (Some(hit_rect), Some(center_x)) = (layout.splitter_hit_rect, layout.splitter_center_x) {
            let is_splitter_hovered = ui.rect_contains_pointer(hit_rect);
            let primary_down = ui.input(|i| i.pointer.primary_down());
            let primary_pressed = ui.input(|i| i.pointer.primary_clicked() || i.pointer.button_pressed(egui::PointerButton::Primary));

            if is_splitter_hovered && primary_pressed {
                self.is_dragging_sidebar_splitter = true;
            }

            if self.is_dragging_sidebar_splitter {
                if primary_down {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos())) {
                        let drag_x = pos.x - bounds.min.x - crate::layout::GAP;
                        if drag_x < 70.0 {
                            self.sidebar_open = false;
                            self.is_dragging_sidebar_splitter = false;
                        } else {
                            let max_sb = (bounds.width() - 200.0).clamp(crate::layout::MIN_SIDEBAR_W, crate::layout::MAX_SIDEBAR_W);
                            let new_w = drag_x.clamp(crate::layout::MIN_SIDEBAR_W, max_sb);
                            self.sidebar_width = new_w;
                        }
                    }
                } else {
                    self.is_dragging_sidebar_splitter = false;
                    let _ = self.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "sidebar_w".into(),
                        val: self.sidebar_width.to_string(),
                    });
                }
            } else if is_splitter_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
            }

            // Draw vertical divider bar and tactile knob
            let is_active = is_splitter_hovered || self.is_dragging_sidebar_splitter;
            let divider_color = if is_active {
                self.theme.accent
            } else {
                Color32::from_rgba_unmultiplied(self.theme.muted.r(), self.theme.muted.g(), self.theme.muted.b(), 65)
            };
            let panel_top = editor_panel_rect.min.y;
            let panel_bottom = editor_panel_rect.max.y;
            painter.line_segment(
                [pos2(center_x, panel_top), pos2(center_x, panel_bottom)],
                Stroke::new(1.0, divider_color),
            );
            let knob_w = if is_active { 7.0 } else { 5.0 };
            let knob_h = 42.0;
            let knob_rect = Rect::from_center_size(
                pos2(center_x, (panel_top + panel_bottom) * 0.5),
                vec2(knob_w, knob_h),
            );
            painter.rect_filled(knob_rect, 3.0, divider_color);

            let knob_mid = knob_rect.center();
            let grip_color = self.theme.bg;
            for dy in [-6.0, 0.0, 6.0] {
                painter.line_segment(
                    [pos2(knob_mid.x - 1.5, knob_mid.y + dy), pos2(knob_mid.x + 1.5, knob_mid.y + dy)],
                    Stroke::new(1.0, grip_color),
                );
            }
        } else {
            self.is_dragging_sidebar_splitter = false;
        }

        // Render Dedicated Documentation Sidebar on the left (inside Doc mode)
        if self.mode == Mode::Doc && self.sidebar_open {
            if let Some(sb_rect) = layout.sidebar_rect {
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if sb_rect.contains(pos) {
                            self.doc_sidebar_focused = true;
                        } else if editor_panel_rect.contains(pos) {
                            self.doc_sidebar_focused = false;
                        }
                    }
                }

                if let Some(action) = crate::docs::render_doc_sidebar(
                    ui,
                    &painter,
                    sb_rect,
                    self.active_doc_idx,
                    self.doc_selected_idx,
                    self.doc_sidebar_focused,
                    &self.theme,
                ) {
                    match action {
                        crate::docs::DocSidebarAction::SelectDoc(idx) => {
                            self.load_doc_by_index(idx, now);
                            self.doc_sidebar_focused = true;
                        }
                        crate::docs::DocSidebarAction::ToggleSidebar => {
                            self.sidebar_open = !self.sidebar_open;
                            self.doc_sidebar_focused = self.sidebar_open;
                            let msg = if self.sidebar_open {
                                "Documentation sidebar opened"
                            } else {
                                "Documentation sidebar collapsed into full-width reader (Ctrl+B to reopen)"
                            };
                            self.set_status(msg, now);
                        }
                        crate::docs::DocSidebarAction::BackToEditor => {
                            self.mode = Mode::Normal;
                            self.set_status("Switched to Notes Editor", now);
                        }
                        crate::docs::DocSidebarAction::OpenSettings => {
                            self.settings_open = true;
                            self.settings_just_opened = true;
                        }
                    }
                }
            }
        }

        // Detached Editor & Preview Panel Surface (subtle card background & border derived from theme)
        painter.rect(
            editor_panel_rect,
            5.0,
            self.theme.bg,
            Stroke::new(1.0, self.theme.border()),
            egui::StrokeKind::Inside,
        );

        // Tab strip at the top of the editor panel (inside the panel card)
        let tab_bar_rect = Rect::from_min_max(
            editor_panel_rect.min,
            pos2(editor_panel_rect.max.x, editor_panel_rect.min.y + crate::view_editor::TAB_ROW_H),
        );

        if self.mode == Mode::Normal {
            self.sync_active_tab();
            let tab_items: Vec<crate::view_editor::TabItem> = self
                .open_notes
                .iter()
                .enumerate()
                .map(|(idx, note)| crate::view_editor::TabItem {
                    title: &note.title,
                    is_dirty: note.is_dirty,
                    is_active: idx == self.active_tab,
                })
                .collect();

            let active_changed = self.active_tab != self.last_active_tab;
            if active_changed {
                self.last_active_tab = self.active_tab;
            }

            if let Some(action) = crate::view_editor::render_tab_bar(
                ui,
                &painter,
                tab_bar_rect,
                &tab_items,
                &self.theme,
                &mut self.tab_scroll_offset,
                active_changed,
            ) {
                match action {
                    crate::view_editor::TabAction::Select(idx) => {
                        self.switch_tab(idx, now);
                    }
                    crate::view_editor::TabAction::Close(idx) => {
                        self.close_tab(idx, now);
                    }
                }
            }
        } else if self.mode == Mode::Doc {
            let docs = crate::docs::get_docs();
            let tab_items: Vec<crate::view_editor::TabItem> = self
                .open_doc_tabs
                .iter()
                .enumerate()
                .map(|(idx, &doc_idx)| {
                    let title = docs.get(doc_idx).map(|d| d.title).unwrap_or("Guide");
                    crate::view_editor::TabItem {
                        title,
                        is_dirty: false,
                        is_active: idx == self.active_doc_tab,
                    }
                })
                .collect();

            let active_doc_changed = self.active_doc_tab != self.last_active_doc_tab;
            if active_doc_changed {
                self.last_active_doc_tab = self.active_doc_tab;
            }

            if let Some(action) = crate::view_editor::render_tab_bar(
                ui,
                &painter,
                tab_bar_rect,
                &tab_items,
                &self.theme,
                &mut self.doc_tab_scroll_offset,
                active_doc_changed,
            ) {
                match action {
                    crate::view_editor::TabAction::Select(idx) => {
                        self.switch_doc_tab(idx, now);
                    }
                    crate::view_editor::TabAction::Close(idx) => {
                        self.close_doc_tab(idx, now);
                    }
                }
            }
        }

        // Body area below tab strip (for editor, gutter, preview)
        let body_rect = if self.mode == Mode::Normal || self.mode == Mode::Doc {
            Rect::from_min_max(
                pos2(editor_panel_rect.min.x, tab_bar_rect.max.y),
                editor_panel_rect.max,
            )
        } else {
            editor_panel_rect
        };

        // Split Editor & Preview panes setup inside body_rect
        let is_preview_active = self.preview_open && self.mode == Mode::Normal;
        let divider_w = 12.0;
        let (actual_editor_rect, preview_rect_opt, divider_rect_opt) = if is_preview_active {
            let total_w = body_rect.width();
            let available_w = (total_w - divider_w).max(200.0);
            let min_w = 120.0f32;
            let max_w = (available_w - 120.0f32).max(min_w);
            let left_w = (available_w * self.split_ratio).clamp(min_w, max_w);

            let left_rect = Rect::from_min_max(
                body_rect.min,
                pos2(body_rect.min.x + left_w, body_rect.max.y),
            );
            let divider_rect = Rect::from_min_max(
                pos2(left_rect.max.x, body_rect.min.y),
                pos2(left_rect.max.x + divider_w, body_rect.max.y),
            );
            let right_rect = Rect::from_min_max(
                pos2(divider_rect.max.x, body_rect.min.y),
                body_rect.max,
            );
            (left_rect, Some(right_rect), Some(divider_rect))
        } else {
            (body_rect, None, None)
        };

        // Keep visual lines updated to exact editor width (accounting for preview split and line numbers)
        let target_ed = if self.mode == Mode::Doc { &self.doc_ed } else { &self.ed };
        let total_lines = (target_ed.buf.iter().filter(|&&c| c == '\n').count() + 1).max(1);
        let digits = total_lines.to_string().len().max(2);
        let gutter_space = if self.show_line_numbers {
            (digits as f32 * cw + 10.0).max(22.0) + 6.0
        } else {
            0.0
        };

        let effective_editor_w = actual_editor_rect.width();
        let text_area_w = (effective_editor_w - gutter_space - 10.0).max(100.0);
        let max_cols = (text_area_w / cw).floor().max(15.0) as usize;
        self.visual_lines = target_ed.compute_visual_lines(max_cols);



        // Active View rendering delegated to dedicated view modules
        match self.mode {
            Mode::Normal | Mode::Doc => {
                let original_caret_kind = self.caret.kind;
                let active_vim_mode = if self.editor_input_mode == EditorInputMode::Vim {
                    Some(self.vim.mode)
                } else {
                    None
                };
                self.caret.kind = crate::caret::resolve_caret_kind(
                    self.editor_input_mode,
                    active_vim_mode,
                    original_caret_kind,
                );

                let search_matches = if self.editor_input_mode == EditorInputMode::Vim
                    && (!self.vim.search.match_indices.is_empty() || self.vim.is_searching())
                {
                    let q_len = if self.vim.is_searching() {
                        self.vim.search.query.chars().count()
                    } else {
                        self.vim.search.last_query.chars().count()
                    };
                    if q_len > 0 && !self.vim.search.match_indices.is_empty() {
                        Some((self.vim.search.match_indices.as_slice(), q_len))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let (target_ed_mut, target_scroll_y) = if self.mode == Mode::Doc {
                    (&mut self.doc_ed, &mut self.doc_scroll_y)
                } else {
                    (&mut self.ed, &mut self.scroll_y)
                };

                if let (Some(_), Some(divider_rect)) = (preview_rect_opt, divider_rect_opt) {
                    let total_w = editor_panel_rect.width();
                    let available_w = (total_w - divider_w).max(200.0);

                    // Generous hit box for dragging so mouse never slips off (prevents drag dropping)
                    let divider_hit_rect = divider_rect.expand2(vec2(8.0, 0.0));
                    let is_divider_hovered = ui.rect_contains_pointer(divider_hit_rect);

                    let primary_down = ui.input(|i| i.pointer.primary_down());
                    let primary_pressed = ui.input(|i| i.pointer.primary_clicked() || i.pointer.button_pressed(egui::PointerButton::Primary));

                    if is_divider_hovered && primary_pressed {
                        self.is_dragging_splitter = true;
                    }

                    if self.is_dragging_splitter {
                        if primary_down {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                            if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos())) {
                                let new_ratio = ((pos.x - body_rect.min.x - divider_w * 0.5) / available_w).clamp(0.15, 0.85);
                                self.split_ratio = new_ratio;
                            }
                        } else {
                            self.is_dragging_splitter = false;
                        }
                    } else if is_divider_hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                    }

                    let is_active = is_divider_hovered || self.is_dragging_splitter;
                    let divider_color = if is_active {
                        self.theme.accent
                    } else {
                        Color32::from_rgba_unmultiplied(self.theme.muted.r(), self.theme.muted.g(), self.theme.muted.b(), 65)
                    };
                    let mid_x = divider_rect.center().x;
                    painter.line_segment(
                        [pos2(mid_x, divider_rect.min.y), pos2(mid_x, divider_rect.max.y)],
                        Stroke::new(1.0, divider_color),
                    );
                    // Center pill knob handle with tactile grip dots
                    let knob_w = if is_active { 7.0 } else { 5.0 };
                    let knob_h = 42.0;
                    let knob_rect = Rect::from_center_size(pos2(mid_x, actual_editor_rect.center().y), vec2(knob_w, knob_h));
                    painter.rect_filled(knob_rect, 3.0, divider_color);

                    let knob_mid = knob_rect.center();
                    let grip_color = self.theme.bg;
                    for dy in [-6.0, 0.0, 6.0] {
                        painter.line_segment(
                            [pos2(knob_mid.x - 1.5, knob_mid.y + dy), pos2(knob_mid.x + 1.5, knob_mid.y + dy)],
                            Stroke::new(1.0, grip_color),
                        );
                    }
                } else {
                    self.is_dragging_splitter = false;
                }

                render_editor_body(
                    ui,
                    &painter,
                    bounds,
                    actual_editor_rect,
                    target_ed_mut,
                    &self.visual_lines,
                    &mut self.caret,
                    target_scroll_y,
                    &self.theme,
                    self.font_size,
                    cw,
                    lh,
                    dt,
                    now,
                    typed,
                    self.settings_open
                        || self.search_open
                        || self.help_open
                        || self.rename_open
                        || self.delete_confirm_open
                        || self.is_dragging_splitter
                        || self.is_dragging_sidebar_splitter,
                    search_matches,
                    self.show_line_numbers,
                    active_vim_mode,
                );
                self.caret.kind = original_caret_kind;

                // Render Live Markdown Preview side-by-side if active
                if let Some(p_rect) = preview_rect_opt {
                    let note_text = self.ed.text();
                    render_markdown_preview(
                        ui,
                        &painter,
                        p_rect,
                        &note_text,
                        &mut self.preview_scroll_y,
                        &self.theme,
                        self.font_size,
                    );
                }

                // Floating Keystroke Card (Vim showcmd): large borderless capsule pill — bottom-right of editor
                if self.editor_input_mode == EditorInputMode::Vim {
                    let card_anchor = pos2(actual_editor_rect.max.x - 16.0, actual_editor_rect.max.y - 20.0);
                    self.showcmd.render_card(&painter, card_anchor, self.theme.accent, now);
                }
            }
            Mode::Stats => {
                let today_str = Local::now().date_naive().format("%Y-%m-%d").to_string();
                let yest_str = (Local::now().date_naive() - chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string();
                render_stats(
                    ui,
                    editor_panel_rect,
                    &self.today_activity,
                    &self.activity_history,
                    self.lifetime_activity,
                    self.total_notes_count,
                    &self.theme,
                    &today_str,
                    &yest_str,
                );
            }
            Mode::ScanReport => {
                crate::scan_view::render_scan_view(
                    ui,
                    &painter,
                    editor_panel_rect,
                    self.active_scan_result.as_ref(),
                    self.active_scan_error.as_ref().map(|(u, e)| (u.as_str(), e.as_str())),
                    &mut self.scan_report_scroll_y,
                    &self.theme,
                    self.font_size,
                );
            }
            Mode::ScanHistory => {
                let opened_idx = crate::scan_history_view::render_scan_history(
                    ui,
                    &painter,
                    editor_panel_rect,
                    &self.past_scans,
                    &mut self.scan_history_selected,
                    &mut self.scan_history_scroll_y,
                    &self.theme,
                    self.font_size,
                );
                if let Some(idx) = opened_idx {
                    if let Some(record) = self.past_scans.get(idx) {
                        let findings: Vec<webscan::Finding> = serde_json::from_str(&record.findings_json).unwrap_or_default();
                        let result = webscan::ScanResult {
                            url: record.url.clone(),
                            status_code: 200,
                            response_time_ms: 0,
                            tls: None,
                            server_header: None,
                            page_size_bytes: 0,
                            note: record.note.clone(),
                            findings,
                        };
                        self.active_scan_result = Some(result);
                        self.active_scan_error = None;
                        self.scan_report_scroll_y = 0.0;
                        self.mode = Mode::ScanReport;
                    }
                }
            }
        }

        // Update Vim keystroke HUD and ShowCmd card timeout
        if self.editor_input_mode == EditorInputMode::Vim {
            self.vim.update_hud(now);
            self.showcmd.update(now);
        }

        // Check for search status feedback from Vim engine
        if let Some(msg) = self.vim.status_feedback.take() {
            self.set_status(msg, now);
        }

        // Bottom Dock (Shows active editing mode badge, status feedback, word stats)
        let (row, col) = if self.mode == Mode::Doc {
            self.doc_ed.visual_row_col(&self.visual_lines)
        } else {
            self.ed.visual_row_col(&self.visual_lines)
        };
        let word_count = if self.mode == Mode::Doc {
            self.doc_ed.text().split_whitespace().count()
        } else {
            self.ed.text().split_whitespace().count()
        };
        let mode_badge_str = match self.editor_input_mode {
            EditorInputMode::Vim => self.vim.compact_label(),
            EditorInputMode::Hybrid => "HYBRID".to_string(),
        };

        let search_prompt = if self.editor_input_mode == EditorInputMode::Vim && self.vim.is_searching() {
            let symbol = if self.vim.search.backward { "?" } else { "/" };
            Some((symbol, self.vim.search.query.as_str(), self.vim.search.match_indices.len()))
        } else {
            None
        };

        render_bottom_dock(
            ui,
            &painter,
            cmd_bar_rect,
            0.0,
            self.in_command,
            &self.cmd_ed.text(),
            self.cmd_ed.cur,
            self.cmd_ed.selected_range(),
            &self.status_msg,
            self.status_time,
            now,
            row + 1,
            col + 1,
            word_count,
            Some(mode_badge_str.as_str()),
            search_prompt,
            self.theme.accent,
            self.theme.muted,
        );

        // Sleek Sidebar (Ctrl+B)
        if self.sidebar_open && self.mode != Mode::Doc {
            if let Some(sb_rect) = layout.sidebar_rect {
                let active_mode_idx = match self.mode {
                    Mode::Normal => 0,
                    Mode::Stats => 1,
                    Mode::Doc | Mode::ScanReport | Mode::ScanHistory => 0,
                };
                let action = render_sidebar(
                    ui,
                    &painter,
                    sb_rect,
                    active_mode_idx,
                    self.active_note_id,
                    &self.notes_list,
                    self.sidebar_notes_limit,
                    self.total_notes_count,
                    self.is_dirty,
                    &self.theme,
                    self.sidebar_selected_idx,
                    self.sidebar_focused,
                );

                // Click outside sidebar releases sidebar keyboard focus
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if sb_rect.contains(pos) {
                            self.sidebar_focused = true;
                        } else if editor_panel_rect.contains(pos) {
                            self.sidebar_focused = false;
                        }
                    }
                }

            if let Some(act) = action {
                match act {
                    SidebarAction::SwitchMode(idx) => {
                        match idx {
                            0 => self.mode = Mode::Normal,
                            1 => {
                                self.reload_db_state();
                                self.mode = Mode::Stats;
                            }
                            _ => {}
                        }
                    }
                    SidebarAction::LoadNote { id, topic, body, index } => {
                        self.load_note(id, topic, body, now);
                        self.sidebar_selected_idx = index;
                        // Keep focus on the sidebar and on the loaded note
                        self.sidebar_focused = true;
                    }
                    SidebarAction::NewNote => {
                        if let Some(cur) = self.open_notes.get_mut(self.active_tab) {
                            cur.editor = self.ed.clone();
                            cur.title = self.active_note_title.clone();
                            cur.scroll_y = self.scroll_y;
                            cur.is_dirty = self.is_dirty;
                        }
                        self.active_note_id = None;
                        self.save_active_note_id();
                        self.active_note_title = "Untitled Note".to_string();
                        self.ed.clear();
                        self.mode = Mode::Normal;
                        self.vim.set_mode(crate::vim::VimSubMode::Normal, &mut self.ed);
                        self.is_dirty = false;
                        self.scroll_y = 0.0;
                        self.open_notes.push(OpenNote {
                            id: 0,
                            title: "Untitled Note".to_string(),
                            editor: self.ed.clone(),
                            scroll_y: 0.0,
                            is_dirty: false,
                        });
                        self.active_tab = self.open_notes.len() - 1;
                        self.save_open_tabs();
                        self.set_status("Created new note", now);
                    }
                    SidebarAction::DeleteNote(id) => {
                        self.pending_delete_note_id = Some(id);
                        self.delete_confirm_open = true;
                        self.delete_just_opened = true;
                    }
                    SidebarAction::ToggleNotesLimit => {
                        self.sidebar_notes_limit = if self.sidebar_notes_limit >= 100 { 50 } else { 100 };
                        self.reload_db_state();
                    }
                    SidebarAction::OpenSettings => {
                        self.settings_open = true;
                        self.settings_just_opened = true;
                    }
                }
            }
        }
    }

        // Preferences Modal (Ctrl+,)
        if self.settings_open {
            painter.rect_filled(bounds, 0.0, Color32::from_black_alpha(175));

            let modal_w = 780.0f32.min(bounds.width() - 40.0);
            let modal_h = 560.0f32.min(bounds.height() - 40.0);
            let modal_rect = Rect::from_center_size(bounds.center(), eframe::egui::vec2(modal_w, modal_h));

            // Dismiss modal if clicking backdrop outside dialog (skip on frame opened to prevent immediate close)
            if !self.settings_just_opened && ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if !modal_rect.contains(pos) && bounds.contains(pos) {
                        self.settings_open = false;
                    }
                }
            }

            painter.rect(
                modal_rect,
                6.0,
                Color32::from_rgb(14, 15, 18),
                eframe::egui::Stroke::new(1.0, Color32::from_rgb(44, 48, 62)),
                eframe::egui::StrokeKind::Inside,
            );

            // Close icon button (✕) on the right side of the settings modal
            let close_size = 26.0;
            let close_rect = Rect::from_min_size(
                pos2(modal_rect.max.x - close_size - 10.0, modal_rect.min.y + 10.0),
                vec2(close_size, close_size),
            );
            let close_hover = ui.rect_contains_pointer(close_rect);
            if close_hover {
                painter.rect_filled(close_rect, 4.0, Color32::from_rgb(220, 50, 50));
            }
            let close_color = if close_hover { Color32::WHITE } else { Color32::from_gray(140) };
            let c = close_rect.center();
            let d = 4.5;
            painter.line_segment([pos2(c.x - d, c.y - d), pos2(c.x + d, c.y + d)], Stroke::new(1.5, close_color));
            painter.line_segment([pos2(c.x + d, c.y - d), pos2(c.x - d, c.y + d)], Stroke::new(1.5, close_color));
            if close_hover && ui.input(|i| i.pointer.primary_clicked()) {
                self.settings_open = false;
            }

            let tab_w = 175.0;
            let tabs_rect = Rect::from_min_max(modal_rect.min, pos2(modal_rect.min.x + tab_w, modal_rect.max.y));
            let panel_rect = Rect::from_min_max(pos2(modal_rect.min.x + tab_w, modal_rect.min.y), modal_rect.max);

            render_setting_tabs(ui, &painter, tabs_rect, &mut self.active_setting_tab, self.theme.accent);

            // Consume scroll events inside the modal so they never reach the editor
            let scroll_consumed = ui.input(|i| i.raw_scroll_delta.y);
            let _ = scroll_consumed; // acknowledged

            let db_tx = self.db_tx.clone();
            let mut on_save = |key: &str, val: &str| {
                let _ = db_tx.send(DbMsg::SaveSetting {
                    key: key.to_string(),
                    val: val.to_string(),
                });
            };
            let panel_action = render_setting_panel(
                ui,
                &painter,
                panel_rect,
                self.active_setting_tab,
                &mut self.editor_input_mode,
                &mut self.caret,
                &mut self.sound,
                &mut self.theme,
                &mut self.backup_dir,
                self.last_backup_status.as_deref(),
                &self.updater,
                &mut on_save,
            );

            match panel_action {
                Some(SettingPanelAction::TriggerBackup) => {
                    self.trigger_backup(now);
                }
                Some(SettingPanelAction::CheckUpdates) => {
                    self.updater.check_for_updates(env!("CARGO_PKG_VERSION"));
                }
                Some(SettingPanelAction::DownloadUpdate) => {
                    self.updater.start_download();
                }
                Some(SettingPanelAction::RestartToApply) => {
                    let _ = self.updater.restart_and_apply();
                }
                None => {}
            }
            self.settings_just_opened = false;
        }

        // Fuzzy Search Modal (Ctrl+P)
        if self.search_open {
            let action = render_search_modal(
                ui,
                &painter,
                bounds,
                &mut self.search_query,
                &self.search_results,
                &mut self.search_selected,
                self.theme.accent,
                self.search_just_opened,
            );
            self.search_just_opened = false;

            if let Some(item) = action.selected_item {
                if let Some(note) = self.notes_list.iter().find(|n| n.id == item.id).cloned() {
                    self.load_note(note.id, note.topic, note.body, now);
                }
            }
            if action.should_close {
                self.search_open = false;
            }
        }

        // Rename Modal (Ctrl+R)
        if self.rename_open {
            let action = render_rename_modal(
                ui,
                &painter,
                bounds,
                &mut self.rename_input,
                self.theme.accent,
                self.rename_just_opened,
            );
            self.rename_just_opened = false;

            if let Some(new_title) = action.confirmed_title {
                self.rename_active_note(&new_title, now);
            }
            if action.should_close {
                self.rename_open = false;
            }
        }

        // Delete Confirmation Modal (Ctrl+Shift+D or :d / :delete / :rm or sidebar trash icon)
        if self.delete_confirm_open {
            let note_title = if let Some(target_id) = self.pending_delete_note_id {
                self.notes_list
                    .iter()
                    .find(|n| n.id == target_id)
                    .map(|n| n.topic.clone())
                    .unwrap_or_else(|| self.active_note_title.clone())
            } else {
                self.active_note_title.clone()
            };

            let action = render_delete_confirm_modal(
                ui,
                &painter,
                bounds,
                &note_title,
                self.delete_just_opened,
            );
            self.delete_just_opened = false;

            if action.confirmed {
                if let Some(target_id) = self.pending_delete_note_id.take() {
                    let _ = self.db_tx.send(DbMsg::DeleteNote { id: target_id });
                    if let Some(ref db) = self.db {
                        let _ = db.delete_note(target_id);
                    }
                    if let Some(pos) = self.open_notes.iter().position(|n| n.id == target_id) {
                        self.close_tab(pos, now);
                    } else if self.active_note_id == Some(target_id) {
                        self.delete_active_note(now);
                    } else {
                        self.notes_list.retain(|n| n.id != target_id);
                        self.reload_db_state();
                    }
                    self.set_status("Deleted note", now);
                } else {
                    self.delete_active_note(now);
                }
                self.delete_confirm_open = false;
            } else if action.should_close {
                self.pending_delete_note_id = None;
                self.delete_confirm_open = false;
            }
        }

        // Help & Guidance Center Modal (:help or F1)
        if self.help_open {
            let action = render_help_panel(
                ui,
                &painter,
                bounds,
                &mut self.help_tab,
                &mut self.help_scroll_y,
                self.theme.accent,
                self.help_just_opened,
            );
            self.help_just_opened = false;

            if action.open_docs {
                self.help_open = false;
                self.open_docs_mode(now);
            }
            if action.open_settings {
                self.help_open = false;
                self.settings_open = true;
            }
            if action.should_close {
                self.help_open = false;
            }
        }

        // Accent Color Customizer Dropdown
        if self.accent_dropdown_open {
            let default_theme = Theme::from_kind(self.theme.kind);
            if let Some(act) = crate::accent::render_accent_dropdown(
                ui,
                &painter,
                accent_anchor_rect,
                &mut self.accent_overrides,
                &self.theme,
                &default_theme,
            ) {
                match act {
                    crate::accent::AccentAction::Changed => {
                        let mut t = Theme::from_kind(self.theme.kind);
                        self.accent_overrides.apply(&mut t);
                        self.theme = t;
                        if let Some(ref db) = self.db {
                            self.accent_overrides.save_to_db(db);
                        }
                    }
                    crate::accent::AccentAction::ResetAll => {
                        self.accent_overrides.clear();
                        self.theme = Theme::from_kind(self.theme.kind);
                        if let Some(ref db) = self.db {
                            self.accent_overrides.save_to_db(db);
                        }
                    }
                    crate::accent::AccentAction::Close => {
                        self.accent_dropdown_open = false;
                    }
                }
            }
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _v: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                ctx.send_viewport_cmd(cmd);
            }
        }

        let now = ctx.input(|i| i.time);
        let dt = ctx.input(|i| i.unstable_dt).clamp(0.0, 0.05);

        // Precompute visual lines so keyboard navigation (ArrowUp, ArrowDown, PageUp, PageDown) uses accurate visual layout
        let (cw, _) = self.cell_size(ctx);
        let screen_w = ctx.screen_rect().width();
        let left_margin = if self.mode == Mode::Doc || self.sidebar_open {
            crate::layout::GAP + self.sidebar_width + crate::layout::SPLITTER_BAR_W + crate::layout::GAP
        } else {
            crate::layout::GAP
        };
        let editor_w = (screen_w - left_margin - crate::layout::GAP).max(100.0);
        let is_preview_active = self.preview_open && self.mode == Mode::Normal;
        let effective_editor_w = if is_preview_active {
            let divider_w = 12.0;
            let available_w = (editor_w - divider_w).max(200.0);
            let min_w = 120.0f32;
            let max_w = (available_w - 120.0f32).max(min_w);
            (available_w * self.split_ratio).clamp(min_w, max_w)
        } else {
            editor_w
        };
        let gutter_space = if self.show_line_numbers { 42.0 } else { 0.0 };
        let text_area_w = (effective_editor_w - gutter_space - 8.0).max(100.0);
        let max_cols = (text_area_w / cw).floor().max(15.0) as usize;
        let active_ed = if self.mode == Mode::Doc { &self.doc_ed } else { &self.ed };
        self.visual_lines = active_ed.compute_visual_lines(max_cols);

        let typed = handle_input(self, ctx, now);

        // Activity tracking: if user interacted in the last 60s, count dt towards active editor time
        if (now - self.last_char_time) < 60.0 {
            self.pending_secs += dt;
        }
        if typed {
            self.pending_keys += 1;
            // Track completed word if space or newline typed
            if let Some(&last_ch) = self.ed.buf.get(self.ed.cur.saturating_sub(1)) {
                if last_ch.is_whitespace() {
                    self.pending_words += 1;
                }
            }
        }

        // Periodic auto-flush of activity stats to database every 10 seconds
        if (now - self.last_flush_time) > 10.0 {
            self.flush_activity(now);
            self.save_caret_position();
        }

        // Auto-save when idle for 1.2s in Normal mode
        if self.is_dirty && (now - self.last_char_time) > 1.2 && self.mode == Mode::Normal {
            self.quick_save_active_note(now);
            self.save_caret_position();
        }

        // Save session state immediately if window close is requested
        if ctx.input(|i| i.viewport().close_requested()) {
            self.sync_save_session();
        }

        // Live fuzzy search filter update
        if self.search_open {
            self.update_search_results();
        }

        // Check Webscan background worker channel
        if let Some(ref rx) = self.scan_rx {
            if let Ok(msg) = rx.try_recv() {
                let target_url = self.scan_in_progress.take().unwrap_or_default();
                self.scan_rx = None;
                match msg {
                    Ok(scan_res) => {
                        // Persist scan result to SQLite
                        if let Some(ref db) = self.db {
                            if let Ok(json_str) = serde_json::to_string(&scan_res.findings) {
                                let _ = db.save_scan(&scan_res.url, scan_res.note.as_deref(), &json_str);
                            }
                        }
                        self.active_scan_result = Some(scan_res);
                        self.active_scan_error = None;
                        self.scan_report_scroll_y = 0.0;
                        self.mode = Mode::ScanReport;
                        self.set_status("Scan completed successfully", now);
                    }
                    Err(err_msg) => {
                        self.active_scan_result = None;
                        self.active_scan_error = Some((target_url, err_msg));
                        self.scan_report_scroll_y = 0.0;
                        self.mode = Mode::ScanReport;
                        self.set_status("Scan failed (see report for details)", now);
                    }
                }
            }
        }

        window_shortcuts(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.draw(ui, dt, now, typed);
            });

        // Silky smooth repaint: 120Hz/144Hz while typing, sliding, or animating; idle 100ms when resting
        let focused = ctx.input(|i| i.focused);
        if focused
            && (self.caret.is_animating(now)
                || !self.showcmd.text.is_empty()
                || self.search_open
                || self.settings_open
                || self.help_open
                || self.rename_open
                || self.delete_confirm_open)
        {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sync_save_session();
    }
}
