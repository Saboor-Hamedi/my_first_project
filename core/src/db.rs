use crate::models::{Card, DailyActivity, Decision, FocusState, Note};
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens the default database file located in the user's data directory.
    pub fn open_default() -> Result<Self> {
        let path = Self::get_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::open(&path)
    }

    /// Resolves the default database path on disk.
    pub fn get_db_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "mindforge", "mindforge") {
            Ok(proj_dirs.data_local_dir().join("mindforge.db"))
        } else {
            Ok(PathBuf::from("mindforge.db"))
        }
    }

    /// Opens SQLite database at a specific path and applies schema migrations.
    pub fn open(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", path))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Opens an in-memory SQLite database (ideal for unit testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Auto-migrates tables if they don't exist.
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS cards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt TEXT NOT NULL,
                answer TEXT NOT NULL,
                tag TEXT,
                ease REAL NOT NULL DEFAULT 2.5,
                interval_days INTEGER NOT NULL DEFAULT 0,
                reps INTEGER NOT NULL DEFAULT 0,
                due TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS reviews (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                card_id INTEGER NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
                quality INTEGER NOT NULL,
                typed_answer TEXT NOT NULL,
                wpm REAL NOT NULL,
                reviewed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                body TEXT NOT NULL,
                struggled_with TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                decision TEXT NOT NULL,
                reasoning TEXT NOT NULL,
                prediction TEXT NOT NULL,
                confidence INTEGER NOT NULL,
                review_on TEXT NOT NULL,
                outcome INTEGER,
                lessons TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS focus (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                topic1 TEXT NOT NULL DEFAULT '',
                topic2 TEXT NOT NULL DEFAULT ''
            );

            INSERT OR IGNORE INTO focus (id, topic1, topic2) VALUES (1, '', '');

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS daily_activity (
                date TEXT PRIMARY KEY,
                active_seconds INTEGER NOT NULL DEFAULT 0,
                keystrokes INTEGER NOT NULL DEFAULT 0,
                words_written INTEGER NOT NULL DEFAULT 0,
                notes_created INTEGER NOT NULL DEFAULT 0,
                notes_edited INTEGER NOT NULL DEFAULT 0
            );
            ",
        )?;
        Ok(())
    }

    // --- CARDS ---

    pub fn add_card(
        &self,
        prompt: &str,
        answer: &str,
        tag: Option<&str>,
        today: NaiveDate,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO cards (prompt, answer, tag, ease, interval_days, reps, due)
             VALUES (?1, ?2, ?3, 2.5, 0, 0, ?4)",
            params![prompt, answer, tag, today.format("%Y-%m-%d").to_string()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_due_cards(&self, today: NaiveDate) -> Result<Vec<Card>> {
        let today_str = today.format("%Y-%m-%d").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt, answer, tag, ease, interval_days, reps, due
             FROM cards WHERE due <= ?1 ORDER BY id ASC",
        )?;
        let card_iter = stmt.query_map(params![today_str], |row| {
            let due_str: String = row.get(7)?;
            let due = NaiveDate::parse_from_str(&due_str, "%Y-%m-%d")
                .unwrap_or(today);
            Ok(Card {
                id: row.get(0)?,
                prompt: row.get(1)?,
                answer: row.get(2)?,
                tag: row.get(3)?,
                ease: row.get(4)?,
                interval_days: row.get(5)?,
                reps: row.get(6)?,
                due,
            })
        })?;

        let mut list = Vec::new();
        for c in card_iter {
            list.push(c?);
        }
        Ok(list)
    }

    pub fn get_all_cards(&self) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt, answer, tag, ease, interval_days, reps, due
             FROM cards ORDER BY id ASC",
        )?;
        let card_iter = stmt.query_map([], |row| {
            let due_str: String = row.get(7)?;
            let due = NaiveDate::parse_from_str(&due_str, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Local::now().date_naive());
            Ok(Card {
                id: row.get(0)?,
                prompt: row.get(1)?,
                answer: row.get(2)?,
                tag: row.get(3)?,
                ease: row.get(4)?,
                interval_days: row.get(5)?,
                reps: row.get(6)?,
                due,
            })
        })?;

        let mut list = Vec::new();
        for c in card_iter {
            list.push(c?);
        }
        Ok(list)
    }

    pub fn update_card_sm2(
        &self,
        id: i64,
        ease: f32,
        interval_days: u32,
        reps: u32,
        due: NaiveDate,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE cards SET ease = ?1, interval_days = ?2, reps = ?3, due = ?4 WHERE id = ?5",
            params![
                ease,
                interval_days,
                reps,
                due.format("%Y-%m-%d").to_string(),
                id
            ],
        )?;
        Ok(())
    }

    pub fn record_review(
        &self,
        card_id: i64,
        quality: u8,
        typed_answer: &str,
        wpm: f32,
        reviewed_at: NaiveDateTime,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO reviews (card_id, quality, typed_answer, wpm, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                card_id,
                quality,
                typed_answer,
                wpm,
                reviewed_at.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )?;
        Ok(())
    }

    pub fn get_total_cards_count(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))?;
        Ok(count)
    }

    pub fn get_reviews_per_day(&self, days: usize) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT substr(reviewed_at, 1, 10) as day, count(*)
             FROM reviews
             GROUP BY day
             ORDER BY day DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![days], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        results.reverse();
        Ok(results)
    }

    // --- NOTES / EXPLAIN ---

    pub fn add_note(
        &self,
        topic: &str,
        body: &str,
        struggled_with: Option<&str>,
        created_at: NaiveDateTime,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (topic, body, struggled_with, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                topic,
                body,
                struggled_with,
                created_at.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_notes_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_recent_notes(&self, limit: usize) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic, body, struggled_with, created_at FROM notes ORDER BY id DESC LIMIT ?1",
        )?;
        let iter = stmt.query_map(params![limit as i64], |row| {
            let dt_str: String = row.get(4)?;
            let created_at =
                NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S").unwrap_or_default();
            Ok(Note {
                id: row.get(0)?,
                topic: row.get(1)?,
                body: row.get(2)?,
                struggled_with: row.get(3)?,
                created_at,
            })
        })?;
        let mut list = Vec::new();
        for n in iter {
            list.push(n?);
        }
        Ok(list)
    }

    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic, body, struggled_with, created_at FROM notes ORDER BY id DESC",
        )?;
        let iter = stmt.query_map([], |row| {
            let dt_str: String = row.get(4)?;
            let created_at =
                NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S").unwrap_or_default();
            Ok(Note {
                id: row.get(0)?,
                topic: row.get(1)?,
                body: row.get(2)?,
                struggled_with: row.get(3)?,
                created_at,
            })
        })?;
        let mut list = Vec::new();
        for n in iter {
            list.push(n?);
        }
        Ok(list)
    }

    pub fn update_note(&self, id: i64, body: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE notes SET body = ?1 WHERE id = ?2",
            params![body, id],
        )?;
        Ok(())
    }

    pub fn rename_note(&self, id: i64, new_topic: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE notes SET topic = ?1 WHERE id = ?2",
            params![new_topic, id],
        )?;
        Ok(())
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_db_migrations_and_operations() -> Result<()> {
        let db = Database::open_in_memory()?;
        let today = Local::now().date_naive();

        // 1. Add card
        let card_id = db.add_card("What is Rust?", "A systems language", Some("rust"), today)?;
        assert!(card_id > 0);

        let due_cards = db.get_due_cards(today)?;
        assert_eq!(due_cards.len(), 1);
        assert_eq!(due_cards[0].prompt, "What is Rust?");

        // 2. Settings
        db.set_setting("caret", "fire")?;
        let val = db.get_setting("caret")?;
        assert_eq!(val, Some("fire".to_string()));

        // 3. Focus
        db.set_focus("Rust", "Linear Algebra")?;
        let focus = db.get_focus()?;
        assert_eq!(focus.topic1, "Rust");
        assert_eq!(focus.topic2, "Linear Algebra");

        // 4. Daily Activity
        db.record_daily_activity("2026-09-21", 120, 500, 80, 1, 2)?;
        db.record_daily_activity("2026-09-21", 60, 200, 30, 0, 1)?;
        let recents = db.get_recent_activity(7)?;
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].active_seconds, 180);
        assert_eq!(recents[0].keystrokes, 700);
        assert_eq!(recents[0].words_written, 110);
        assert_eq!(recents[0].notes_created, 1);
        assert_eq!(recents[0].notes_edited, 3);

        let (total_s, total_k, total_w, days) = db.get_lifetime_activity()?;
        assert_eq!(total_s, 180);
        assert_eq!(total_k, 700);
        assert_eq!(total_w, 110);
        assert_eq!(days, 1);

        Ok(())
    }
}
