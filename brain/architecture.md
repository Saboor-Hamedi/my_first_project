# Architecture & Core System Design

MindForge is structured as a modular Rust workspace consisting of two primary crates:
- `core/`: Headless business logic, database engine, algorithms, and models.
- `app/`: Modern desktop GUI layer powered by `eframe` and `egui`.

```
my_first_project/
├── Cargo.toml          (Workspace root manifest)
├── core/               (Headless Core Engine)
│   ├── src/
│   │   ├── db.rs       (SQLite operations, schema migrations, backup logic)
│   │   ├── sm2.rs      (SuperMemo-2 spaced repetition algorithm)
│   │   ├── calibration.rs (Brier score & probabilistic calibration)
│   │   └── lib.rs      (Core export interface)
│   └── Cargo.toml
└── app/                (Desktop Application GUI)
    ├── src/
    │   ├── main.rs         (Entry point & eframe native window configuration)
    │   ├── app.rs          (Main App state, frame loop, and coordinator)
    │   ├── editor.rs       (Char buffer, cursor, selection, visual lines, undo/redo)
    │   ├── hybrid/mod.rs   (Hybrid Mode engine: auto-pairing, line duplicate)
    │   ├── vim/mod.rs      (Vim Modal engine: Normal, Insert, Visual, V-Line)
    │   ├── db_worker.rs    (Asynchronous database background worker)
    │   ├── input.rs        (Global shortcut router and event handler)
    │   ├── modals.rs       (Centered dialogs: Search, Rename, Delete confirmation)
    │   ├── sidebar.rs      (Notes drawer & list navigation)
    │   ├── bottom_bar.rs   (Status dock, :CMD dispatcher, word statistics, mode badge)
    │   ├── settingpanel.rs (Drawer panel for theme, sound, caret, backup, mode)
    │   ├── theme.rs        (Color palettes: Obsidian, Nord, Dracula, Solarized)
    │   ├── caret.rs        (Spring-physics cursor animation & shapes)
    │   └── sound.rs        (Auditory feedback engine: Mechanical, Typewriter, Soft)
    └── Cargo.toml
```

---

## 🧵 Asynchronous Database Architecture (`DbWorker`)

To prevent any UI stutter or frame drops during file I/O or SQLite queries, all database writes and heavy operations are dispatched asynchronously:

1. The UI thread communicates with `DbWorker` through a non-blocking `crossbeam_channel::Sender<DbMsg>`.
2. `DbMsg` variants include:
   - `SaveNote { id, topic, body, note_type }`
   - `DeleteNote { id }`
   - `RenameNote { id, new_topic }`
   - `SaveSetting { key, val }`
   - `LogActivity { date, chars_typed, notes_created, time_spent_secs }`
   - `Backup { target_dir }`
3. The background thread executes queries with WAL (Write-Ahead Logging) enabled on SQLite, ensuring zero UI latency.

---

## 🖥️ UI Layout & Design Language

MindForge embraces a modern, distraction-free aesthetic inspired by obsidian minimalism:
- **Frameless Window**: Custom drag regions with rounded corners (5px).
- **Collapsible Sidebar (`Ctrl+B`)**: Slides smoothly to reveal notes, tags, and document counts.
- **Bottom Status Dock**:
  - Displays mode badge (`NORMAL`, `INSERT`, `VISUAL`, `V-LINE`, or `HYBRID`).
  - Temporary notification toast area (fade out after 3 seconds).
  - Monospaced line number, column index, and live document word count.
- **Centered Modal Dialogs**:
  - **Fuzzy Search (`Ctrl+P`)**: Substring/fuzzy scoring across all note titles and body snippets.
  - **Rename Modal (`Ctrl+R`)**: Quick inline renaming with Enter confirmation.
  - **Delete Note Modal (`Ctrl+Shift+D`)**: High-contrast, safe destructive confirmation box preventing accidental loss.
