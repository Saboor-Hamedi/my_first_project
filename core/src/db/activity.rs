//! Daily activity counters, lifetime stats, decisions calibration, and settings persistence.

use super::Database;
use crate::models::{DailyActivity, Decision, FocusState};
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::params;

impl Database {
    // --- DECISIONS ---

    pub fn add_decision(
        &self,
        decision: &str,
        reasoning: &str,
        prediction: &str,
        confidence: u8,
        review_on: NaiveDate,
        created_at: NaiveDateTime,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO decisions (decision, reasoning, prediction, confidence, review_on, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                decision,
                reasoning,
                prediction,
                confidence,
                review_on.format("%Y-%m-%d").to_string(),
                created_at.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_pending_decisions(&self, today: NaiveDate) -> Result<Vec<Decision>> {
        let today_str = today.format("%Y-%m-%d").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, decision, reasoning, prediction, confidence, review_on, outcome, lessons, created_at
             FROM decisions WHERE outcome IS NULL AND review_on <= ?1 ORDER BY review_on ASC",
        )?;
        let iter = stmt.query_map(params![today_str], |row| {
            let r_str: String = row.get(5)?;
            let c_str: String = row.get(8)?;
            let review_on = NaiveDate::parse_from_str(&r_str, "%Y-%m-%d").unwrap_or(today);
            let created_at =
                NaiveDateTime::parse_from_str(&c_str, "%Y-%m-%d %H:%M:%S").unwrap_or_default();
            Ok(Decision {
                id: row.get(0)?,
                decision: row.get(1)?,
                reasoning: row.get(2)?,
                prediction: row.get(3)?,
                confidence: row.get(4)?,
                review_on,
                outcome: row.get(6)?,
                lessons: row.get(7)?,
                created_at,
            })
        })?;
        let mut list = Vec::new();
        for d in iter {
            list.push(d?);
        }
        Ok(list)
    }

    pub fn resolve_decision(&self, id: i64, outcome: i32, lessons: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE decisions SET outcome = ?1, lessons = ?2 WHERE id = ?3",
            params![outcome, lessons, id],
        )?;
        Ok(())
    }

    pub fn get_resolved_decisions_for_calibration(&self) -> Result<Vec<(u8, i32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT confidence, outcome FROM decisions WHERE outcome IS NOT NULL",
        )?;
        let iter = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut list = Vec::new();
        for item in iter {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn get_all_decisions(&self) -> Result<Vec<Decision>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, decision, reasoning, prediction, confidence, review_on, outcome, lessons, created_at
             FROM decisions ORDER BY id DESC",
        )?;
        let iter = stmt.query_map([], |row| {
            let r_str: String = row.get(5)?;
            let c_str: String = row.get(8)?;
            let review_on = NaiveDate::parse_from_str(&r_str, "%Y-%m-%d").unwrap_or_default();
            let created_at =
                NaiveDateTime::parse_from_str(&c_str, "%Y-%m-%d %H:%M:%S").unwrap_or_default();
            Ok(Decision {
                id: row.get(0)?,
                decision: row.get(1)?,
                reasoning: row.get(2)?,
                prediction: row.get(3)?,
                confidence: row.get(4)?,
                review_on,
                outcome: row.get(6)?,
                lessons: row.get(7)?,
                created_at,
            })
        })?;
        let mut list = Vec::new();
        for d in iter {
            list.push(d?);
        }
        Ok(list)
    }

    // --- FOCUS ---

    pub fn get_focus(&self) -> Result<FocusState> {
        let mut stmt = self.conn.prepare("SELECT topic1, topic2 FROM focus WHERE id = 1")?;
        let focus = stmt.query_row([], |r| {
            Ok(FocusState {
                topic1: r.get(0)?,
                topic2: r.get(1)?,
            })
        })?;
        Ok(focus)
    }

    pub fn set_focus(&self, topic1: &str, topic2: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE focus SET topic1 = ?1, topic2 = ?2 WHERE id = 1",
            params![topic1, topic2],
        )?;
        Ok(())
    }

    // --- SETTINGS ---

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut iter = stmt.query_map(params![key], |r| r.get(0))?;
        if let Some(val) = iter.next() {
            Ok(Some(val?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // --- DAILY ACTIVITY ---

    pub fn record_daily_activity(
        &self,
        date: &str,
        delta_secs: u32,
        delta_keys: u32,
        delta_words: u32,
        delta_created: u32,
        delta_edited: u32,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO daily_activity (date, active_seconds, keystrokes, words_written, notes_created, notes_edited)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(date) DO UPDATE SET
                 active_seconds = active_seconds + excluded.active_seconds,
                 keystrokes = keystrokes + excluded.keystrokes,
                 words_written = words_written + excluded.words_written,
                 notes_created = notes_created + excluded.notes_created,
                 notes_edited = notes_edited + excluded.notes_edited",
            params![date, delta_secs, delta_keys, delta_words, delta_created, delta_edited],
        )?;
        Ok(())
    }

    pub fn get_recent_activity(&self, days: usize) -> Result<Vec<DailyActivity>> {
        let mut stmt = self.conn.prepare(
            "SELECT date, active_seconds, keystrokes, words_written, notes_created, notes_edited
             FROM daily_activity
             ORDER BY date DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![days], |r| {
            Ok(DailyActivity {
                date: r.get(0)?,
                active_seconds: r.get(1)?,
                keystrokes: r.get(2)?,
                words_written: r.get(3)?,
                notes_created: r.get(4)?,
                notes_edited: r.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_lifetime_activity(&self) -> Result<(u64, u64, u64, usize)> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(active_seconds), 0),
                    COALESCE(SUM(keystrokes), 0),
                    COALESCE(SUM(words_written), 0),
                    COUNT(*)
             FROM daily_activity",
        )?;
        let res = stmt.query_row([], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, i64>(1)? as u64,
                r.get::<_, i64>(2)? as u64,
                r.get::<_, usize>(3)?,
            ))
        })?;
        Ok(res)
    }
}
