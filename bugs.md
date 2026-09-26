# Bugs & Warnings Log

### Status: Fixed & Verified (0 Warnings)

1. **Unknown Lint Warning** (`app/src/main.rs:2:10`):
   - Removed obsolete `#![allow(float_literal_f32_fallback)]` (all float literals have been explicitly typed `_f32`).

2. **Unused Import** (`app/src/view_dashboard.rs:4:54`):
   - Removed unused `Key` import from `eframe::egui`.

3. **Keybinding Alignment**:
   - `Ctrl+P`: Fuzzy note search.
   - `Ctrl+Shift+P` & `Ctrl+,`: Settings & Preferences modal.
   - Removed `Ctrl+F` from search bindings and documentation.
   - Fully documented the `:` command dock and search capabilities in `README.md`.

Verified with `cargo check --workspace` — completed with 0 errors and 0 warnings.

