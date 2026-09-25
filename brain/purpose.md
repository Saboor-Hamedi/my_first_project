# MindForge: Purpose, Architecture & Design Vision

MindForge is an ultra-fast, local-first personal knowledge hub and engineering workspace engineered for thinking, writing, coding, and continuous learning with zero latency and zero distraction.

---

## 1. Core Purpose & Mission

Modern note-taking tools are often weighed down by cloud synchronization lag, sluggish Electron runtimes, forced logins, bloated user interfaces, and noisy visual clutter. MindForge exists to deliver the exact opposite:

- **Instantaneous Responsiveness**: Built with Rust and GPU-accelerated immediate-mode graphics (`egui`), every keystroke renders at native display refresh rates (60Hz / 120Hz / 144Hz) with instant feedback.
- **Local-First & Offline Privacy**: All notes, metadata, settings, search indices, and daily logs are stored in an embedded SQLite database on your local machine. No cloud telemetry, no accounts, and no sync conflicts.
- **Cognitive Flow & Ergonomics**: You never need to lift your hands off the keyboard. Modal Vim motions, Hybrid editing shortcuts, and command-palette controls keep you in deep focus.
- **Unified Engineering Workspace**: Integrates notes, code blocks with syntax highlighting, an embedded terminal emulator, web security scanning tools, and an AI pair-programming assistant under one cohesive roof.

---

## 2. Key Architecture & Capabilities

### The Hybrid / Vim Dual-Engine Editor
- **Vim Mode**: Full modal editing (`NORMAL`, `INSERT`, `VISUAL`, `VISUAL LINE`, `COMMAND`) with standard motions (`w`, `b`, `ci"`, `da(`, `y$`), repeatable multipliers (`3dd`), search jumps (`/`, `?`), and a live `showcmd` HUD.
- **Hybrid Mode**: Familiar modern IDE conventions (`Ctrl+D` duplicate line, `Ctrl+Shift+K` delete line, `Ctrl+Backspace` word delete, auto-closing brackets and quotes).
- **Inline Live Markdown**: Seamless transition between raw markdown syntax and live styled typography (headings, tasks, interactive `- [ ]` / `- [x]` checkboxes, blockquotes, horizontal rules, and dynamic tables).
- **Invisible Scrolling**: Fluid mouse wheel and keyboard-driven scrolling without visual scrollbar bars cluttering the canvas.

### The Right Pane: Dual-Tab Split
- **Live Markdown Preview**: Proportional reading typography, GitHub-flavored tables, callouts, and code blocks with centered 60–80 character reading columns.
- **AI Agent Assistant**: Direct conversational interface with local and cloud LLMs for brainstorming, drafting, code explanation, and note summarization.
- **Resilient Split Geometry**: A tactile resize knob allows resizing the workspace smoothly, enforcing a minimum 150px threshold so neither editor nor assistant collapse unintentionally.

### Zen Mode & Modular Surface Architecture
- **Pure Distraction-Free Writing**: `Ctrl+.` instantly strips away all window framing (titlebar, tabs, sidebar, preview) to leave only the editor and the bottom statusbar.
- **Granular Surface Toggle**: Every individual component of the application (`:titlebar`, `:sidebar`, `:tabs`, `:ai`, `:preview`, `:settings`) can be toggled on or off independently on demand, both in Zen mode and normal mode.
- **Adaptive Modular Layout**: The layout engine dynamically recomputes panel dimensions so that when the titlebar is hidden, the canvas smoothly extends all the way to the top border (`bounds.min.y + 5.0`).

### Bidirectional Wikilinks & Autocompletion
- **Interconnected Knowledge**: Support for `[[Note Title]]` and nested directory links `[[folder/sub/topic]]`.
- **Intelligent Autocompletion**: Typing `[[` in Insert mode displays a sleek, clean title dropdown positioned right beneath the trigger.
- **Smart Bracket Lookahead**: Avoids duplicate `]]` when typing within auto-paired brackets, closing unclosed links seamlessly upon Enter or Tab.
- **Title-Focused Typography**: Autocomplete highlights clean note titles with text-only accents, free from raw URL noise or metadata clutter.
- **Keyboard Navigation**: Navigate suggestions with `Ctrl+J` / `Ctrl+K` or arrow keys, confirm with `Enter` / `Tab`, and dismiss with `Esc`.

