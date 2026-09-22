//! Command line dispatcher for vim-like commands (:w, :r, :d, :add, :sound, :caret, :theme, :stats, :quit).

use crate::app::App;
use crate::caret::CaretKind;
use crate::db_worker::DbMsg;
use crate::mode::Mode;
use crate::sound::SoundProfile;
use crate::theme::{Theme, ThemeKind};

pub fn execute_command(app: &mut App, raw: &str, now: f64) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let cmd = parts[0].to_lowercase();
    let args = if parts.len() > 1 {
        trimmed[parts[0].len()..].trim()
    } else {
        ""
    };

    match cmd.as_str() {
        "help" => {
            app.set_status(
                ":w | :d | :set showcmd [on/off] | :vim [on/off] | :theme | :sound | :clear",
                now,
            );
        }
        "set" => {
            let opt = args.to_lowercase();
            let opt = opt.trim();
            match opt {
                "showcmd" | "sc" => {
                    app.showcmd.enabled = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "true".into(),
                    });
                    app.set_status(":set showcmd (Keystroke card ON)", now);
                }
                "noshowcmd" | "nosc" => {
                    app.showcmd.enabled = false;
                    app.showcmd.clear();
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "false".into(),
                    });
                    app.set_status(":set noshowcmd (Keystroke card OFF)", now);
                }
                "showcmd!" | "sc!" => {
                    app.showcmd.enabled = !app.showcmd.enabled;
                    if !app.showcmd.enabled {
                        app.showcmd.clear();
                    }
                    let val = if app.showcmd.enabled { "true" } else { "false" };
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: val.into(),
                    });
                    let msg = if app.showcmd.enabled {
                        ":set showcmd (Keystroke card ON)"
                    } else {
                        ":set noshowcmd (Keystroke card OFF)"
                    };
                    app.set_status(msg, now);
                }
                _ => {
                    app.set_status(
                        format!("Unknown option: :set {}. Try :set showcmd or :set noshowcmd", opt),
                        now,
                    );
                }
            }
        }
        "showcmd" => {
            match args.to_lowercase().trim() {
                "off" | "disable" | "0" => {
                    app.showcmd.enabled = false;
                    app.showcmd.clear();
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "false".into(),
                    });
                    app.set_status("showcmd disabled (Keystroke card OFF)", now);
                }
                "on" | "enable" | "1" => {
                    app.showcmd.enabled = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "true".into(),
                    });
                    app.set_status("showcmd enabled (Keystroke card ON)", now);
                }
                _ => {
                    app.showcmd.enabled = !app.showcmd.enabled;
                    if !app.showcmd.enabled {
                        app.showcmd.clear();
                    }
                    let val = if app.showcmd.enabled { "true" } else { "false" };
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: val.into(),
                    });
                    let msg = if app.showcmd.enabled {
                        "showcmd enabled (Keystroke card ON)"
                    } else {
                        "showcmd disabled (Keystroke card OFF)"
                    };
                    app.set_status(msg, now);
                }
            }
        }
        "vim" => {
            match args.to_lowercase().trim() {
                "on" | "enable" | "1" => {
                    app.editor_input_mode = crate::app::EditorInputMode::Vim;
                    app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
                    let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "editor_mode".into(),
                        val: "vim".into(),
                    });
                    app.set_status("Vim Mode Enabled (-- NORMAL --)", now);
                }
                "off" | "disable" | "0" => {
                    app.editor_input_mode = crate::app::EditorInputMode::Hybrid;
                    let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "editor_mode".into(),
                        val: "hybrid".into(),
                    });
                    app.set_status("Hybrid Mode Enabled (Modern IDE)", now);
                }
                _ => {
                    if app.editor_input_mode == crate::app::EditorInputMode::Vim {
                        app.editor_input_mode = crate::app::EditorInputMode::Hybrid;
                        let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                            key: "editor_mode".into(),
                            val: "hybrid".into(),
                        });
                        app.set_status("Switched to Hybrid Mode (Modern IDE)", now);
                    } else {
                        app.editor_input_mode = crate::app::EditorInputMode::Vim;
                        app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
                        let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                            key: "editor_mode".into(),
                            val: "vim".into(),
                        });
                        app.set_status("Switched to Vim Mode (-- NORMAL --)", now);
                    }
                }
            }
        }
        "mode" => {
            match args.to_lowercase().trim() {
                "vim" => {
                    app.editor_input_mode = crate::app::EditorInputMode::Vim;
                    app.vim.set_mode(crate::vim::VimSubMode::Normal, &mut app.ed);
                    let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "editor_mode".into(),
                        val: "vim".into(),
                    });
                    app.set_status("Vim Mode Active (-- NORMAL --)", now);
                }
                "hybrid" => {
                    app.editor_input_mode = crate::app::EditorInputMode::Hybrid;
                    let _ = app.db_tx.send(crate::db_worker::DbMsg::SaveSetting {
                        key: "editor_mode".into(),
                        val: "hybrid".into(),
                    });
                    app.set_status("Hybrid Mode Active (Modern IDE)", now);
                }
                _ => {
                    let current = match app.editor_input_mode {
                        crate::app::EditorInputMode::Hybrid => "Hybrid (Modern IDE)",
                        crate::app::EditorInputMode::Vim => "Vim (Modal Engine)",
                    };
                    app.set_status(format!("Mode: {} (type :vim or :mode vim/hybrid)", current), now);
                }
            }
        }
        "w" | "save" => {
            app.quick_save_active_note(now);
        }
        "r" | "rename" => {
            if !args.is_empty() {
                app.active_note_title = args.to_string();
                if let Some(id) = app.active_note_id {
                    if let Some(ref db) = app.db {
                        let _ = db.rename_note(id, args);
                    }
                    if let Some(n) = app.notes_list.iter_mut().find(|n| n.id == id) {
                        n.topic = args.to_string();
                    }
                    app.set_status("Renamed", now);
                }
            } else {
                app.rename_open = true;
                app.rename_input = app.active_note_title.clone();
                app.rename_just_opened = true;
            }
        }
        "d" | "delete" | "rm" => {
            app.delete_confirm_open = true;
            app.delete_just_opened = true;
        }
        "export" => {
            let clean_arg = args.trim_matches(|c| c == '"' || c == '\'').trim();
            let title = if app.active_note_title.trim().is_empty() {
                "Untitled"
            } else {
                &app.active_note_title
            };
            let safe_name = title
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
                .collect::<String>();
            let default_name = format!("{}.md", safe_name.trim());

            let chosen_path = if clean_arg.is_empty() {
                // Interactive native folder / save file picker
                rfd::FileDialog::new()
                    .set_file_name(&default_name)
                    .add_filter("Markdown Document (*.md)", &["md"])
                    .add_filter("Plain Text Document (*.txt)", &["txt"])
                    .save_file()
            } else {
                let p = std::path::PathBuf::from(clean_arg);
                if p.is_dir() {
                    // Export into the chosen directory
                    Some(p.join(&default_name))
                } else if p.extension().is_none() {
                    Some(p.with_extension("md"))
                } else {
                    Some(p)
                }
            };

            if let Some(out_path) = chosen_path {
                match std::fs::write(&out_path, app.ed.text()) {
                    Ok(_) => {
                        app.set_status(format!("Exported to: {}", out_path.display()), now);
                    }
                    Err(e) => {
                        app.set_status(format!("Export failed: {}", e), now);
                    }
                }
            } else {
                app.set_status("Export cancelled", now);
            }
        }
        "import" => {
            let clean_arg = args.trim_matches(|c| c == '"' || c == '\'').trim();
            let chosen_file = if clean_arg.is_empty() {
                // Interactive native file picker
                rfd::FileDialog::new()
                    .add_filter("Markdown & Text Files", &["md", "txt", "markdown"])
                    .pick_file()
            } else {
                let mut p = std::path::PathBuf::from(clean_arg);
                if !p.exists() {
                    if let Ok(cur) = std::env::current_dir() {
                        let candidate_direct = cur.join(clean_arg);
                        let candidate_md = cur.join(format!("{}.md", clean_arg));
                        let candidate_txt = cur.join(format!("{}.txt", clean_arg));
                        if candidate_direct.exists() {
                            p = candidate_direct;
                        } else if candidate_md.exists() {
                            p = candidate_md;
                        } else if candidate_txt.exists() {
                            p = candidate_txt;
                        }
                    }
                }
                Some(p)
            };

            if let Some(path) = chosen_file {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let title = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Imported Note")
                            .to_string();
                        let ext = path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("txt");
                        let clean_content = content.replace("\r\n", "\n").replace('\r', "\n");
                        let now_dt = chrono::Local::now().naive_local();
                        if let Some(ref db) = app.db {
                            if let Ok(new_id) = db.add_note(&title, &clean_content, None, now_dt) {
                                app.active_note_id = Some(new_id);
                                app.notes_list.insert(0, core::Note {
                                    id: new_id,
                                    topic: title.clone(),
                                    body: clean_content.clone(),
                                    struggled_with: None,
                                    created_at: now_dt,
                                });
                                app.total_notes_count += 1;
                            }
                        } else {
                            let _ = app.db_tx.send(DbMsg::SaveNote {
                                topic: title.clone(),
                                body: clean_content.clone(),
                                struggled: None,
                            });
                        }
                        app.active_note_title = title.clone();
                        app.ed.set_text(&clean_content);
                        app.ed.cur = 0;
                        app.is_dirty = false;
                        app.scroll_y = 0.0;
                        app.pending_created += 1;
                        app.set_status(format!("Imported: \"{}\" (.{})", title, ext), now);
                    }
                    Err(e) => {
                        app.set_status(format!("Failed to import {}: {}", path.display(), e), now);
                    }
                }
            } else {
                app.set_status("Import cancelled", now);
            }
        }
        "sound" => {
            if let Some(profile) = SoundProfile::parse(args) {
                app.sound.profile = profile;
                let _ = app.db_tx.send(DbMsg::SaveSetting {
                    key: "sound".into(),
                    val: profile.name().to_lowercase(),
                });
                app.set_status(format!("Typing sound set to: {}", profile.name()), now);
            } else {
                app.set_status(
                    "Usage: :sound <off|thocky|clacky|creamy|marbly|poppy|clicky>",
                    now,
                );
            }
        }
        "caret" => {
            if let Some(kind) = CaretKind::parse(args) {
                app.caret.kind = kind;
                let _ = app.db_tx.send(DbMsg::SaveSetting {
                    key: "caret".into(),
                    val: args.into(),
                });
                app.set_status(format!("Caret set to {}", args), now);
            }
        }
        "theme" => {
            if let Some(kind) = ThemeKind::parse(args) {
                app.theme = Theme::from_kind(kind);
                let _ = app.db_tx.send(DbMsg::SaveSetting {
                    key: "theme".into(),
                    val: args.into(),
                });
                app.set_status(format!("Theme set to {}", args), now);
            }
        }
        "stats" => {
            app.mode = Mode::Stats;
        }
        "clear" => {
            app.ed.clear();
            app.is_dirty = true;
            app.scroll_y = 0.0;
            app.set_status("Editor cleared", now);
        }
        "backup" | "snapshot" => {
            if !args.is_empty() {
                app.backup_dir = args.to_string();
            }
            app.trigger_backup(now);
        }
        "quit" | "q" => {
            std::process::exit(0);
        }
        _ => {
            app.set_status(format!("Unknown command: :{}. Type :help", cmd), now);
        }
    }
}
