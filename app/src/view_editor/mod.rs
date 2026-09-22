//! Main text editor view: header, text canvas, smooth scrolling, and caret rendering.

pub mod body;
pub mod header;
pub mod ligatures;

pub use body::render_editor_body;
pub use header::render_editor_header;
