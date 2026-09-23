# Prompt for coding agent: add a real embedded terminal (`:term`) to Mindforge — v2

**This replaces the previous terminal prompt.** That version had the agent hand-roll
PTY spawning (`portable-pty`) and ANSI parsing (`vte`) from scratch. That was more
risk than necessary: `egui_term` (https://github.com/Harzu/egui_term) already does
this job, using `alacritty_terminal` — the actual engine behind the real Alacritty
terminal — as its backend, instead of a partial VT100 implementation written by
hand. Use it as the primary path. Section 6 below is the old hand-rolled approach,
kept only as a fallback if `egui_term` turns out not to fit (see step 0).

## Step 0 (do this first): check version compatibility

`egui_term = "0.1"` depends on `egui ^0.31.0`. Check Mindforge's current
`eframe`/`egui` version in `app/Cargo.toml`.

- **Versions match (or Mindforge can reasonably upgrade to 0.31.x):** proceed with
  `egui_term`, sections 1-5 below.
- **Versions conflict and upgrading egui isn't acceptable right now:** `TerminalView`
  won't type-check against your `Ui`, full stop — fall back to Appendix A (the
  hand-rolled `portable-pty` + `vte` approach), which has zero dependency on egui's
  version since it doesn't touch egui at all, only exposes a plain grid of cells for
  `app` to paint with whatever egui version it's on.

## Same constraint as before: don't touch the `:` command bar

Everything in the earlier discussion still applies: `:term` is one more arm in the
existing `execute_command` match (same shape as `:scan`/`:stats`), keystrokes inside
terminal mode bypass `execute_command`/`handle_command_key` entirely, and the shell
session should persist across mode switches rather than respawning. Reuse the
`app.scan_rx` / `app.prev_mode_before_scan` pattern already in the real `commands.rs`
— call the terminal's equivalents `app.term_backend` / `app.prev_mode_before_term`.

## 1. Before writing any integration code: read the real examples

`egui_term`'s public surface (confirmed via docs.rs) is:
```
Structs: BackendSettings, Binding, ColorPalette, FontSettings, TerminalBackend,
         TerminalFont, TerminalTheme, TerminalView
Enums:   BackendCommand, BindingAction, InputKind
Aliases: KeyboardBinding, PtyEvent, TerminalMode
Macro:   generate_bindings
```
That's the shape, but not the exact call sequence — clone the repo and read these two files before writing any integration code, they are the ground truth for how the pieces actually connect:
- `github.com/Harzu/egui_term/blob/main/examples/full_screen/main.rs` — minimal working terminal: spawning `TerminalBackend`, polling `PtyEvent`s, drawing `TerminalView`.
- `github.com/Harzu/egui_term/blob/main/examples/custom_bindings/main.rs` — how to add a custom keybinding, which is exactly what's needed for "the key that exits terminal mode back to Mindforge" (see section 4).

Do not guess method names or invent an API surface — match what those files actually do.

## 2. Cargo changes

`app/Cargo.toml`:
```toml
egui_term = "0.1"
```
No `portable-pty` or `vte` needed — `egui_term` already depends on `alacritty_terminal`
internally, which supersedes both.

No new `terminal/` crate is needed either, unlike the `webscan`-style separate-crate
pattern used elsewhere in this project: `egui_term` is already a self-contained
widget library, so the integration surface is small enough to live directly in
`app` (e.g. a new `app/src/terminal_pane.rs`). If it grows a lot of Mindforge-specific
logic later (multiple named sessions, persistence, etc.), it can be split out then.

## 3. Sketch of the shape (confirm exact calls against the real examples)

```rust
// app/src/terminal_pane.rs — confirm every call here against
// examples/full_screen/main.rs before treating this as correct.

use egui_term::{TerminalBackend, TerminalView, BackendSettings, PtyEvent};
use std::sync::mpsc;

pub struct TerminalPane {
    backend: TerminalBackend,
    event_rx: mpsc::Receiver<(u64, PtyEvent)>, // confirm exact PtyEvent channel shape in the example
}

impl TerminalPane {
    pub fn spawn(ctx: &eframe::egui::Context) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let backend = TerminalBackend::new(0, ctx.clone(), tx, BackendSettings::default())?;
        Ok(Self { backend, event_rx: rx })
    }

    /// Call every frame: drains pending PTY events into the backend.
    /// Confirm against the example whether this is manual or handled
    /// internally by TerminalView — don't assume.
    pub fn pump(&mut self) {
        while let Ok((_id, event)) = self.event_rx.try_recv() {
            self.backend.process_event(event); // confirm exact method name
        }
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.add(TerminalView::new(ui, &mut self.backend)); // confirm builder methods (focus, size) in example
    }
}
```

