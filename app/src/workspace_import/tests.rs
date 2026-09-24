//! Comprehensive test suite for workspace & Obsidian vault import.

#[cfg(test)]
mod tests {
    use crate::workspace_import::scanner::scan_workspace_paths;
    use crate::workspace_import::state::{ImportStats, ImportStatus, WorkspaceImporter};
    use crate::workspace_import::worker::extract_title;
    use chrono::Local;
    use core::Database;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    #[test]
    fn test_format_bytes() {
        assert_eq!(ImportStats::format_bytes(512), "512 B");
        assert_eq!(ImportStats::format_bytes(2048), "2.0 KB");
        assert_eq!(ImportStats::format_bytes(1024 * 1024 * 5), "5.0 MB");
        assert_eq!(ImportStats::format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    #[test]
    fn test_title_extraction() {
        // First heading extraction
        let md_with_h1 = "# Project Architecture\n\nSome body text here.";
        assert_eq!(extract_title(md_with_h1, "architecture"), "Project Architecture");

        // Heading with leading whitespace
        let md_with_spaced_h1 = "\n\n  # Clean Title  \n\nNotes";
        assert_eq!(extract_title(md_with_spaced_h1, "fallback"), "Clean Title");

        // No heading defaults to fallback filename
        let md_no_h1 = "Just regular text with no heading.";
        assert_eq!(extract_title(md_no_h1, "My Fallback Note"), "My Fallback Note");

        // Obsidian frontmatter before heading
        let md_frontmatter = "---\ntags: [rust, dev]\n---\n# Frontmatter Title\nBody";
        assert_eq!(extract_title(md_frontmatter, "frontmatter"), "Frontmatter Title");
    }

    #[test]
    fn test_scan_filtering_and_silently_ignoring_non_markdown() {
        let temp_dir = std::env::temp_dir().join(format!("mindforge_test_vault_{}", fastrand::u64(..)));
        let _ = fs::create_dir_all(&temp_dir);

        // Nested folders
        let sub_folder = temp_dir.join("Nested").join("Subfolder");
        let obsidian_folder = temp_dir.join(".obsidian");
        let _ = fs::create_dir_all(&sub_folder);
        let _ = fs::create_dir_all(&obsidian_folder);

        // Allowed files
        let md_file1 = temp_dir.join("Note1.md");
        let md_file2 = sub_folder.join("Note2.markdown");
        let txt_file = sub_folder.join("Readme.txt");
        let _ = fs::write(&md_file1, "# First Note\nContent 1");
        let _ = fs::write(&md_file2, "Content 2 without heading");
        let _ = fs::write(&txt_file, "Plain text document");

        // Disallowed files (MUST BE SILENTLY IGNORED)
        let pdf_file = temp_dir.join("Document.pdf");
        let png_file = sub_folder.join("Diagram.png");
        let canvas_file = temp_dir.join("Mindmap.canvas");
        let json_file = temp_dir.join("data.json");
        let obsidian_config = obsidian_folder.join("app.json");
        let obsidian_hidden_md = obsidian_folder.join("workspace.md"); // inside .obsidian

        let _ = fs::write(&pdf_file, b"%PDF-1.4 dummy binary");
        let _ = fs::write(&png_file, b"\x89PNG dummy image");
        let _ = fs::write(&canvas_file, "{\"nodes\": []}");
        let _ = fs::write(&json_file, "{\"version\": 1}");
        let _ = fs::write(&obsidian_config, "{\"theme\": \"dark\"}");
        let _ = fs::write(&obsidian_hidden_md, "# Should be ignored");

        let cancel_token = Arc::new(AtomicBool::new(false));
        let (scanned, total_bytes) = scan_workspace_paths(&[temp_dir.clone()], &cancel_token);

        // Clean up temp dir
        let _ = fs::remove_dir_all(&temp_dir);

        // Exactly 3 allowed files: Note1.md, Note2.markdown, Readme.txt
        assert_eq!(scanned.len(), 3);
        assert!(total_bytes > 0);

        let titles: Vec<String> = scanned.iter().map(|f| f.relative_title.clone()).collect();
        assert!(titles.contains(&"Note1".to_string()));
        assert!(titles.contains(&"Note2".to_string()));
        assert!(titles.contains(&"Readme".to_string()));

        // Ensure disallowed files were never included
        assert!(!titles.contains(&"Document".to_string()));
        assert!(!titles.contains(&"Diagram".to_string()));
        assert!(!titles.contains(&"Mindmap".to_string()));
        assert!(!titles.contains(&"workspace".to_string()));
    }

    #[test]
    fn test_batch_database_insertion() {
        let mut db = Database::open_in_memory().expect("In-memory DB should open");
        let now = Local::now().naive_local();

        let batch = vec![
            ("Note Alpha".to_string(), "Body of alpha".to_string(), None, now),
            ("Note Beta".to_string(), "Body of beta".to_string(), Some("tag".to_string()), now),
            ("Note Gamma".to_string(), "Body of gamma".to_string(), None, now),
        ];

        let inserted = db.add_notes_batch(&batch).expect("Batch insert should succeed");
        assert_eq!(inserted, 3);

        let count = db.get_notes_count().expect("Notes count should succeed");
        assert_eq!(count, 3);

        let recent = db.get_recent_notes(10).expect("Recent notes should succeed");
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_importer_lifecycle_and_poll_completion() {
        let mut importer = WorkspaceImporter::new();
        assert!(!importer.is_active());
        assert!(!importer.is_modal_open);
        assert_eq!(importer.poll_completion(), None);

        // Open modal
        importer.open_modal();
        assert!(importer.is_modal_open);

        // Close modal (should close without cancelling)
        importer.close_modal();
        assert!(!importer.is_modal_open);

        // Simulate background worker completing
        if let Ok(mut st) = importer.status.write() {
            *st = ImportStatus::Completed { count: 42, total_bytes: 10240 };
        }

        // poll_completion should return Some(42) once
        assert_eq!(importer.poll_completion(), Some(42));
        // Subsequent poll should return None
        assert_eq!(importer.poll_completion(), None);
    }
}
