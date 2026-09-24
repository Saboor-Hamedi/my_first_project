//! Markdown preview subsystem: parser, syntax highlighter, header tabs, and live renderer.

pub mod blockquote;
pub mod code_block;
pub mod header;
pub mod heading;
pub mod parser;
pub mod render;
pub mod syntax;
pub mod table;
#[cfg(test)]
pub mod tests;

pub use header::{render_right_pane_header, RightPaneAction};
pub use parser::{build_inline_job, parse_markdown, MdBlock};
pub use render::{render_markdown_document, render_markdown_preview};
pub use syntax::highlight_code_line;
