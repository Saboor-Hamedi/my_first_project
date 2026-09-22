//! Spaced repetition SM-2 flashcard queries and review tracking.

use super::Database;
use crate::models::Card;
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::params;

impl Database {
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
}
