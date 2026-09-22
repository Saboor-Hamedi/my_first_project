# MindForge — Future Roadmap & Architecture Plan

---

## 🎯 Vision
Transform MindForge from a distraction-free notebook into a high-performance, keyboard-first, local-native knowledge workstation and lightweight code/note editor that can rival GUI-first Vim/Neovim experiences.

---

## 🗺️ Milestone 1: Editor & Vim Engine Polish (Stability First)
- [ ] **Dot Repeat (`.`) Engine**: Full record-and-replay of last mutating action (motions + text mutations).
- [ ] **Registers & System Clipboard Integration**:
  - Unnamed register `""` sync with OS clipboard option.
  - Named registers `"a` through `"z` persistent across sessions.
  - Blackhole register `"_`.
- [ ] **Line-based motions**: `H`, `M`, `L` (screen line jumps), `zt`, `zz`, `zb` (viewport scroll without moving cursor).
- [ ] **Visual Block Mode (`Ctrl+V`)**: Columnar selections, block insertions (`I`), and block appends (`A`).
- [ ] **Undo Tree / Branching History**: Visual timeline or list representation of undo branches instead of purely linear history.

---

## ⚡ Milestone 2: Syntax Highlighting & Visual Engine
- [ ] **Tree-sitter or Syntect Integration**: Fast, tree-based syntax coloring for Markdown, Rust, Python, JavaScript, JSON, and TOML.
- [ ] **Inline Code Block Evaluation / Scratchpad**: Execute small snippets or preview output inline.
- [ ] **Gutter & Line Numbers**:
  - Relative / hybrid line numbers toggleable via `:set rnu` / `:set nornu`.
  - Git diff indicators in the margin gutter (added, modified, deleted lines).
- [ ] **Fold / Unfold Engine**: Markdown header folding (`za`, `zc`, `zo`) with subtle folding pill indicators.

---

## 📂 Milestone 3: Workspace & File Management
- [ ] **Native File Tree Explorer**: Left sidebar toggle (`Ctrl+E` / `:Ex`) displaying real filesystem directories, not just SQLite notes.
- [ ] **Multi-Tab & Split Panes**:
  - Horizontal (`:sp`) and Vertical (`:vsp`) editor splits with smooth resizable dividers.
  - Tab bar with clean keyboard navigation (`gt`, `gT`, `Ctrl+W` window motions).
- [ ] **Project-wide Ripgrep Search**: Blazing fast workspace text search (`Ctrl+Shift+F` / `:grep`) with live jump list.

---

## 🧠 Milestone 4: Knowledge Graph & Spaced Repetition (MindForge Core)
- [ ] **Backlinks & Bidirectional Wiki-Links**: `[[Note Title]]` autocompletion with floating preview popup on hover/cursor.
- [ ] **Interactive 2D Knowledge Graph**: GPU-accelerated force-directed graph view of note connections using egui/wgpu.
- [ ] **SM-2 Flashcard Dashboard**: Dedicated review session screen with spaced repetition analytics and heatmaps.
- [ ] **Daily Journaling Auto-template**: One-key daily note generator (`Ctrl+J` / `:today`).

---

## 🧩 Milestone 5: Config & Extensibility
- [ ] **JSON Keymaps & Custom Bindings (`vim.json`)**: User-definable key bindings, remaps, and leader key combos (`<Space>`).
- [ ] **Theme JSON Engine**: Drop-in user themes in a `themes/` directory (Tokyo Night, Catppuccin, Gruvbox, Nord).
- [ ] **LSP Client Foundation (Language Server Protocol)**:
  - Hover doc tooltips.
  - Autocomplete dropdown.
  - Go to definition (`gd`).

---

## 🏗️ Codebase Architecture & Modularization
- [ ] **`app.rs` De-bloating**:
  - Extract modal state and handlers into `app/src/modals/`.
  - Extract activity tracking / stats flush into `app/src/activity/`.
- [ ] **Vim Subsystem Isolation**: Move `app/src/vim/` to a completely decoupled crate or distinct module boundary with zero egui rendering dependencies.
