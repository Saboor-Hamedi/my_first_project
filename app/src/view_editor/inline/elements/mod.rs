//! Dedicated modular components for inline editor elements.
//!
//! Each element (block_quote, code_wrapper, table, heading, task, rule, selection)
//! is encapsulated in its own focused, testable, and DRY module.

pub mod block_quote;
pub mod code_wrapper;
pub mod heading;
pub mod rule;
pub mod selection;
pub mod table;
pub mod task;

pub use block_quote::render_block_quote_wrapper;
pub use code_wrapper::render_code_wrapper_line;
pub use heading::{heading_color, heading_metrics};
pub use rule::render_horizontal_rule;
pub use selection::render_document_selection;
pub use table::render_table_row_decorations;
pub use task::render_task_checkbox;
