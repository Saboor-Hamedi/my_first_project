//! Asynchronous background worker for reading, parsing, and batch inserting notes.

use super::scanner::{scan_workspace_paths, ScannedFile};
use super::state::{ImportStats, ImportStatus};
use chrono::Local;
use core::Database;
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

/// Spawns a background thread to scan and batch-insert notes without locking the main thread.
pub fn spawn_import_worker(
    paths: Vec<PathBuf>,
    stats: Arc<RwLock<ImportStats>>,
    status: Arc<RwLock<ImportStatus>>,
    cancel_token: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    ctx: egui::Context,
) {
    is_running.store(true, Ordering::SeqCst);
    cancel_token.store(false, Ordering::SeqCst);

    // Initial status: Scanning
    if let Ok(mut st) = status.write() {
        *st = ImportStatus::Scanning;
    }
    if let Ok(mut s) = stats.write() {
        *s = ImportStats::default();
    }
    ctx.request_repaint();

    thread::spawn(move || {
        // Step 1: Scan all files
        let (files, total_bytes) = scan_workspace_paths(&paths, &cancel_token);

        if cancel_token.load(Ordering::Relaxed) {
            finish_cancelled(0, &status, &is_running, &ctx);
            return;
        }

        let total_count = files.len();
        if let Ok(mut s) = stats.write() {
            s.total_files = total_count;
            s.remaining_files = total_count;
            s.total_bytes = total_bytes;
        }

        if total_count == 0 {
            if let Ok(mut st) = status.write() {
                *st = ImportStatus::Completed {
                    count: 0,
                    total_bytes: 0,
                };
            }
            is_running.store(false, Ordering::SeqCst);
            ctx.request_repaint();
            return;
        }

        // Step 2: Open SQLite database connection
        let mut db = match Database::open_default() {
            Ok(db) => db,
            Err(e) => {
                if let Ok(mut st) = status.write() {
                    *st = ImportStatus::Error(format!("Database error: {}", e));
                }
                is_running.store(false, Ordering::SeqCst);
                ctx.request_repaint();
                return;
            }
        };

        // Step 3: Batch import files in transactions of 100 notes
        let batch_size = 100;
        let mut inserted_count = 0usize;
        let mut inserted_bytes = 0u64;

        for chunk in files.chunks(batch_size) {
            if cancel_token.load(Ordering::Relaxed) {
                finish_cancelled(inserted_count, &status, &is_running, &ctx);
                return;
            }

            let mut batch_records = Vec::with_capacity(chunk.len());
            let mut chunk_bytes = 0u64;
            let now = Local::now().naive_local();

            for ScannedFile { path, size, relative_title } in chunk {
                if cancel_token.load(Ordering::Relaxed) {
                    finish_cancelled(inserted_count, &status, &is_running, &ctx);
                    return;
                }

                // Read file content
                let content = fs::read_to_string(path).unwrap_or_else(|_| {
                    // Fallback to lossy reading if file contains non-UTF-8 characters
                    fs::read(path)
                        .map(|b| String::from_utf8_lossy(&b).to_string())
                        .unwrap_or_default()
                });

                // Extract title: prefer first Markdown heading if available, otherwise file stem
                let title = extract_title(&content, relative_title);
                batch_records.push((title, content, None, now));
                chunk_bytes += *size;
            }

            // Update current active file indicator in status
            if let Some(first) = chunk.first() {
                if let Ok(mut st) = status.write() {
                    *st = ImportStatus::Importing {
                        current_file: first.relative_title.clone(),
                    };
                }
            }

            // Commit batch to SQLite inside a single atomic transaction
            match db.add_notes_batch(&batch_records) {
                Ok(count) => {
                    inserted_count += count;
                    inserted_bytes += chunk_bytes;

                    if let Ok(mut s) = stats.write() {
                        s.inserted_files = inserted_count;
                        s.remaining_files = total_count.saturating_sub(inserted_count);
                        s.inserted_bytes = inserted_bytes;
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    if let Ok(mut st) = status.write() {
                        *st = ImportStatus::Error(format!("Batch insert failed: {}", e));
                    }
                    is_running.store(false, Ordering::SeqCst);
                    ctx.request_repaint();
                    return;
                }
            }
        }

        // Step 4: Mark completed
        if let Ok(mut st) = status.write() {
            *st = ImportStatus::Completed {
                count: inserted_count,
                total_bytes: inserted_bytes,
            };
        }
        if let Ok(mut s) = stats.write() {
            s.inserted_files = inserted_count;
            s.remaining_files = 0;
            s.inserted_bytes = inserted_bytes;
        }

        is_running.store(false, Ordering::SeqCst);
        ctx.request_repaint();
    });
}

fn finish_cancelled(
    count: usize,
    status: &Arc<RwLock<ImportStatus>>,
    is_running: &Arc<AtomicBool>,
    ctx: &egui::Context,
) {
    if let Ok(mut st) = status.write() {
        *st = ImportStatus::Cancelled { count };
    }
    is_running.store(false, Ordering::SeqCst);
    ctx.request_repaint();
}

/// Extracts a clean note topic/title:
/// Checks for the first `# Heading` in markdown content. If found, uses it.
/// Otherwise defaults to the clean file stem name.
pub fn extract_title(content: &str, fallback: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            let heading = trimmed[2..].trim();
            if !heading.is_empty() {
                return heading.to_string();
            }
        }
    }
    fallback.to_string()
}
