use super::Database;
use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    pub id: i64,
    pub url: String,
    pub note: Option<String>,
    pub findings_json: String,
    pub scanned_at: String,
}

impl Database {
    /// Persists a successful scan result into the scans table.
    pub fn save_scan(&self, url: &str, note: Option<&str>, findings_json: &str) -> Result<i64> {
        let scanned_at = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO scans (url, note, findings_json, scanned_at) VALUES (?1, ?2, ?3, ?4)",
            params![url, note, findings_json, scanned_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Lists past scans in reverse chronological order.
    pub fn list_scans(&self) -> Result<Vec<ScanRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, note, findings_json, scanned_at FROM scans ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ScanRecord {
                id: row.get(0)?,
                url: row.get(1)?,
                note: row.get(2)?,
                findings_json: row.get(3)?,
                scanned_at: row.get(4)?,
            })
        })?;

        let mut scans = Vec::new();
        for r in rows {
            scans.push(r?);
        }
        Ok(scans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_list_scans() {
        let db = Database::open_in_memory().unwrap();
        let id = db.save_scan("https://example.com", Some("Test note"), "[]").unwrap();
        assert!(id > 0);

        let list = db.list_scans().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].url, "https://example.com");
        assert_eq!(list[0].note.as_deref(), Some("Test note"));
        assert_eq!(list[0].findings_json, "[]");
    }
}
