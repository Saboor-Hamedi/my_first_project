//! Note CRUD operations and query methods.

use super::Database;
use crate::models::Note;
use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::params;

impl Database {
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

    pub fn get_note(&self, id: i64) -> Result<Option<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic, body, struggled_with, created_at FROM notes WHERE id = ?1",
        )?;
        let mut iter = stmt.query_map(params![id], |row| {
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
        if let Some(n) = iter.next() {
            Ok(Some(n?))
        } else {
            Ok(None)
        }
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
}
