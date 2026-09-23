# Mindforge Embedded Terminal Architecture & User Guide (`terminal.md`)

This document provides a comprehensive technical reference and user guide for Mindforge's embedded Linux/Bash terminal system.

---

## 1. System Overview & Architecture

Mindforge integrates a full-featured, hardware-accelerated terminal emulator directly inside the desktop application. Unlike basic command runners or simulated text boxes, this is an **interactive PTY (Pseudo-Terminal)** backed by the engine behind the Alacritty terminal emulator.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Mindforge Desktop Application                   │
├──────────────┬─────────────────────────────────────────────────────────┤
│              │                   Top Panel (Editor / Preview)          │
│              │                                                         │
│   Sidebar    ├─────────────────────────────────────────────────────────┤
│  (Full Ht)   │ ══════════════ [ Tactile Knob Splitter ] ══════════════ │
│              ├──────────────────────────────────────────┬──────────────┤
│  Resizable   │              Terminal View               │   SESSIONS   │
│  Independent │  • Real PTY via alacritty_terminal       │  [+] New     │
│              │  • Git Bash with ~/.bashrc & aliases     │  💻 bash 1   │
│              │  • Keystroke Direct PTY Dispatch         │  💻 bash 2   │
└──────────────┴──────────────────────────────────────────┴──────────────┘
```

### Core Components
- **PTY Engine**: `alacritty_terminal` with ConPTY on Windows and POSIX PTYs on Linux/macOS.
- **Rendering Layer**: `egui_term 0.1` rendering into Mindforge's `eframe` canvas.
- **Input Pipeline**: Direct event routing in `app/src/terminal_pane.rs` and `app/src/input/mod.rs` to guarantee non-blocking typing and control-sequence dispatch.
- **Session Manager**: Multi-session concurrent process pool managed via the terminal's right sidebar.

---

## 2. Shell Environment & Full Bash Integration (`~/.bashrc`)

### The `.bashrc` Problem & Resolution
Previously, Windows installations spawned `cmd.exe` via `COMSPEC`, which ignored user aliases, shell customizations, and Unix commands. When users ran Git Bash, if launched as a non-login shell without an explicit `HOME` directory, `~/.bashrc` and `~/.bash_profile` were skipped.

Mindforge solves this automatically through dynamic candidate cascade:

1. **Automatic Git Bash Resolution**:
   Scans common and standard installation directories:
   - `C:\Program Files\Git\bin\bash.exe`
   - `C:\Program Files\Git\usr\bin\bash.exe`
   - `C:\Program Files (x86)\Git\bin\bash.exe`
   - `%LOCALAPPDATA%\Programs\Git\bin\bash.exe`
   - `C:\msys64\usr\bin\bash.exe`
   - `PATH` entries containing `bash.exe` (excluding System32 WSL stubs)

2. **Login Shell Flag (`--login`)**:
   When launching Bash, Mindforge passes:
   ```rust
   vec!["--login".to_string()]
   ```
   - `--login`: Instructs Bash to act as a login shell, reading `/etc/profile`, `~/.bash_profile`, and `~/.profile`.
   - By running cleanly once, it sources `~/.bashrc` without duplicate conda execution, eliminating path search warnings and accelerating startup.

3. **`HOME` Environment Variable Bridging**:
   On Windows, `USERPROFILE` is always set (e.g. `C:\Users\Saboor`), but `HOME` is often unset. Mindforge detects this on startup and bridges `HOME = USERPROFILE`.
   As a result, Git Bash evaluates `~` directly as `C:\Users\Saboor`, reliably sourcing:
   - `~/.bash_profile`
   - `~/.bashrc` (all custom aliases like `alias v="nvim"`, `alias cc="clear"`, `alias all="git add ."`, `alias p="python"`, Miniconda environments, and Oh-My-Posh prompts).

---

## 3. Multi-Session Manager (Right Sidebar)

Mindforge features a multi-session manager displayed in a dedicated sidebar on the right side of the terminal dock:

### Features & Capabilities:
- **`[+]` New Session**: Spawns an independent PTY process and attaches it to the session list.
- **Concurrent Execution**: Background sessions stay alive. Long-running tasks, dev servers, or builds continue running even when switching to another tab.
- **Visual Session State**:
  - The active session is highlighted with the current Mindforge theme accent bar and pill.
  - Hover states provide visual feedback.
  - Header tag `[bash 1]`, `[bash 2]` displays the active session's identifier.
- **Session Termination**:
  - Each session in the right sidebar has a dedicated `✕` button.
  - Closing a session terminates its PTY process cleanly.
  - If the last remaining session is closed, the terminal dock automatically collapses.

---

## 4. Docked Geometry & Layout Integration

Mindforge's layout keeps the terminal integrated without disrupting the workspace:

### 1. Docked Underneath Editor & Preview
- The terminal **never covers the entire screen**.
- It docks strictly beneath the active editor and markdown live preview.
- **Sidebar Protection**: The left navigation/file sidebar extends the full height of the window, remaining completely unaffected by the terminal dock and fully resizable at all times.

### 2. Uniform 5px Splitter & Tactile Knob
- A 5px horizontal splitter divides the editor/preview and the terminal dock.
- A centered tactile knob provides visual feedback.
- Dragging the splitter adjusts the height split ratio (`terminal_split_ratio`, default `0.38`).
- Drag-to-close thresholds: Dragging the splitter near the bottom (< 8% height) or near the top (> 92% height) automatically closes the dock and restores the split ratio to default.

---

## 5. Keyboard Navigation & Control Protocol

| Shortcut | Context | Action |
| :--- | :--- | :--- |
| **`Ctrl + J`** | Global (Anywhere) | **Toggle Terminal Open / Closed** |
| **`Ctrl + \``** | Global (Anywhere) | Alternative toggle (Backtick / Tilde) |
| **`:term` / `:terminal`** | Command Bar (`:`) | Open/focus terminal from command bar |
| **`Escape`** | Inside Terminal | **Release focus to Editor** (status becomes unfocused) |
| **Click into Terminal** | Terminal Dock | Focus terminal (`● ACTIVE`) |
| **Click into Editor** | Editor Area | Focus editor (status becomes unfocused) |
| **`✕` Button** | Terminal Header | Close terminal dock |
| **`✕` on Session Row** | Sessions Sidebar | Close that specific terminal session |
| **`+` Button** | Sessions Sidebar | Open a new concurrent session |

