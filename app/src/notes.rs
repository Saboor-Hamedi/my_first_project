//! Synchronous note persistence, live search filtering, and note operations.

use crate::app::App;
use crate::fuzzy::{fuzzy_match, SearchItem, SearchResultKind};
use chrono::Local;
use core::Note;

/// Saves the active note directly to SQLite and updates in-memory notes_list immediately.
pub fn quick_save_active_note(app: &mut App, now: f64) {
    let content = app.ed.text();
    if let Some(ref db) = app.db {
        if let Some(id) = app.active_note_id {
            let _ = db.update_note(id, &content);
            app.is_dirty = false;
            app.last_saved_time = now;
            app.pending_edited += 1;
            if let Some(n) = app.notes_list.iter_mut().find(|n| n.id == id) {
                n.body = content.clone();
            }
            app.save_active_note_id();
            app.set_status("Saved", now);
        } else {
            let topic = if app.active_note_title.trim().is_empty() {
                "Untitled Note".to_string()
            } else {
                app.active_note_title.clone()
            };
            let dt = Local::now().naive_local();
            if let Ok(new_id) = db.add_note(&topic, &content, None, dt) {
                app.active_note_id = Some(new_id);
                app.save_active_note_id();
                app.is_dirty = false;
                app.last_saved_time = now;
                app.pending_created += 1;
                app.notes_list.insert(
                    0,
                    Note {
                        id: new_id,
                        topic: topic.clone(),
                        body: content,
                        struggled_with: None,
                        created_at: dt,
                    },
                );
                app.set_status("Saved", now);
            }
        }
    }
}

/// Deletes the active note from SQLite and in-memory notes_list.
pub fn delete_active_note(app: &mut App, now: f64) {
    if let Some(id) = app.active_note_id {
        if let Some(ref db) = app.db {
            let _ = db.delete_note(id);
        }
        let _ = app.db_tx.send(crate::db_worker::DbMsg::DeleteNote { id });
        app.notes_list.retain(|n| n.id != id);
        app.active_note_id = None;
        app.save_active_note_id();
        app.active_note_title = "Untitled Note".to_string();
        app.ed.clear();
        app.is_dirty = false;
        app.set_status("Deleted", now);
        app.reload_db_state();

        // Load the next available note if one exists
        if let Some(first) = app.notes_list.first() {
            let first_id = first.id;
            let topic = first.topic.clone();
            let body = first.body.clone();
            let clean = body.replace("\r\n", "\n").replace('\r', "\n");
            app.active_note_id = Some(first_id);
            app.save_active_note_id();
            app.active_note_title = topic;
            app.ed.set_text(&clean);
            app.ed.cur = 0;
            app.is_dirty = false;
        }
    } else {
        app.ed.clear();
        app.is_dirty = false;
        app.set_status("Cleared note", now);
    }
}

/// Renames the active note directly in SQLite and updates in-memory notes_list.
pub fn rename_active_note(app: &mut App, new_title: &str, now: f64) {
    let trimmed = new_title.trim().to_string();
    if trimmed.is_empty() {
        return;
    }
    app.active_note_title = trimmed.clone();
    if let Some(id) = app.active_note_id {
        if let Some(ref db) = app.db {
            let _ = db.rename_note(id, &trimmed);
        }
        if let Some(n) = app.notes_list.iter_mut().find(|n| n.id == id) {
            n.topic = trimmed.clone();
        }
        app.set_status("Renamed", now);
    }
}

/// Updates fuzzy search results across notes, flashcards, and decisions.
pub fn update_search_results(app: &mut App) {
    let query = app.search_query.trim();
    let mut results = Vec::new();

    for note in &app.notes_list {
        let score_topic = fuzzy_match(query, &note.topic);
        let score_body = fuzzy_match(query, &note.body);
        if let Some(score) = score_topic.or(score_body) {
            let snippet = if let Some(pos) = note.body.to_lowercase().find(&query.to_lowercase()) {
                let start = pos.saturating_sub(20);
                let end = (pos + query.len() + 40).min(note.body.len());
                format!("...{}...", note.body[start..end].replace('\n', " "))
            } else if note.body.len() > 60 {
                format!("{}...", &note.body[..60].replace('\n', " "))
            } else {
                note.body.replace('\n', " ")
            };
            results.push(SearchItem {
                kind: SearchResultKind::Document,
                id: note.id,
                title: note.topic.clone(),
                snippet,
                score,
            });
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    app.search_results = results;
    if app.search_selected >= app.search_results.len() {
        app.search_selected = 0;
    }
}