## 4. The "leave terminal mode" key

Use `egui_term`'s own custom-bindings mechanism (see `examples/custom_bindings/main.rs`
and the `generate_bindings!` macro / `Binding` / `KeyboardBinding` types) to bind one
dedicated key combo to a Mindforge-specific action, rather than intercepting keys
before they reach the widget. As before: **don't use plain `Esc`** — real programs
run inside the terminal (`vim`, `less`) use `Esc` themselves. Pick something like
`Ctrl+Shift+Escape` or another combo unlikely to collide with normal shell/editor use,
and confirm in the example how a custom binding communicates "an app-level action
happened" back out to the surrounding Mindforge code (likely via `BackendCommand` or
a callback — check the example, don't assume).

## 5. Theming

`ColorPalette` / `TerminalTheme` (confirm construction in `examples/themes/main.rs`)
let the terminal pane match whatever `ThemeKind` is currently active in Mindforge —
map `Theme::bg`/`text`/`accent`/etc. into the palette so switching `:theme` also
re-themes the terminal pane consistently, rather than the terminal looking like a
foreign widget dropped into the app.

## Build order

1. Confirm step 0 (version compatibility) before anything else.
2. Get the `examples/full_screen` example running standalone, unmodified, to confirm
   the crate itself works in this environment before integrating anything.
3. Wire `Mode::Terminal` + `:term`, following the real `commands.rs`/`Mode` patterns.
4. Add the custom exit-key binding (section 4).
5. Add theming (section 5).
6. Run `vim` inside it — same real test as before: full-screen redraws, cursor
   movement, `Ctrl-C`/`Ctrl-D` all need to behave normally.

## Definition of done

Same as before: `:term` opens a real shell with working `ls`/`cd`/`git status`,
colored output renders correctly, `vim`/`htop` work inside the pane, the session
persists across mode switches, terminal input never triggers `:` commands, and none
of the existing `:` commands are affected.

---

## Appendix A: fallback — hand-rolled PTY + VT parsing

Only use this path if step 0 found a real version conflict with `egui_term` that
isn't worth resolving right now. This is the original approach: a separate
`terminal/` crate using `portable-pty` for PTY spawning and `vte` for ANSI parsing,
built entirely by hand with no egui dependency at all — so it has zero coupling to
Mindforge's egui version, at the cost of reimplementing what `alacritty_terminal`
already does well. It's a real, working starting point, just a rougher one: the ANSI
subset it handles is smaller (colors and cursor positioning only; screen/line
clearing are stubbed), so expect more `TODO`s to fill in with real-world usage
(`git status`, colored prompts) than the `egui_term` path needs.

### `terminal/Cargo.toml`
```toml
[package]
name = "terminal"
version = "0.1.0"
edition = "2021"

[dependencies]
portable-pty = "0.8"
vte = "0.13"
```

### `terminal/src/lib.rs`
```rust
mod pty; mod vt;
pub use vt::{Cell, Grid};

pub struct Term {
    pty: pty::PtySession,
    grid: vt::Grid,
}

impl Term {
    pub fn spawn(cols: u16, rows: u16) -> anyhow::Result<Self> {
        let pty = pty::PtySession::spawn(cols, rows)?;
        Ok(Self { pty, grid: vt::Grid::new(cols, rows) })
    }

    pub fn send_input(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.pty.write(bytes)
    }

    pub fn pump(&mut self) -> anyhow::Result<bool> {
        let mut changed = false;
        while let Some(chunk) = self.pty.try_read()? {
            self.grid.feed(&chunk);
            changed = true;
        }
        Ok(changed)
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.pty.resize(cols, rows)?;
        self.grid.resize(cols, rows);
        Ok(())
    }

    pub fn grid(&self) -> &Grid { &self.grid }
    pub fn is_alive(&mut self) -> bool { self.pty.is_alive() }
}
```

