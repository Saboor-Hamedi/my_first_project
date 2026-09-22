//! Main text editor view: header, text canvas, smooth scrolling, and caret rendering.

pub mod body;
pub mod header;
pub mod ligatures;
pub mod preview;

pub use body::render_editor_body;
pub use header::{render_editor_header, render_split_editor_header};
pub use preview::render_markdown_preview;
