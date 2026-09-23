//! Inline Live Markdown Editor module.
//!
//! Provides rich live-rendered markdown editing directly in the editor buffer
//! with active-line syntax expansion, dynamic heading sizes, vector task checkboxes,
//! blockquote styling, horizontal rules, and 100% robust pixel-aligned carets & selections.

pub mod charmap;
pub mod classify;
pub mod elements;
pub mod interaction;
pub mod layout;
pub mod parser;
pub mod render;
pub mod spans;
pub mod types;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use charmap::CharMapBuilder;
#[allow(unused_imports)]
pub use classify::{classify_line, classify_lines};
#[allow(unused_imports)]
pub use layout::compute_inline_layout;
#[allow(unused_imports)]
pub use layout::build_line_layout;
pub use render::render_inline_editor;
#[allow(unused_imports)]
pub use spans::parse_inline_spans;
#[allow(unused_imports)]
pub use types::{
    InlineEditorLayout, InlineLine, InlineLineKind, InlineSpan, InlineSpanKind, TableAlign,
    TableRowInfo,
};
