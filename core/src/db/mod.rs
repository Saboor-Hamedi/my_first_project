//! SQLite database connection, models, migrations, and query interfaces.

pub mod activity;
pub mod cards;
pub mod connection;
pub mod notes;

use rusqlite::Connection;

pub struct Database {
    pub(crate) conn: Connection,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
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

        // 5. Test Backup
        let tmp_backup_dir = std::env::temp_dir().join("mindforge_test_backup");
        let backup_file = db.backup(&tmp_backup_dir)?;
        assert!(backup_file.exists());
        let _ = std::fs::remove_file(&backup_file);
        let _ = std::fs::remove_dir(&tmp_backup_dir);

        Ok(())
    }
}
