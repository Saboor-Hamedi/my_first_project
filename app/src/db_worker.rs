//! Asynchronous background database worker thread and messages.

use chrono::{Local, NaiveDate};
use core::Database;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[allow(dead_code)]
pub enum DbMsg {
    AddCard {
        prompt: String,
        answer: String,
        tag: Option<String>,
        today: NaiveDate,
    },
    RecordReview {
        card_id: i64,
        quality: u8,
        typed: String,
        wpm: f32,
    },
    UpdateCardSm2 {
        id: i64,
        ease: f32,
        interval: u32,
        reps: u32,
        due: NaiveDate,
    },
    SaveSetting {
        key: String,
        val: String,
    },
    SaveFocus {
        t1: String,
        t2: String,
    },
    SaveNote {
        topic: String,
        body: String,
        struggled: Option<String>,
    },
    UpdateNote {
        id: i64,
        body: String,
    },
    RenameNote {
        id: i64,
        new_topic: String,
    },
    DeleteNote {
        id: i64,
    },
    AddDecision {
        decision: String,
        reasoning: String,
        prediction: String,
        confidence: u8,
        review_on: NaiveDate,
    },
    ResolveDecision {
        id: i64,
        outcome: i32,
        lessons: String,
    },
    FlushActivity {
        date: String,
        delta_secs: u32,
        delta_keys: u32,
        delta_words: u32,
        delta_created: u32,
        delta_edited: u32,
    },
}

/// Spawns the background database worker thread.
/// UI thread communicates strictly via channels, never blocking on disk operations.
pub fn spawn_db_worker() -> Sender<DbMsg> {
    let (tx, rx): (Sender<DbMsg>, Receiver<DbMsg>) = channel();
    thread::spawn(move || {
        if let Ok(db) = Database::open_default() {
            while let Ok(msg) = rx.recv() {
                let now = Local::now().naive_local();
                match msg {
                    DbMsg::AddCard {
                        prompt,
                        answer,
                        tag,
                        today,
                    } => {
                        let _ = db.add_card(&prompt, &answer, tag.as_deref(), today);
                    }
                    DbMsg::RecordReview {
                        card_id,
                        quality,
                        typed,
                        wpm,
                    } => {
                        let _ = db.record_review(card_id, quality, &typed, wpm, now);
                    }
                    DbMsg::UpdateCardSm2 {
                        id,
                        ease,
                        interval,
                        reps,
                        due,
                    } => {
                        let _ = db.update_card_sm2(id, ease, interval, reps, due);
                    }
                    DbMsg::SaveSetting { key, val } => {
                        let _ = db.set_setting(&key, &val);
                    }
                    DbMsg::SaveFocus { t1, t2 } => {
                        let _ = db.set_focus(&t1, &t2);
                    }
                    DbMsg::SaveNote {
                        topic,
                        body,
                        struggled,
                    } => {
                        let _ = db.add_note(&topic, &body, struggled.as_deref(), now);
                    }
                    DbMsg::UpdateNote { id, body } => {
                        let _ = db.update_note(id, &body);
                    }
                    DbMsg::RenameNote { id, new_topic } => {
                        let _ = db.rename_note(id, &new_topic);
                    }
                    DbMsg::DeleteNote { id } => {
                        let _ = db.delete_note(id);
                    }
                    DbMsg::AddDecision {
                        decision,
                        reasoning,
                        prediction,
                        confidence,
                        review_on,
                    } => {
                        let _ = db.add_decision(
                            &decision,
                            &reasoning,
                            &prediction,
                            confidence,
                            review_on,
                            now,
                        );
                    }
                    DbMsg::ResolveDecision {
                        id,
                        outcome,
                        lessons,
                    } => {
                        let _ = db.resolve_decision(id, outcome, &lessons);
                    }
                    DbMsg::FlushActivity {
                        date,
                        delta_secs,
                        delta_keys,
                        delta_words,
                        delta_created,
                        delta_edited,
                    } => {
                        let _ = db.record_daily_activity(
                            &date,
                            delta_secs,
                            delta_keys,
                            delta_words,
                            delta_created,
                            delta_edited,
                        );
                    }
                }
            }
        }
    });
    tx
}