### Input Routing Guarantee
When the terminal is open:
1. **Typing `:`**: When the terminal is unfocused (via `Escape` or clicking into the editor), pressing `:` immediately invokes the `:` command bar.
2. **Interactive Shell Commands**: When the terminal is focused (`● ACTIVE`), keystrokes bypass editor shortcuts and route directly to the active PTY:
   - `Enter` (`\r`), `Backspace` (`\x7f`), `Tab` (`\t` for autocomplete).
   - Arrow keys (`Up`/`Down` command history, `Left`/`Right` navigation).
   - `Ctrl+C` (SIGINT break), `Ctrl+D` (EOF exit), `Ctrl+L` (Clear screen), `Ctrl+Z` (SIGTSTP suspend).
   - All standard bash editing shortcuts (`Ctrl+A` line start, `Ctrl+E` line end, `Ctrl+K` cut line, `Ctrl+U` delete to start, `Ctrl+W` delete word, `Ctrl+R` reverse history search).

---

## 6. Dynamic Theme Synchronization

Terminal styling automatically adapts to Mindforge's theme settings:
- Background matches `theme.bg`.
- Foreground text matches `theme.text`.
- Selection and cursor colors match `theme.accent`.
- Header and sessions sidebar match `theme.surface()` and `theme.sidebar_bg`.
- ANSI colors (bright white, cyan, blue, dim foreground) are mapped directly from the active `ThemeKind` (Dark, Light, Monokai, Nord, Gruvbox, Cyberpunk, etc.).
- When the user switches themes via `:theme <name>` or the Settings modal, the terminal updates its color palette dynamically on the very next frame.
