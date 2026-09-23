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
        "help" | "guidance" | "guide" | "h" | "?" => {
            app.help_open = true;
            app.help_just_opened = true;
            app.set_status("Guidance & Help Center opened (Esc to close)", now);
        }
        "set" => {
            let opt = args.to_lowercase();
            let opt = opt.trim();
            match opt {
                "noshowcmd"
                | "nonshowcmd"
                | "nosc"
                | "nonsc"
                | "noshow"
                | "nonshow"
                | "no_showcmd"
                | "non_showcmd"
                | "showcmd off"
                | "showcmd=off"
                | "showcmd 0"
                | "showcmd=0"
                | "showcmd false"
                | "showcmd=false"
                | "showcmd disable" => {
                    app.showcmd.enabled = false;
                    app.showcmd.clear();
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "false".into(),
                    });
                    app.set_status(":set noshowcmd (Keystroke card OFF)", now);
                }
                "showcmd"
                | "sc"
                | "showcmd on"
                | "showcmd=on"
                | "showcmd 1"
                | "showcmd=1"
                | "showcmd true"
                | "showcmd=true"
                | "showcmd enable" => {
                    app.showcmd.enabled = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "true".into(),
                    });
                    app.set_status(":set showcmd (Keystroke card ON)", now);
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
                "nonu"
                | "nonumber"
                | "no_number"
                | "no_nu"
                | "nu off"
                | "nu=off"
                | "nu 0"
                | "number off"
                | "number=off"
                | "number 0" => {
                    app.show_line_numbers = false;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "line_numbers".into(),
                        val: "false".into(),
                    });
                    app.set_status(":set nonu (Line numbers hidden)", now);
                }
                "nu"
                | "number"
                | "nu on"
                | "nu=on"
                | "nu 1"
                | "number on"
                | "number=on"
                | "number 1" => {
                    app.show_line_numbers = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "line_numbers".into(),
                        val: "true".into(),
                    });
                    app.set_status(":set nu (Line numbers visible)", now);
                }
                "nu!" | "number!" => {
                    app.show_line_numbers = !app.show_line_numbers;
                    let val = if app.show_line_numbers { "true" } else { "false" };
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "line_numbers".into(),
                        val: val.into(),
                    });
                    let msg = if app.show_line_numbers {
                        ":set nu (Line numbers visible)"
                    } else {
                        ":set nonu (Line numbers hidden)"
                    };
                    app.set_status(msg, now);
                }
                "nopreview"
                | "no_preview"
                | "noprev"
                | "preview off"
                | "preview=off"
                | "preview 0"
                | "preview disable" => {
                    app.preview_open = false;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: "false".into(),
                    });
                    app.set_status(":set nopreview (Live preview closed)", now);
                }
                "preview"
                | "prev"
                | "preview on"
                | "preview=on"
                | "preview 1"
                | "preview enable" => {
                    app.preview_open = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: "true".into(),
                    });
                    app.set_status(":set preview (Live preview opened side-by-side)", now);
                }
                "preview!" | "prev!" => {
                    app.preview_open = !app.preview_open;
                    let val = if app.preview_open { "true" } else { "false" };
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: val.into(),
                    });
                    let msg = if app.preview_open {
                        ":set preview (Live preview opened side-by-side)"
                    } else {
                        ":set nopreview (Live preview closed)"
                    };
                    app.set_status(msg, now);
                }
                _ => {
                    app.set_status(
                        format!("Unknown option: :set {}. Try :set nu / :set nonu or :set preview / :set nopreview", opt),
                        now,
                    );
                }
            }
        }
        "showcmd" => {
            match args.to_lowercase().trim() {
                "off" | "disable" | "0" | "false" => {
                    app.showcmd.enabled = false;
                    app.showcmd.clear();
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "showcmd".into(),
                        val: "false".into(),
                    });
                    app.set_status("showcmd disabled (Keystroke card OFF)", now);
                }
                "on" | "enable" | "1" | "true" => {
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
        "noshowcmd" | "nonshowcmd" | "nosc" | "nonsc" | "noshow" | "nonshow" => {
            app.showcmd.enabled = false;
            app.showcmd.clear();
            let _ = app.db_tx.send(DbMsg::SaveSetting {
                key: "showcmd".into(),
                val: "false".into(),
            });
            app.set_status("showcmd disabled (Keystroke card OFF)", now);
        }
        "nu" | "number" => {
            app.show_line_numbers = true;
            let _ = app.db_tx.send(DbMsg::SaveSetting {
                key: "line_numbers".into(),
                val: "true".into(),
            });
            app.set_status("Line numbers exposed (ON)", now);
        }
        "nonu" | "nonumber" => {
            app.show_line_numbers = false;
            let _ = app.db_tx.send(DbMsg::SaveSetting {
                key: "line_numbers".into(),
                val: "false".into(),
            });
            app.set_status("Line numbers hidden (OFF)", now);
        }
        "preview" | "prev" => {
            match args.to_lowercase().trim() {
                "off" | "disable" | "0" | "false" => {
                    app.preview_open = false;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: "false".into(),
                    });
                    app.set_status("Live preview closed", now);
                }
                "on" | "enable" | "1" | "true" => {
                    app.preview_open = true;
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: "true".into(),
                    });
                    app.set_status("Live preview opened side-by-side (drag center knob)", now);
                }
                _ => {
                    app.preview_open = !app.preview_open;
                    let val = if app.preview_open { "true" } else { "false" };
                    let _ = app.db_tx.send(DbMsg::SaveSetting {
                        key: "preview".into(),
                        val: val.into(),
                    });
                    let msg = if app.preview_open {
                        "Live preview opened side-by-side (Ctrl + \\ or drag knob)"
                    } else {
                        "Live preview closed"
                    };
                    app.set_status(msg, now);
                }
            }
        }
        "nopreview" | "noprev" => {
            app.preview_open = false;
            let _ = app.db_tx.send(DbMsg::SaveSetting {
                key: "preview".into(),
                val: "false".into(),
            });
            app.set_status("Live preview closed", now);
        }
        "noh" | "nohl" | "nohlsearch" => {
            app.vim.search.clear_matches();
            app.set_status("Search highlighting cleared (:noh)", now);
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
            if app.mode == Mode::Doc {
                app.set_status("Documentation files are read-only (changes not saved).", now);
                return;
            }
            app.quick_save_active_note(now);
        }
        "r" | "rename" => {
            if app.mode == Mode::Doc {
                app.set_status("Documentation files are read-only and cannot be renamed.", now);
                return;
            }
            if !args.is_empty() {
                app.rename_active_note(args, now);
            } else {
                app.rename_open = true;
                app.rename_input = app.active_note_title.clone();
                app.rename_just_opened = true;
            }
        }
        "d" | "delete" | "rm" => {
            if app.mode == Mode::Doc {
                app.set_status("Documentation files cannot be deleted.", now);
                return;
            }
            app.delete_confirm_open = true;
            app.delete_just_opened = true;
        }
        "export" => {
            if app.mode == Mode::ScanReport {
                if let Some(ref res) = app.active_scan_result {
                    let md = crate::scan_view::export_scan_to_markdown(res);
                    let safe_url = res
                        .url
                        .replace("https://", "")
                        .replace("http://", "")
                        .replace('/', "_")
                        .replace(':', "_")
                        .replace('?', "_");
                    let default_name = format!("scan_{}.md", safe_url.trim_matches('_'));
                    let clean_arg = args.trim_matches(|c| c == '"' || c == '\'').trim();
                    let chosen_path = if clean_arg.is_empty() {
                        rfd::FileDialog::new()
                            .set_file_name(&default_name)
                            .add_filter("Markdown Document (*.md)", &["md"])
                            .add_filter("Plain Text Document (*.txt)", &["txt"])
                            .save_file()
                    } else {
                        let p = std::path::PathBuf::from(clean_arg);
                        if p.is_dir() {
                            Some(p.join(&default_name))
                        } else if p.extension().is_none() {
                            Some(p.with_extension("md"))
                        } else {
                            Some(p)
                        }
                    };

                    if let Some(out_path) = chosen_path {
                        match std::fs::write(&out_path, md) {
                            Ok(_) => {
                                app.set_status(format!("Exported scan report: {}", out_path.display()), now);
                            }
                            Err(e) => {
                                app.set_status(format!("Export failed: {}", e), now);
                            }
                        }
                    } else {
                        app.set_status("Export cancelled", now);
                    }
                    return;
                } else {
                    app.set_status("No scan report available to export", now);
                    return;
                }
            }

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
        "doc" | "docs" | "tutorial" | "tutorials" | "document" | "documents" | "documentation" | "documentations" => {
            app.open_docs_mode(now);
        }
        "edit" | "editor" | "note" | "notes" => {
            app.mode = Mode::Normal;
            app.set_status("Switched to Notes Editor", now);
        }
        "scan" => {
            if app.scan_in_progress.is_some() {
                app.set_status("A scan is already in progress...", now);
                return;
            }
            match parse_scan_args(args) {
                Ok((url, opts)) => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    app.scan_rx = Some(rx);
                    app.scan_in_progress = Some(url.clone());
                    app.prev_mode_before_scan = app.mode;
                    app.set_status(format!("Scanning {}...", url), now);

                    let url_clone = url.clone();
                    std::thread::spawn(move || {
                        let result = webscan::scan(&url_clone, &opts);
                        match result {
                            Ok(res) => {
                                let _ = tx.send(Ok(res));
                            }
                            Err(err) => {
                                let _ = tx.send(Err(format!("{}: {:#}", url_clone, err)));
                            }
                        }
                    });
                }
                Err(usage) => {
                    app.set_status(usage, now);
                }
            }
        }
        "scans" | "scanhistory" => {
            if let Some(ref db) = app.db {
                if let Ok(scans) = db.list_scans() {
                    app.past_scans = scans;
                }
            }
            app.prev_mode_before_scan = app.mode;
            app.scan_history_selected = 0;
            app.scan_history_scroll_y = 0.0;
            app.mode = Mode::ScanHistory;
            app.set_status("Webscan History (↑/↓ to navigate, Enter to view report, Esc to exit)", now);
        }
        "quit" | "q" => {
            std::process::exit(0);
        }
        _ => {
            app.set_status(format!("Unknown command: :{}. Type :help", cmd), now);
        }
    }
}

