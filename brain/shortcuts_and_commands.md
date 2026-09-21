# Shortcuts & Command Reference

MindForge is designed around a fluid, keyboard-first editing experience. Everything can be operated without touching a mouse.

---

## ⌨️ Global Application Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + S` | **Quick Save** | Saves the active document to SQLite immediately with timestamped status. |
| `Ctrl + N` | **New Note** | Creates a fresh, empty note in `Normal` mode. Automatically saves active note if dirty. |
| `Ctrl + R` | **Rename Note** | Opens the centered rename modal dialog. |
| `Ctrl + Shift + D` | **Delete Note** | Opens the safe, centered red delete confirmation dialog. |
| `Ctrl + P` / `Ctrl + F` | **Fuzzy Search** | Opens the centered fuzzy search modal across all notes in the database. |
| `Ctrl + B` | **Toggle Sidebar** | Smoothly expands or collapses the left notes navigation drawer. |
| `Ctrl + ,` | **Settings Panel** | Opens the integrated settings drawer (Themes, Caret, Sound, Editor Mode, Backup). |
| `Ctrl + Z` | **Undo** | Reverts to the previous snapshot in the undo history stack. |
| `Ctrl + Shift + Z` / `Ctrl + Y` | **Redo** | Restores an undone snapshot from the redo history stack. |
| `Ctrl + A` | **Select All** | Selects the entire document buffer. |
| `Ctrl + C` | **Copy** | Copies selected text to the system clipboard. |
| `Ctrl + X` | **Cut** | Cuts selected text to the system clipboard and deletes it from the buffer. |
| `Ctrl + V` | **Paste** | Pastes text from the system clipboard into the editor. |
| `Ctrl + Enter` | **Insert Line Below** | Inserts a new line below the current line without splitting words, and moves caret there. |
| `Ctrl + Q` / `Ctrl + Shift + W` | **Quit** | Saves dirty active notes and cleanly closes the application. |
| `Esc` | **Dismiss / Cancel** | Closes any open modal (Search, Rename, Delete, Settings) or returns to Normal mode. |

---

## ✍️ Hybrid Mode Shortcuts (Default IDE Mode)

Hybrid mode blends modern code editor productivity with streamlined writing ergonomics.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + D` | **Duplicate Line** | Duplicates the current line directly below the cursor, preserving column. |
| `Ctrl + Backspace` | **Delete Word Backward** | Deletes preceding whitespace and the entire word before the caret. |
| `Ctrl + Delete` | **Delete Word Forward** | Deletes trailing whitespace and the entire word after the caret. |
| `Ctrl + Left` | **Word Left** | Moves cursor to the start of the previous word. |
| `Ctrl + Right` | **Word Right** | Moves cursor to the start of the next word. |
| `Ctrl + Shift + Left` | **Select Word Left** | Expands text selection one word to the left. |
| `Ctrl + Shift + Right` | **Select Word Right** | Expands text selection one word to the right. |
| `Shift + Up` / `Down` | **Multi-line Select** | Expands selection vertically across visual lines, empty line gaps, and whitespace. |
| `Shift + Left` / `Right` | **Character Select** | Expands selection horizontally character by character. |
| `Tab` | **Indent** | Inserts 4 spaces at cursor (or indents selection). |
| `Shift + Tab` | **Dedent** | Removes up to 4 spaces from the start of the line. |
| `(` `[` `{` `"` `'` `*` | **Auto-Pairing** | Automatically inserts closing pair `)`, `]`, `}`, `"`, `'`, `*`. If text is selected, wraps it! |

---

## 🧙 Vim Mode Keybindings

When switched to Vim mode (`:vim` or Settings > Editor Mode), MindForge operates as a modal Vim engine with dynamic caret shape morphing (Block in Normal, Beam in Insert).

### Sub-Modes
- **`-- NORMAL --`** (Status Badge: `NORMAL`): Navigation, motions, and operations.
- **`-- INSERT --`** (Status Badge: `INSERT`): Natural free-form typing.
- **`-- VISUAL --`** (Status Badge: `VISUAL`): Character-wise selection.
- **`-- VISUAL LINE --`** (Status Badge: `V-LINE`): Whole-line selection.

### Transitions
- `i` : Enter Insert mode before cursor.
- `a` : Enter Insert mode after cursor.
- `I` : Move cursor to line beginning and enter Insert mode.
- `A` : Move cursor to line end and enter Insert mode.
- `o` : Open a new line below and enter Insert mode.
- `O` : Open a new line above and enter Insert mode.
- `v` : Enter character Visual mode.
- `V` : Enter line-wise Visual Line mode.
- `Esc` or `Ctrl + [` : Return to Normal mode and clear selection.

### Normal Motions & Navigation
- `h`, `j`, `k`, `l` : Left, Down, Up, Right.
- `w` : Jump forward to beginning of next word.
- `b` : Jump backward to beginning of previous word.
- `0` : Jump to start of current line.
- `$` : Jump to end of current line.
- `G` : Jump to end of document.
- `g g` : Jump to top of document.

### Normal Operators & Editing
- `d d` : Delete entire line and yank into register.
- `d w` : Delete from cursor to next word start.
- `x` : Delete character under cursor.
- `y y` : Yank (copy) current line into register.
- `y w` : Yank next word into register.
- `p` : Paste register contents after cursor (or below current line if whole line).
- `P` : Paste register contents before cursor (or above current line if whole line).
- `u` : Undo last edit.
- `Ctrl + R` : Redo undone edit.
- `Ctrl + D` (in Insert Mode): Duplicate current line directly below.

### Visual Mode (`v` and `V`)
- `h`, `j`, `k`, `l` : Expand selection in direction, highlighting empty line gaps seamlessly.
- `w`, `b`, `0`, `$` : Expand selection to word/line boundaries.
- `y` / `Ctrl + C` : Yank selected text into register and return to Normal mode.
- `d` / `x` : Delete selected text and return to Normal mode.

---

## 💻 Bottom Command Dock (`:CMD`)

Type `:` in the bottom command bar to activate the command dispatcher:

| Command | Arguments | Description |
| :--- | :--- | :--- |
| `:w` / `:save` | *(none)* | Saves the active document to SQLite. |
| `:vim` | `[on\|off]` | Toggles or sets Vim mode. Persists across restarts. |
| `:mode` | `[hybrid\|vim]`| Switches between `hybrid` and `vim` editor modes. |
| `:r` / `:rename` | `[new title]` | Renames current note immediately, or opens rename modal if no argument provided. |
| `:d` / `:delete` / `:rm` | *(none)* | Permanently deletes active note and switches to adjacent note. |
| `:backup` | *(none)* | Executes instant database backup into `mindforge_backup/`. |
| `:export` | `[filename]` | Exports active note as Markdown (`.md`) or Plain Text (`.txt`). Uses native file picker if empty. |
| `:import` | `[path]` | Imports an existing `.md` or `.txt` file into the editor. |
| `:sound` | `[mechanical\|typewriter\|soft\|silent]` | Sets typing audio feedback profile. |
| `:caret` | `[block\|beam\|underline\|glow]` | Configures cursor aesthetic. |
| `:theme` | `[obsidian\|nord\|dracula\|solarized]` | Applies color palette across the entire application. |
| `:stats` | *(none)* | Displays productivity & SRS statistics dashboard. |
| `:clear` | *(none)* | Wipes document buffer for the current session. |
| `:help` | *(none)* | Displays available commands in the status bar. |
| `:quit` / `:q` | *(none)* | Saves and exits MindForge cleanly. |
