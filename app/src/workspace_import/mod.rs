//! Vault and workspace recursive importer module.
//!
//! Handles smooth, non-blocking background imports of Obsidian vaults and nested folders
//! containing `.md` and `.txt` documents directly into the SQLite database.

pub mod drag_drop;
pub mod import_ui;
pub mod scanner;
pub mod state;
pub mod worker;

pub use drag_drop::{handle_drag_and_drop, render_hover_indicator, start_workspace_import};
pub use import_ui::{render_import_modal, ImportModalAction};
pub use state::{ImportStats, ImportStatus, WorkspaceImporter};
pub use worker::spawn_import_worker;

#[cfg(test)]
mod tests;