### Floating Hover Wikilink Preview
- **Instant Non-Destructive Reading**: Resting the cursor over any `[[link]]` for 150ms reveals a sleek floating preview card with full markdown rendering, tables, and code syntax highlighting.
- **Safe Bridge Corridor & Grace Window**: A 400ms travel grace window lets the user smoothly transition their mouse into the preview card without premature dismissal.
- **Scroll Isolation**: Mouse wheel scrolling inside the preview card never scrolls or disturbs the background editor.
- **Direct Navigation & Dismissal**: Click the link icon on the header to immediately jump to the note, or dismiss via the `×` button, pressing `Esc`, or moving the cursor with keyboard motions (`h, j, k, l`).

### Unified Right Split Pane: Preview, AI, Backlinks & Outline
- **Unified 4-Tab Suite**: Houses Markdown Live Preview (`Ctrl+\`), DeepSeek AI Assistant (`Ctrl+Shift+I`), Backlinks (`Ctrl+I`), and Document Outline (`Ctrl+Shift+O`) within a single, cohesive split pane (`app/src/app/right_pane.rs`).
- **Zero Conflict Architecture**: Sharing the exact same split-ratio, resize knob, and theme surface ensures that live preview, assistant interactions, and document inspectors never collide or occlude each other.
- **Automated Backlink Indexing**: Dynamically queries notes referencing the active document (`:backlinks`, `:bl`, `:links`, or `Ctrl+I`), displaying context snippets, line numbers, and live reference counts (`Links (N)`).
- **Live Document Outline**: Automatically parses and lists headings H1–H6 (`:outline`, `:ol`, `:headings`, or `Ctrl+Shift+O`) with hierarchical indentations and instant jump-to-heading navigation.

### Tab Management & Rapid Navigation
- **Direct Numbered Tab Jump**: `Ctrl+1` through `Ctrl+9` instantly jumps to the corresponding open tab.
- **Seamless Tab Cycling**: `Ctrl+Tab` cycles forward through open notes; `Ctrl+Shift+Tab` cycles backward.
- **Tab Closing**: `Ctrl+W` closes the active tab, saving caret and scroll positions automatically.

### The Docked Terminal Drawer
- Integrated cross-platform shell dock (`PowerShell`, `cmd`, `bash`, `zsh`) accessible via `Ctrl+J`.
- Tabbed multi-session support that collapses to a single clean indicator when only one terminal is active.

### Local SQLite Engine
- Fast atomic transactions with automatic debounced saving.
- Full-text search and fuzzy title matching.
- Automated daily backup snapshots to protect against data loss.

---

## 3. UI/UX Aesthetic Principles

MindForge embraces a quiet, focused aesthetic that respects attention and minimizes cognitive overhead.

### Visual Hierarchy & Layout
- **Uniform 5px Spatial Grid**: All panels, cards, and docks float with uniform 5px margins, creating an airy, breathing canvas.
- **Soft Contrast over Hard Borders**: Panels are defined by subtle background luminance shifts rather than heavy 1px or 2px dividing lines.
- **Tactile Knobs, No Dividing Lines**: Resizable splitters (sidebar, preview, terminal drawer) feature minimal tactile knobs without full-height / full-width separator lines cutting through the workspace.
- **Centered Sidebar Branding**: A clean, centered `MINDFORGE` title in the sidebar with generous whitespace, free of redundant toggle hints.
- **Lightweight Unboxed Text Badges**: Mode indicators (`NORMAL`, `HYBRID`, `:CMD`), docked utilities (`AI Agent`), and autocomplete command tags (`HIST`, `CMD`) use clean, colored typography without clunky background boxes or borders.
- **Unified Content Typography**: The Editor, Markdown Preview, and AI Assistant pane all share the identical reading text color (`theme.text`), keeping the accent color strictly reserved for interactive chrome, active navigation, and toggle knobs.
- **Clean Chat Dynamics**: In the AI panel, user messages anchor cleanly to the right without bubble background fills, assistant messages anchor to the left with readable bubbles, and copy buttons confirm with a clean green checkmark.

---

## 4. What We Are Building Next

1. **Deep Semantic Search**: Local vector embeddings to surface conceptually related notes and code snippets without exact keyword matches.
2. **Enhanced AI Tooling**: Granular file attachments, diff-based note suggestions, and autonomous code execution in the sandbox terminal.
3. **Custom Extensibility**: User script hooks and personalized theme definitions allowing complete visual and functional adaptation.
