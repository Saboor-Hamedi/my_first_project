# Customization & Features

MindForge provides deep personalization settings and built-in power features to match your exact workspace preferences.

---

## Dynamic Keybindings Manager

MindForge features a comprehensive, VSCode-style keyboard shortcut manager in Settings (`Ctrl + ,`):
- **Live Keystroke Recorder**: Click any key slot to rebind. The centered recorder modal listens to your real keystroke combination with zero interference.
- **Multiple Keys Per Command**: Bind multiple alternative keys to a single action (e.g. `k` and `ArrowUp` for Move Up) using the `[+]` button.
- **Conflict Detection**: Real-time warnings alert you if a key combination is already bound to another command, with seamless reassignment on Enter.
- **Fast Continuous Editing**: Click any key slot or `[+]` button while the recorder is open to switch immediately without closing and reopening.
- **Factory Reset**: Revert any single command with its `[↺]` button, or restore all default shortcuts with **`↺ Reset All Defaults`**.
- **JSON Configuration**: All bindings are read from and automatically saved to `keymap.json` in your configuration directory.

---

## Inline Live Markdown vs Raw Monospace Mode

Press **`Ctrl + E`** to toggle between two distinct writing modes:
- **Inline Live Markdown Mode**: Renders headings, bold, italic, code spans, blockquotes, and tables in-place while you type.
- **Checklist Checkmarks**: Task items (`- [ ]` and `- [x]`) render as beautiful interactive checkboxes with green completed marks.
- **Raw Monospace Mode**: Pure plaintext monospace editor for distraction-free code and markdown editing.

---

## Integrated Interactive Terminal

Press **`Ctrl + J`** to toggle the built-in terminal dock at the bottom of the screen:
- Run commands, git operations, and build tools directly within MindForge.
- Full shell execution with color-coded stdout and stderr streams.
- Preserves terminal session output across view toggles.

---

## Themes & Visual Design

Open Settings with `Ctrl + ,` to customize your visual palette:
- **Color Themes**: Amber, Emerald, Indigo, Rose, Teal, Cyan, Violet, and Monochrome.
- **Accent Colors**: Select from a rich palette of accent colors with real-time UI previews.
- **Code Ligatures**: High-performance vector drawing for arrows (`->`, `=>`), comparisons (`!=`, `<=`, `>=`), and programming symbols.

---

## Caret Styles & Effects

Choose from over 12 distinct caret styles with smooth physics and particle animations:
- **Classic**: Block, Beam, and Underline.
- **Elemental**: Fire, Water, Ice, Electric, and Comet.
- **Futuristic**: Neon, Matrix, Glitch, and Rainbow.
- **Caret Width & Glow**: Fully customizable sliders in the Carets tab.

---

## Mechanical Keyboard Audio

MindForge features synthesized mechanical switch audio feedback:
- **Sound Profiles**: Blue, Brown, Red, and Black switch acoustics.
- **Volume & Mute**: Adjust audio volume or mute completely.
- **Zero Lag**: Audio is synthesized in real-time with zero external latency.

---

## Atomic Database Backups

Your data is stored in an ACID-compliant local SQLite database:
- Press "Create Backup Now" in Settings to create a timestamped snapshot of your entire notes library.
- Configure custom backup directory destinations.
- Database integrity checks and WAL journaling ensure zero data corruption.

---

## Writing Statistics & Tracking

Switch to the Stats view from the sidebar (or type `:stats`) to monitor your writing habits:
- Track today's word count, keystrokes, and active editing duration.
- View 30-day activity histories and lifetime totals.
- All statistics are computed and saved locally without third-party tracking.
