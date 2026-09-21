# MindForge Brain & Knowledge Base

Welcome to the **MindForge Brain** documentation repository. This directory serves as the definitive source of truth and architectural reference for MindForge, ensuring that features, keystrokes, design decisions, and system mechanics are clearly preserved.

---

## 📚 Documentation Table of Contents

1. **[Shortcuts & Command Reference](file:///b:/rust/my_first_project/brain/shortcuts_and_commands.md)**
   - Complete cheat sheet for all global hotkeys, modal triggers, and command bar commands (`:w`, `:vim`, `:mode`, `:d`, `:export`, etc.).
   - Keybindings for both **Hybrid** and **Vim** editor modes.

2. **[Editor Engines & Typography](file:///b:/rust/my_first_project/brain/editor_engines.md)**
   - **Hybrid Mode**: Modern IDE ergonomics, smart auto-pairing, duplicate line (`Ctrl+D`), word boundary deletion, selection wrapping.
   - **Vim Engine**: Modal editing (`Normal`, `Insert`, `Visual`, `Visual Line`), caret shape morphing (`Block` vs `Beam`), visual line navigation and gap selection.
   - Buffer geometry, visual soft-wrapping, viewport frustum culling, and whitespace/gap selection highlights.

3. **[Architecture & Core System](file:///b:/rust/my_first_project/brain/architecture.md)**
   - High-level design: Separation between `core` (headless engine, SQLite, SM-2, calibration) and `app` (eframe/egui GUI).
   - Asynchronous worker architecture (`DbWorker`, crossbeam channels).
   - UI layout: Frameless Obsidian window, responsive sidebar, bottom dock, modal dialogs.

4. **[Database & Backup System](file:///b:/rust/my_first_project/brain/database_and_backup.md)**
   - SQLite tables: `notes`, `settings`, `daily_activity`.
   - Backup directory structure: Isolated `mindforge_backup/` directory with timestamped SQL dumps.
   - Markdown and plain text file import & export.

---

## ⚡ Quick Start & Common Tasks

- **Run Dev App**: `cargo run -p app`
- **Run All Tests**: `cargo test --workspace`
- **Check Compilation**: `cargo check --workspace`
