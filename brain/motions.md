# Vim Motions & Text Objects

Quick reference for moving around and selecting text in Vim mode.

---

## Basic Movement

| Key | What it does |
|-----|-------------|
| `h` | Move left one character |
| `l` | Move right one character |
| `j` | Move down one line |
| `k` | Move up one line |
| `w` | Jump forward to next word |
| `b` | Jump backward to previous word |
| `0` | Go to start of line |
| `$` | Go to end of line |
| `gg` | Go to very top of the note |
| `G` | Go to very bottom of the note |

---

## Text Objects — `vi'`, `va"`, `vi[` etc.

Text objects let you select **by meaning**, not by counting characters.

The pattern is always: **operator + scope + delimiter**

```
v  i  '
│  │  └── what to select around ('  "  `  (  [  {  w)
│  └───── i = inner (without the delimiters)
└──────── v = select (visual mode)
```

### Scope

| Scope | Meaning |
|-------|---------|
| `i` | **inner** — content only, no surrounding delimiters |
| `a` | **around** — content + the delimiters themselves |

### Delimiters

| Delimiter | Selects |
|-----------|---------|
| `'` | Single quotes `'...'` |
| `"` | Double quotes `"..."` |
| `` ` `` | Backticks `` `...` `` |
| `(` or `)` | Parentheses `(...)` |
| `[` or `]` | Square brackets `[...]` |
| `{` or `}` | Curly braces `{...}` |
| `w` | Word under cursor |

### Examples

| Command | Text: `say 'hello world'` | Result |
|---------|--------------------------|--------|
| `vi'`   | cursor anywhere inside   | selects `hello world` |
| `va'`   | cursor anywhere inside   | selects `'hello world'` |
| `vi"`   | cursor inside `"..."`    | selects content inside |
| `vi(`   | cursor inside `(...)`    | selects content inside |
| `viw`   | cursor on any word       | selects the word |

> **Tip:** You do NOT need to position cursor exactly on the quote.
> Place it **anywhere inside** `'hello world'` and `vi'` finds the pair automatically.

---

## Operators — acting on text objects

Instead of `v` (select), you can use an operator directly:

| Command | Action |
|---------|--------|
| `vi'`   | Select inside quotes (visual) |
| `di'`   | Delete inside quotes |
| `ci'`   | Change inside quotes (deletes + enter Insert mode) |
| `yi'`   | Yank (copy) inside quotes |

These work with ANY delimiter above.

---

## Search Motion

| Key | Action |
|-----|--------|
| `/text` | Search forward for `text` |
| `?text` | Search backward for `text` |
| `n` | Jump to next match |
| `N` | Jump to previous match |

---

## Visual Mode

Press `v` to enter Visual mode — then use any motion to expand the selection.

| Key | Action |
|-----|--------|
| `v` | Start character selection |
| `V` | Start line selection |
| `Esc` | Cancel and return to Normal |

After selecting, press `d` to delete, `y` to copy, or `c` to change.
