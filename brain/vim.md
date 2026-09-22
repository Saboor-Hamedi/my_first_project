# Vim Navigation

MindForge features an authentic modal Vim editing engine for users who prefer home-row navigation, composition, and precision text editing.

---

## How to Turn Vim Mode ON and OFF

You can enable or disable Vim mode at any time using either the command line or the visual settings panel:

### Method 1: Command Bar (`:`)
- **Turn ON**: Press `:` to open the command prompt, type `:vim on` (or `:mode vim`), and press `Enter`.
- **Turn OFF**: Press `:` to open the command prompt, type `:vim off` (or `:mode hybrid`), and press `Enter`.
- **Toggle Quickly**: Type `:vim` with no arguments to toggle instantly between Vim mode and Hybrid mode.

### Method 2: Preferences / Settings (`Ctrl + ,`)
1. Press `Ctrl + ,` to open the Settings modal.
2. Select the **EDITOR MODE** tab in the left sidebar.
3. Click either:
   - **`VIM` chip**: Enables authentic modal editing with Normal, Insert, Visual, and Command submodes.
   - **`HYBRID` chip**: Modern IDE mode with standard cursor navigation, selection, and auto-pairing.
4. Your choice is automatically saved and remembered across application restarts.

### Active Mode Indicator
Check the bottom status dock anytime:
- **`-- NORMAL --` / `-- INSERT --` / `-- VISUAL --`**: Vim Mode is active.
- **`HYBRID`**: Standard modern IDE mode is active.

---

## Editor Submodes

In Vim mode, the editor operates across distinct submodes:
- Normal Mode: For navigation, paragraph traversal, text objects, and commands.
- Insert Mode: For standard typing and text insertion.
- Visual Mode (v): For character-wise selection.
- Visual Line Mode (V): For full-line selection and block actions.
- Command-line Mode (:): For file, documentation, and app commands.

> Note: In MindForge, your chosen caret style remains identical across Normal, Insert, and Visual modes for consistent visual feedback.

---

## Home-Row Navigation

- h / j / k / l: Move cursor Left, Down, Up, and Right
- w: Jump forward to start of next word
- b: Jump backward to previous word
- e: Jump forward to end of current word
- 0: Jump to beginning of line
- $: Jump to end of line
- gg: Jump to the very beginning of the document
- G: Jump to the very end of the document

---

## Mode Transitions

- i: Enter Insert mode before cursor
- I: Enter Insert mode at beginning of line
- a: Enter Insert mode after cursor
- A: Enter Insert mode at end of line
- o: Insert new line below and enter Insert mode
- O: Insert new line above and enter Insert mode
- v: Toggle Visual character selection mode
- V: Toggle Visual Line selection mode
- Esc: Return to Normal mode

---

## In-Buffer Search

- /pattern: Search forward in document for matching text
- ?pattern: Search backward in document
- n: Jump to next search occurrence
- N: Jump to previous search occurrence

---

## Command Mode (: )

Press : from Normal mode to open the command palette:
- :w: Save active note immediately
- :q: Quit MindForge
- :doc: Open built-in Documentation viewer
- :editor: Return to Notes editor
- :stats: Open daily writing statistics
- :d or :delete: Delete active note with confirmation prompt

---

## ShowCmd Keystroke & Command HUD

MindForge features a borderless, floating Heads-Up Display (HUD) in the bottom-right corner of the editor window:
- **`[VIM]` badge**: Real-time tracking of pending multi-key operators (e.g. `d`, `c`, `40j`, `ci"`, `ya)`).
- **`[VIS]` badge**: Visual mode selection chords (e.g. `vi"`, `va(`).
- **`[FIND]` badge**: Active forward and backward search queries (`/needle`, `?query`).
- **`[CMD]` badge**: Live command-line buffer tracking (`:w`, `:doc`, `:editor`).
- **Auto Fade-out**: Automatically fades after 2.0s for pending commands and 1.2s for completed actions.
