Here are two clear, actionable prompts you can give to your coding agent or use as tickets.

### Prompt 1: Fix `Ctrl+A` Selection Background Gaps
**Subject:** Fix CSS styling for full-text selection background continuity

> "Currently, when I use `Ctrl+A` to select all text in the editor, the selection background color has visible gaps between lines or blocks. It looks like there is unwanted margin or padding breaking the visual continuity of the selection highlight.
>
> **Requirement:**
> Please update the CSS for the text container and its children (lines/blocks) so that the selection background (`::selection` or active state class) appears as a single, continuous block of color from the first character to the last, with no vertical gaps. Ensure line-height, margins, and padding are adjusted so the highlight touches edge-to-edge vertically."

***

### Prompt 2: Fix Vim-style `k` Navigation at Bottom of File
**Subject:** Fix cursor navigation logic when at the end of the document buffer

> "There is a navigation bug when the cursor is positioned at the very bottom of the document (specifically after a trailing newline/empty line created by `Ctrl+A` or Enter).
>
> **Current Behavior:**
> When the caret is on this final empty line, pressing `k` (up) does nothing. The user is forced to press `h` (left) first to move to the end of the previous line's content before `k` will work again.
>
> **Requirement:**
> Fix the keyboard event handler for `k` (Up Arrow). It should correctly calculate the target row index even when the current row is an empty trailing line. Pressing `k` from the absolute bottom of the file must immediately move the cursor up to the last line containing text, without requiring an intermediate horizontal movement."