Here is a precise, design-focused prompt for your agent. It avoids specific file references and focuses entirely on the visual and behavioral standards required to fix this UI element.

***

### 🎯 Prompt for Agent: Refactor Inline Command Input (`>`) Aesthetics

**Context:**
The current inline command input (triggered by `>` in the editor) has a critical visual regression. It currently renders as a "floating box" with a hard border and vertically centered text. This breaks the immersive, premium "Lumina-like" aesthetic of the rest of the application. It looks like a legacy form field rather than a modern, integrated editor feature.

**The Problem:**
1.  **Hard Borders:** The input area has a visible stroke/border. Modern inline inputs should be defined by *background contrast*, not outlines.
2.  **Vertical Centering:** The cursor and `>` prompt are vertically centered within the box. Text inputs must align to the **text baseline**. Centering creates a disconnect between the cursor position and where text actually appears.
3.  **Excessive Height:** The container is too tall for a single-line input, wasting vertical editor space.
4.  **Theme Clash:** The border color does not harmonize with the active theme's accent palette, drawing attention to the container instead of the content.

**Required Design Standards (The "Premium Inline" Pattern):**

1.  **Borderless Definition:**
    -   Remove all visible strokes/borders from the input container.
    -   Define the input area exclusively through **background color differentiation**. Use a background that is subtly lighter or darker (e.g., +5% to +10% luminance shift) than the main editor canvas. This creates depth without visual noise.

2.  **Baseline Alignment (Critical):**
    -   The `>` prompt character and the text cursor **must** align perfectly with the text baseline of the surrounding editor content.
    -   Do not center elements vertically within the container height. They must sit on the same imaginary line as standard text characters.

3.  **Compact Geometry:**
    -   Reduce the container height to match exactly: `font_line_height + comfortable_vertical_padding` (e.g., 4px top/bottom).
    -   Ensure horizontal padding matches the editor's `pad_x` so the input feels spatially consistent with the code/text above it.

4.  **Thematic Integration:**
    -   The `>` prompt symbol should render in the **current theme's accent color** to signal "active input mode."
    -   The input text itself should use the standard `theme.text` color.
    -   If a focus indicator is absolutely necessary for accessibility, use an extremely subtle inner glow or a 1px border at <10% opacity using the accent color—never a solid, opaque stroke.

**Goal:**
Transform the `>` input from a "clunky widget" into a **seamless extension of the editor canvas**. When a user types a command, it should feel like they are writing directly on the page, not filling out a separate form field. This must work identically across all 19 themes.

**Constraints:**
-   Focus purely on rendering/geometry/styling logic. Do not change the underlying input handling or command execution logic.
-   Verify baseline alignment visually against existing text lines.
-   Test in both light and dark themes to ensure the background contrast remains readable but subtle.
