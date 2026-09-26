# Compiler Warnings & Bugs Resolution Log

All 207 compiler warnings reported in `bugs.md` have been resolved across the workspace.

## Summary of Fixes

### 1. `float_literal_f32_fallback` (Rust Issue #154024)
- **Root Cause**: Floating-point literals (such as `1.0`, `1.5`, `2.0`, `0.8`, `1.8`, `1.3`) passed directly into `Stroke::new(...)` or other APIs expecting `impl Into<f32>` previously relied on implicit fallback to `f32`. In future rustc versions, this fallback becomes a hard compile error.
- **Resolution**: Explicitly specified the float literal type with `_f32` (e.g. `1.0_f32`, `1.5_f32`, `2.0_f32`, `0.8_f32`, `1.8_f32`, `1.3_f32`) across all UI rendering and painter calls in:
  - `app/src/app/modals.rs`
  - `app/src/app/panes.rs`
  - `app/src/app/shell.rs`
  - `app/src/agent/deepseek_ui.rs`
  - `app/src/docs.rs`
  - `app/src/lunaline/render.rs`
  - `app/src/rightsidebar/backlinks.rs`
  - `app/src/rightsidebar/mod.rs`
  - `app/src/rightsidebar/outline.rs`
  - `app/src/scan_history_view.rs`
  - `app/src/scan_view.rs`
  - `app/src/sidebar/body.rs`
  - `app/src/sidebar/footer.rs`
  - `app/src/terminal_pane.rs`
  - `app/src/ui_components/icons.rs`
  - `app/src/ui_components/toggle.rs`
  - `app/src/view_editor/body.rs`
  - `app/src/view_editor/inline/elements/code_wrapper.rs`
  - `app/src/view_editor/inline/elements/rule.rs`
  - `app/src/view_editor/inline/elements/table.rs`
  - `app/src/view_editor/inline/elements/task.rs`
  - `app/src/view_editor/inline/render.rs`
  - `app/src/view_editor/inline/render/active.rs`
  - `app/src/view_editor/inline/render/inactive.rs`
  - `app/src/view_editor/preview/header.rs`
  - `app/src/view_editor/preview/parser.rs`
  - `app/src/view_editor/preview/render.rs`
  - `app/src/view_editor/tabs.rs`
  - `app/src/view_editor/titlebar.rs`
  - `app/src/view_stats.rs`
  - `app/src/wikilink/hover_wikilink.rs`
  - `app/src/wikilink/wikilink_autocompletion.rs`
  - `app/src/workspace_import/drag_drop.rs`
  - `app/src/workspace_import/import_ui.rs`

### 2. Cross-Platform Unused Items (Linux/macOS CI)
- **`app/src/sound.rs`**:
  - Gated Windows-only structs (`WaveFormatEx`, `WaveHdr`, `Default for WaveHdr`, `NUM_CHANNELS`) behind `#[cfg(windows)]`.
  - Added `#[allow(dead_code)]` to `channel_idx`.
  - Prefixed unused non-Windows argument `sample_rate` as `_sample_rate`.
- **`app/src/blur.rs`**:
  - Prefixed non-Windows argument `effect` as `_effect` and bound locally on Windows to prevent `unused_variables` warnings.

## Verification
- `cargo check --workspace`: **0 errors, 0 warnings**
- `cargo test --workspace`: **187 passed; 0 failed; 0 warnings**
