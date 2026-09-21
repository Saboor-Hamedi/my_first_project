# Database & Backup Architecture

MindForge prioritizes local-first data ownership and durability. All user notes, settings, and activity logs are stored locally in an embedded SQLite database, with automated and manual backup capabilities into dedicated directories.

---

## 🗄️ Database Schema (`core/src/db.rs`)

### 1. `notes`
The primary table storing user documents:
- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `topic`: TEXT NOT NULL (Note title)
- `body`: TEXT NOT NULL (Markdown or plain text content)
- `created_at`: INTEGER NOT NULL (Unix timestamp)
- `updated_at`: INTEGER NOT NULL (Unix timestamp)
- `note_type`: TEXT NOT NULL DEFAULT 'standard'
- `tags`: TEXT (Optional comma-separated tags)

### 2. `settings`
Key-value configuration store preserving user customizations across sessions:
- `key`: TEXT PRIMARY KEY
- `val`: TEXT NOT NULL

Common persisted keys:
- `editor_mode`: `"hybrid"` | `"vim"`
- `theme`: `"obsidian"` | `"nord"` | `"dracula"` | `"solarized"`
- `sound_profile`: `"mechanical"` | `"typewriter"` | `"soft"` | `"silent"`
- `caret_kind`: `"block"` | `"beam"` | `"underline"` | `"glow"`
- `backup_dir`: Custom directory path for automated database backups.

### 3. `daily_activity`
Tracks productivity metrics for review and stats visualizations:
- `date`: TEXT PRIMARY KEY (Format `YYYY-MM-DD`)
- `chars_typed`: INTEGER NOT NULL
- `notes_created`: INTEGER NOT NULL
- `time_spent_secs`: INTEGER NOT NULL

---

## 💾 Backup System (`mindforge_backup`)

### Dedicated Backup Directory
When a backup is triggered (via Settings > Backup, `:backup`, or automatic routines), MindForge creates a dedicated folder named `mindforge_backup` inside the target directory:

```
target_backup_directory/
└── mindforge_backup/
    ├── mindforge_backup_2026-09-22_12-30-00.db
    └── ...
```

### Triggering Backups
1. **Command Dock**: Run `:backup` in the bottom command bar.
2. **Settings Panel (`Ctrl+,`)**: Navigate to the **Backup** tab, review the destination directory, and click **Trigger Backup Now**.
3. **Status Feedback**: Success notifications display the exact time and folder path in the bottom bar and in Settings.

---

## 📤 Document Import & Export

In addition to full database backups, individual documents can be imported or exported:

### Exporting Notes
- Command: `:export` or `:export "my_document.md"`
- If no argument is provided, MindForge launches a native file save dialog with `.md` and `.txt` filters.
- Sanitizes file names to prevent invalid filesystem characters.

### Importing Notes
- Command: `:import [path_to_file]`
- Reads UTF-8 plain text or Markdown directly into the editor buffer.
