use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub total_files: usize,
    pub inserted_files: usize,
    pub remaining_files: usize,
    pub total_bytes: u64,
    pub inserted_bytes: u64,
}

impl ImportStats {
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * 1024 * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportStatus {
    Idle,
    Scanning,
    Importing { current_file: String },
    Completed { count: usize, total_bytes: u64 },
    Cancelled { count: usize },
    Error(String),
}

pub struct WorkspaceImporter {
    pub is_modal_open: bool,
    pub stats: Arc<RwLock<ImportStats>>,
    pub status: Arc<RwLock<ImportStatus>>,
    pub cancel_token: Arc<AtomicBool>,
    pub is_running: Arc<AtomicBool>,
    pub just_completed: bool,
}

impl Default for WorkspaceImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceImporter {
    pub fn new() -> Self {
        Self {
            is_modal_open: false,
            stats: Arc::new(RwLock::new(ImportStats::default())),
            status: Arc::new(RwLock::new(ImportStatus::Idle)),
            cancel_token: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
            just_completed: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn cancel(&mut self) {
        if self.is_active() {
            self.cancel_token.store(true, Ordering::SeqCst);
        }
    }

    pub fn close_modal(&mut self) {
        self.is_modal_open = false;
    }

    pub fn open_modal(&mut self) {
        self.is_modal_open = true;
    }

    pub fn get_stats(&self) -> ImportStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn get_status(&self) -> ImportStatus {
        self.status.read().map(|s| s.clone()).unwrap_or(ImportStatus::Idle)
    }

    /// Polls whether an import just completed in the background so the app can refresh its database.
    pub fn poll_completion(&mut self) -> Option<usize> {
        let status = self.get_status();
        if let ImportStatus::Completed { count, .. } = status {
            if !self.just_completed {
                self.just_completed = true;
                return Some(count);
            }
        } else if self.is_active() {
            self.just_completed = false;
        }
        None
    }
}
