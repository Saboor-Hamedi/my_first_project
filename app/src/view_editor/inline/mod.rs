//! Inline Live Markdown Editor module.
//!
//! Provides rich live-rendered markdown editing directly in the editor buffer
//! with active-line syntax expansion, dynamic heading sizes, vector task checkboxes,
//! blockquote styling, horizontal rules, and 100% robust pixel-aligned carets & selections.

pub mod elements;
pub mod interaction;
pub mod layout;
pub mod parser;
pub mod render;
pub mod types;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use layout::compute_inline_layout;
pub use render::render_inline_editor;
#[allow(unused_imports)]
pub use types::{InlineEditorLayout, InlineLine, InlineLineKind};