### `terminal/src/pty.rs`
```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize, Child, MasterPty};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    child: Box<dyn Child + Send + Sync>,
}

impl PtySession {
    pub fn spawn(cols: u16, rows: u16) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) { "powershell.exe".into() } else { "/bin/bash".into() }
        });
        let cmd = CommandBuilder::new(shell);
        let child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => { if tx.send(buf[..n].to_vec()).is_err() { break; } }
                    Err(_) => break,
                }
            }
        });

        Ok(Self { master: pair.master, writer, rx, child })
    }

    pub fn write(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.writer.write_all(bytes)?;
        Ok(())
    }

    pub fn try_read(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        match self.rx.try_recv() {
            Ok(chunk) => Ok(Some(chunk)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Ok(None),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}
```

### `terminal/src/vt.rs`
```rust
use vte::{Params, Parser, Perform};

#[derive(Clone, Copy, Default)]
pub struct Cell { pub ch: char, pub fg: (u8,u8,u8), pub bg: (u8,u8,u8), pub bold: bool }

pub struct Grid {
    cells: Vec<Vec<Cell>>,
    cursor: (usize, usize),
    cols: u16, rows: u16,
    parser: Parser,
    cur_fg: (u8,u8,u8),
    cur_bg: (u8,u8,u8),
}

impl Grid {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cells: vec![vec![Cell::default(); cols as usize]; rows as usize],
            cursor: (0, 0), cols, rows,
            parser: Parser::new(),
            cur_fg: (220, 220, 220), cur_bg: (0, 0, 0),
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        let mut performer = GridPerformer { grid: self };
        for &b in bytes {
            performer.grid.parser.advance(&mut performer, b);
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols; self.rows = rows;
        self.cells = vec![vec![Cell::default(); cols as usize]; rows as usize];
        self.cursor = (0, 0);
    }

    pub fn rows(&self) -> &[Vec<Cell>] { &self.cells }
    pub fn cursor(&self) -> (usize, usize) { self.cursor }
}

struct GridPerformer<'a> { grid: &'a mut Grid }

impl<'a> Perform for GridPerformer<'a> {
    fn print(&mut self, c: char) {
        let (col, row) = self.grid.cursor;
        if let Some(cell) = self.grid.cells.get_mut(row).and_then(|r| r.get_mut(col)) {
            *cell = Cell { ch: c, fg: self.grid.cur_fg, bg: self.grid.cur_bg, bold: false };
        }
        self.grid.cursor.0 = (col + 1).min(self.grid.cols as usize - 1);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => { self.grid.cursor.1 = (self.grid.cursor.1 + 1).min(self.grid.rows as usize - 1); self.grid.cursor.0 = 0; }
            b'\r' => { self.grid.cursor.0 = 0; }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'm' => self.apply_sgr(params),
            'H' | 'f' => {
                let mut it = params.iter();
                let row = it.next().and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize - 1;
                let col = it.next().and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize - 1;
                self.grid.cursor = (col.min(self.grid.cols as usize - 1), row.min(self.grid.rows as usize - 1));
            }
            'J' => { /* TODO: clear screen variants */ }
            'K' => { /* TODO: clear line variants */ }
            _ => {}
        }
    }
}

impl<'a> GridPerformer<'a> {
    fn apply_sgr(&mut self, params: &Params) {
        for p in params.iter() {
            match p.first().copied().unwrap_or(0) {
                0 => { self.grid.cur_fg = (220,220,220); self.grid.cur_bg = (0,0,0); }
                30..=37 => self.grid.cur_fg = ansi_16(p[0] as u8 - 30),
                40..=47 => self.grid.cur_bg = ansi_16(p[0] as u8 - 40),
                _ => {}
            }
        }
    }
}

fn ansi_16(n: u8) -> (u8, u8, u8) {
    const PALETTE: [(u8,u8,u8); 8] = [
        (0,0,0), (205,49,49), (13,188,121), (229,229,16),
        (36,114,200), (188,63,188), (17,168,205), (229,229,229),
    ];
    PALETTE[n as usize % 8]
}
```

## Rules for the agent (both paths)

- Do not modify `execute_command`'s existing arms beyond adding one new `"term"` arm.
- Try `egui_term` first (sections 1-5). Only drop to Appendix A if step 0's version
  check genuinely fails, or if `egui_term` proves broken/unmaintained-feeling in
  practice after actually trying it — don't switch paths on a hunch, switch on a
  concrete blocker.
- Read the real example files before writing integration code; do not invent method
  names that weren't confirmed against them.
- After each build step, give a 3-line summary of what changed and what to try.
