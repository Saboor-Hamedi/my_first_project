//! Application mode state enum.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Normal,
    Stats,
    Doc,
    Help,
    ScanReport,
    ScanHistory,
    Terminal,
}
