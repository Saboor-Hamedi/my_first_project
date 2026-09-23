//! Dedicated modular components for inline editor elements.
//!
//! Each element (block_quote, code_wrapper, table, heading, task, list, rule, selection)
//! is encapsulated in its own focused, testable, and DRY module.

pub mod block_quote;
pub mod code_wrapper;
pub mod heading;
pub mod list;
pub mod rule;
pub mod selection;
pub mod table;
pub mod task;

pub use block_quote::{quote_color, quote_indent, render_block_quote_wrapper};
#[allow(unused_imports)]
pub use code_wrapper::{code_block_copy_button_rect, code_metrics, highlight_code_chars, render_code_block_card};
pub use heading::{heading_color, heading_metrics};
pub use list::{bullet_glyph, checkbox_glyph, number_glyph};
#[allow(unused_imports)]
pub use rule::{render_horizontal_rule, rule_metrics};
pub use selection::render_document_selection;
#[allow(unused_imports)]
pub use table::{
    cell_color, parse_aligns, pipe_color, render_table_block_decorations,
    render_table_row_decorations, split_table_cells, table_metrics, TableCellSpan,
};
pub use task::render_task_checkbox;
