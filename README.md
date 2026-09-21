# MINDFORGE v2

A GPU-accelerated, borderless learning system you type into.

MINDFORGE looks like a pitch-black CLI terminal, but runs natively on the GPU with custom-rendered text, animated physics carets (fire, water, electric bolts, comet, rainbow, matrix, ice, glitch, neon, heartbeat), an SM-2 spaced repetition engine, an explanation scratchpad, and a calibrated decision journal.

---

## 1. Quick Start

Run the app in release mode for maximum speed and smooth animations:

```powershell
cargo run --release
```

To run unit tests:
```powershell
cargo test --workspace
```

---

## 2. Window Controls & Navigation

Because MINDFORGE is completely borderless with no OS title bar:

| Action | Shortcut |
|---|---|
| **Toggle Sleek Sidebar** | `Ctrl+B` (Floats with top, bottom, and left gaps) |
| **Instant Fuzzy Search** | `Ctrl+P` or `Ctrl+F` (Searches documents, cards, and decisions) |
| **Caret & Settings Modal** | `Ctrl+,` (Customize styles, themes, and physics) |
| **Quick Save Document** | `Ctrl+S` (Saves directly to SQLite) |
| **Rename Document** | `F2` (Renames active document directly in SQLite) |
| **Move Window** | Hold `Alt` and drag anywhere with the left mouse button |
| **Toggle Fullscreen** | Press `F11` |
| **Close App** | Press `Ctrl+Q` or type `:quit` |
| **Command Line** | Type `:` to open the command prompt at the bottom |
| **Cancel / Close Modal** | Press `Escape` |

---

## 3. 13 Animated Caret Styles

Open the visual settings modal with `Ctrl+,` or change your caret live by typing `:` followed by `:caret <name>`:

| Caret Style | Description |
|---|---|
| `fire` | **Tuned & compact** glowing flame base with additive rising spark particles |
| `water` | Liquid blue cursor with splashing droplets (with gravity) and expanding baseline ripples |
| `electric` | Snapping high-voltage crackling lightning bolts branching on keypress |
| `matrix` | Cascading green phosphor digital rain glyphs trickling downwards |
| `ice` | Crystalline cyan frost shards falling with subtle horizontal drift |
| `glitch` | Cyberpunk RGB chromatic aberration split with brief position jitter |
| `neon` | Triple-layer atmospheric glowing aura expanding from the cursor |
| `heartbeat` | Smooth rhythmic organic pulse breathing animation |
| `comet` | Motion-blur fading trail following cursor movements |
| `rainbow` | Smoothly shifting HSV color spectrum |
| `block` | Classic solid block cursor |
| `beam` | Sleek thin vertical bar |
| `underline` | Classic terminal underscore |

### Caret & Physics Tuning
- `:glide <10-200|off>` — Adjust cursor glide responsiveness (default `50`; `off` for instant jump)
- `:calm` — Toggle ambient idle particles on/off
- `:perf` — Show real-time frame render time in milliseconds in the top-right corner
- `:theme <green|amber|white|ice>` — Change the accent color palette
- `:opacity <0.2-1.0>` — Adjust window transparency (1.0 = solid pitch black)
- `:font <size>` — Adjust font size (default `20`)

---

## 4. Features & Commands

### 4.1 Sleek Floating Sidebar (`Ctrl+B`)
- Floats with gaps from the top, bottom, and left of the window.
- Quick navigation: Editor, Recall, Explain, Decisions, Stats, Caret Settings.
- **Live Document Browser**: Displays documents saved in SQLite. Click any document to open it immediately in the editor.
- **`+` Button**: Start a new untitled note.

### 4.2 Instant Fuzzy Search (`Ctrl+P` / `Ctrl+F`)
- Instant subsequence fuzzy search across:
  - **Saved Documents** (matched by title and body content)
  - **Spaced Repetition Flashcards** (matched by question and answer)
  - **Decisions** (matched by decision and prediction)
- Use `↑` / `↓` arrows to navigate and `Enter` to jump straight into the selected document in the editor.

### 4.3 Spaced Repetition Recall (`:review`)
Retrieval practice with the SuperMemo SM-2 algorithm:
1. Shows the prompt for cards due today.
2. Type your answer from memory and press `Enter`.
3. Compares your response with the true answer.
4. Rate your recall difficulty:
   - `1` = Again (failed, repeats soon)
   - `2` = Hard
   - `3` = Good
   - `4` = Easy
5. Displays your accuracy and typing speed (WPM) upon completion.

### 4.4 Adding Cards (`:add`)
Add flashcards using the delimiter syntax:
```
:add What is the borrow checker? | A compile-time mechanism ensuring memory safety without a GC | rust
```

### 4.5 Feynman Explaining Mode (`:explain <topic>`)
Open a distraction-free explanation screen to teach a concept in simple terms:
```
:explain Ownership in Rust
```
Press `Escape` when done to save your explanation directly to SQLite.

### 4.6 Decision Journal (`:decide`, `:resolve`, `:calibration`)
Track your predictions and calibrate your confidence:
- `:decide` — Step-by-step wizard logging decision, reasoning, prediction, confidence (1-99%), and review date.
- `:resolve` — View decisions ready for review and record if your prediction came true (`1` for true, `0` for false).
- `:calibration` — Displays your **Brier score** (0.00 = perfect calibration, 0.25 = random chance) and accuracy breakdown across confidence buckets (50-59%, 60-69%, 70-79%, 80-89%, 90-99%).

### 4.7 Focus & Stats
- `:focus <topic1> | [topic2]` — Sets your current learning focus displayed in the header.
- `:stats` — Displays learning retention and weekly review volume with block-character graphs.
- `:export` — Exports all flashcards, notes, and decisions into `mindforge_export.md`.

---

## 5. Backing Up Your Data

All data is stored offline in a local SQLite database that auto-migrates on startup.
- **Windows location:** `%LOCALAPPDATA%\mindforge\mindforge\data\mindforge.db` (or alongside the executable in the fallback directory).
- To backup: copy `mindforge.db` to a flash drive or cloud drive.
