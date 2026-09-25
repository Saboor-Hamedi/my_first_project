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
| `p` | Paragraph block (bounded by blank lines) |

### Examples

| Command | Text / Cursor Location | Result |
|---------|------------------------|--------|
| `vi'`   | cursor anywhere inside `'...'` | selects `hello world` |
| `va'`   | cursor anywhere inside `'...'` | selects `'hello world'` |
| `vi"`   | cursor inside `"..."`          | selects content inside |
| `vi(`   | cursor inside `(...)`          | selects content inside |
| `viw`   | cursor on any word             | selects the word |
| `vip`   | cursor anywhere in paragraph   | selects the entire inner paragraph |
| `vap`   | cursor anywhere in paragraph   | selects the paragraph + adjacent blank line |

> **Tip:** You do NOT need to position cursor at the very start of a line or quote.
> Place it **anywhere inside** (beginning, center, or end) and the text object resolves automatically.

---

## Paragraph Text Objects — `vip`, `vap`, `yip`, `dap`, `cip`

Paragraph text objects operate on entire blocks of text separated by blank lines. Your cursor can be anywhere inside the paragraph — **at the beginning, in the center, or at the end**:

| Command | Type | What it does |
|---------|------|-------------|
| `vip` | **Visual Inner Paragraph** | `v` + `i` + `p`<br>Visually selects the whole inner paragraph where your cursor is, strictly ignoring surrounding empty lines. Promotes visual selection to `VisualLine` mode so expanding up/down behaves line-by-line. |
| `vap` | **Visual Around Paragraph** | `v` + `a` + `p`<br>Visually selects around the paragraph, including the paragraph text and the blank line next to it (trailing blank line by default, or preceding if at EOF). |
| `yip` | **Yank Inner Paragraph** | `y` + `i` + `p`<br>Automatically yanks (copies) the entire paragraph as linewise content into the register **without needing to visually select it first**. Pressing `p` afterwards pastes it as clean new lines. |
| `dap` | **Delete Around Paragraph** | `d` + `a` + `p`<br>Automatically deletes the entire paragraph along with its extra blank line, preventing double-gap whitespace residue. |
| `cip` | **Change Inner Paragraph** | `c` + `i` + `p`<br>Deletes the entire paragraph and immediately drops you into **Insert mode** on that line, ready to type the replacement paragraph. |
| `dip` | **Delete Inner Paragraph** | `d` + `i` + `p`<br>Deletes only the paragraph text, preserving the blank line separators above and below it. |

### Visual Breakdown of `vip` vs `vap` vs `dap`

```markdown
Paragraph 1 line 1.
Paragraph 1 line 2.
                       <─── Blank line

Paragraph 2 line 1.   <─── Cursor can be anywhere on line 1, line 2, or line 3
Paragraph 2 line 2.
Paragraph 2 line 3.
                       <─── Blank line next to Paragraph 2

Paragraph 3 line 1.
```

- **`vip` on Paragraph 2:**
  Selects lines 1, 2, and 3 of Paragraph 2. Surrounding blank lines are left untouched.
- **`vap` on Paragraph 2:**
  Selects lines 1, 2, and 3 of Paragraph 2 **plus** the blank line immediately below it.
- **`yip` on Paragraph 2:**
  Copies lines 1, 2, and 3 to clipboard/register. Buffer remains unchanged.
- **`dap` on Paragraph 2:**
  Removes lines 1, 2, 3 and the trailing blank line. Paragraph 1 and Paragraph 3 remain separated by exactly one blank line.
- **`cip` on Paragraph 2:**
  Removes lines 1, 2, 3 and enters Insert mode at Paragraph 2's slot.

---

## Operators — acting on text objects

Instead of `v` (select), you can use an operator directly:

| Command | Action |
|---------|--------|
| `vi'`   | Select inside quotes (visual) |
| `di'`   | Delete inside quotes |
| `ci'`   | Change inside quotes (deletes + enter Insert mode) |
| `yi'`   | Yank (copy) inside quotes |
| `vip`   | Select entire paragraph (visual) |
| `dap`   | Delete entire paragraph + blank line |
| `yip`   | Yank entire paragraph directly |
| `cip`   | Delete paragraph and start typing |

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
