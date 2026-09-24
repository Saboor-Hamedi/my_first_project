# Keyboard Shortcuts

MindForge provides an extensive suite of intuitive, ergonomic keyboard shortcuts so you can write, navigate, and edit at maximum speed without taking your hands off the keyboard.

---

## Global Shortcuts

- **Ctrl + N**: Create a new blank note
- **Ctrl + S**: Quick save active note to local SQLite database
- **Ctrl + B**: Toggle notes sidebar open or closed
- **Ctrl + .**: Toggle Zen Mode (distraction-free editor)
- **Ctrl + P**: Open fast fuzzy search across all notes
- **Ctrl + R**: Quick rename active note
- **Ctrl + E**: Toggle Inline Live Markdown Mode vs Raw Monospace Editor
- **Ctrl + J**: Toggle integrated interactive Terminal dock
- **Ctrl + \\**: Toggle side-by-side Live Markdown Preview split
- **Ctrl + Shift + I**: Toggle DeepSeek AI Assistant right pane
- **Ctrl + ,**: Open Settings & Preferences modal
- **Ctrl + Shift + D**: Safely delete active note (with confirmation modal)
- **Esc**: Dismiss active modal, cancel key recording, or return to Normal mode

---

## Zen Mode & Modular Interface Commands

MindForge gives you full control over every single piece of the interface. Hide or show any element on demand, and Zen mode is faithfully persisted across sessions:
- **Ctrl + .** or `:zen`: Toggle Zen mode (strictly hides titlebar, tabs, sidebar, preview, and terminal for an uninterrupted focus canvas; preserved in SQLite)
- `:titlebar` (or `:tb`, `:set titlebar` / `:set notitlebar`): Toggle the top window titlebar
- `:sidebar` (or `:sb`, `:set sidebar` / `:set nosidebar`): Toggle notes sidebar
- `:tabs` (or `:tabbar`, `:set tabs` / `:set notabs`): Toggle document tabs strip
- `:ai` (or `:agent`, `:set ai` / `:set noai`): Toggle AI Assistant pane
- `:preview` (or `:set preview` / `:set nopreview`): Toggle live Markdown preview
- `:dashboard` (or `:welcome`): Toggle the Neovim-style ASCII dashboard
- `:blur` (or `:acrylic`, `:mica`, `:noblur`): Toggle Windows DWM Acrylic/Mica backdrop blur
- `:opacity <0.2-1.0>` (or `:set opacity <val>`): Set window opacity percentage
- `:font <name>`: Switch editor typography to JetBrains Mono, Iosevka, Victor Mono, Fira Code, Caskaydia Cove, or Berkeley Mono
- `:settings` (or `:pref`, `:set`): Open Preferences & Settings modal

---

## Window Navigation & Borderless Dragging

- **Alt + Left-Click Drag Anywhere**: Move the borderless window instantly from any position on the canvas or background without needing a titlebar.
- **Top-Edge Grab Strip (7px)**: Drag the window from the top boundary when the titlebar is hidden or in Zen mode.
- **Statusbar Drag**: Grab and drag the window from any unoccupied area of the bottom dock.

---

## Task Lists & Checklist Toggles

- **Ctrl+Shift+X**: Toggle task checkbox (`- [ ]` <-> `- [x]`)
  - **Normal Mode**: Toggles the checkbox on the current line and saves immediately.
  - **Visual / VisualLine Mode**: Toggles every task checkbox within the selection simultaneously. Non-task lines are left untouched.
  - **Live Inline View**: Rendered checkboxes display a crisp green checkmark when completed.

---

## Customizing Keybindings (Live Rebinding)

Every shortcut in MindForge can be dynamically customized in the Settings panel:
1. Press `Ctrl + ,` and click the **Keybindings** tab.
2. Toggle between **Normal Mode** and **Visual Mode** or use the search bar `🔍` to find any command.
3. Click any existing key pill (e.g. `[ k ✕ ]`) to record a new key combination.
4. Click the `[+]` button to bind an alternate key without replacing existing shortcuts.
5. Click `[✕]` on any key badge to unbind that key.
6. Click `[↺]` next to any modified command (or **`↺ Reset All Defaults`** in the header) to revert to factory defaults.
7. All custom bindings are automatically persisted to `keymap.json` in your configuration directory.

---

## Text Editing & Line Manipulation

- **Tab**: Indent line 4 spaces or advance to next table cell
- **Shift + Tab**: Dedent line 4 spaces or retreat to previous table cell
- **Ctrl + ]**: Indent line or selected block 4 spaces right
- **Ctrl + [**: Dedent line or selected block 4 spaces left
- **Ctrl + D**: Duplicate current line or selection directly below
- **Ctrl + C**: Copy selected text to system clipboard
- **Ctrl + X**: Cut selected text to system clipboard
- **Ctrl + V**: Paste text from system clipboard
- **Ctrl + Z**: Undo last edit
- **Ctrl + Y**: Redo last undone edit
- **Ctrl + A**: Select all text in active note

---

## Markdown Tables & Code Blocks

- **Tab** (in table): Move to next cell; automatically appends a new row when pressed at the end of the table
- **Shift + Tab** (in table): Move to previous cell
- **Enter** (in table header): Automatically inserts markdown divider row (`|---|---|`) and starts the first body row
- **Enter** (in table body): Creates a new aligned table row without splitting text
- **Ctrl + Enter**: Cleanly exit a table or code fence block below

---

## Word & Cursor Navigation

- **Ctrl + Left**: Jump cursor one word to the left
- **Ctrl + Right**: Jump cursor one word to the right
- **Ctrl + Backspace**: Delete entire word behind cursor
- **Ctrl + Delete**: Delete entire word ahead of cursor
- **Home**: Jump cursor to start of current visual line
- **End**: Jump cursor to end of current visual line
- **Page Up / Page Down**: Scroll viewport by full page

---

## Line Numbers & Live Preview

- `:nu` / `:set nu`: Show line numbers gutter
- `:nonu` / `:set nonu`: Hide line numbers gutter
- `:preview` / `:set preview`: Toggle live markdown preview side-by-side
- `:nopreview` / `:set nopreview`: Close live markdown preview
- Drag the center pill knob between the editor and preview to smoothly resize the split ratio