fn parse_scan_args(raw_args: &str) -> Result<(String, webscan::ScanOptions), String> {
    let mut url = String::new();
    let mut full = false;
    let mut probe_forms = false;
    let mut delay_ms = 200;
    let mut timeout_secs = 10;
    let mut note = None;

    let mut tokens = Vec::new();
    let mut cur_token = String::new();
    let mut in_quotes = false;
    for ch in raw_args.chars() {
        if ch == '"' || ch == '\'' {
            in_quotes = !in_quotes;
        } else if ch.is_whitespace() && !in_quotes {
            if !cur_token.is_empty() {
                tokens.push(cur_token);
                cur_token = String::new();
            }
        } else {
            cur_token.push(ch);
        }
    }
    if !cur_token.is_empty() {
        tokens.push(cur_token);
    }

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        if tok == "--full" {
            full = true;
        } else if tok == "--forms" || tok == "--probe-forms" {
            probe_forms = true;
        } else if tok == "--delay" {
            i += 1;
            if i < tokens.len() {
                if let Ok(d) = tokens[i].parse::<u64>() {
                    delay_ms = d;
                }
            }
        } else if tok == "--timeout" {
            i += 1;
            if i < tokens.len() {
                if let Ok(t) = tokens[i].parse::<u64>() {
                    timeout_secs = t;
                }
            }
        } else if tok == "--note" {
            i += 1;
            if i < tokens.len() {
                note = Some(tokens[i].clone());
            }
        } else if !tok.starts_with("--") && url.is_empty() {
            url = tok.clone();
        }
        i += 1;
    }

    if url.is_empty() {
        return Err("Usage: :scan <url> [--full] [--forms] [--note \"...\"]".to_string());
    }

    // Prepend https:// if protocol missing
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{}", url);
    }

    let opts = webscan::ScanOptions {
        full,
        probe_forms,
        delay_ms,
        timeout_secs,
        note,
    };

    Ok((url, opts))
}
