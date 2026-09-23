Here is a precise, self-contained prompt for your agent. It defines the "Lumina" aesthetic from scratch using design principles rather than references, and explicitly protects your existing layout gaps.

***

### 🎯 Prompt for AI Agent: MindForge UI Refactor to "Modern Workspace" Aesthetic

**Context:**
We are refactoring the **MindForge** Egui application to match a specific "Modern Knowledge Workspace" visual style (codenamed "Lumina"). The goal is to transform the UI from a "rigid IDE" look to a "soft, premium workspace" feel.

**️ CRITICAL CONSTRAINT: PRESERVE LAYOUT GAPS**
*   **DO NOT** remove, shrink, or merge the spacing/gaps between UI components (sidebar, editor, title bar).
*   **DO NOT** change the panel docking logic or resize handles.
*   The structural grid and breathing room between panels must remain **exactly as they are**. We are only changing *surface styling* (colors, fonts, shapes), not *geometry*.

**Design System Definition ("Lumina Style"):**
Since you do not have external references, adhere strictly to these rules:

1.  **Color Palette (Deep Slate & Soft Accents):**
    *   **Backgrounds:** Replace pure blacks/grays with deep slate/navy tones (e.g., `#0f1115` for main bg, `#16181d` for sidebars).
    *   **Text:** Never use pure white (`#FFFFFF`). Use off-white (`#e2e8f0`) for primary text and muted slate (`#94a3b8`) for secondary text/labels.
    *   **Accents:** Replace harsh yellow selections with a soft, electric purple/pink (`#d946ef`) or warm amber (`#d4a373`) at low opacity (15-20%) for backgrounds.

2.  **Typography Hierarchy:**
    *   **UI Elements (Sidebar, Tabs, Headers):** Switch from Monospace to a clean **Sans-Serif** font (Inter, Roboto, or System UI). This distinguishes "interface" from "code."
    *   **Editor Content:** Keep Monospace for code only.
    *   **Line Numbers:** Reduce opacity to 40% so they recede visually.

3.  **Component Styling Rules:**
    *   **Selection States:** Instead of solid block highlights, use **rounded rectangles** (`corner_radius = 4.0`) with a soft tinted background. Shrink the highlight rect by 2-4px from the edges so it "floats" inside the list item.
    *   **Dividers:** Remove hard 1px border lines between panels. Rely on **background color contrast** (e.g., sidebar is 5% lighter than editor) to define space.
    *   **Tabs:** Style tabs as "floating chips" rather than a connected strip. Give them rounded corners and distinct separation.
    *   **Cursor:** Change from Block cursor to a **thin vertical beam** with a subtle glow effect.

4.  **Egui Implementation Specifics:**
    *   Use `ui.painter().rect_filled()` with `shrink()` for selection highlights to achieve the "floating pill" look.
    *   Apply `WindowFrame::shadow` to floating elements (modals, menus) to create depth.
    *   Ensure all interactive elements have a smooth transition state (if possible in Egui) or at least a distinct hover state that doesn't rely on brightness alone.

**Execution Plan:**
1.  Update the global `Style` object in Egui with the new color palette and font families.
2.  Refactor the Sidebar list rendering to use rounded, inset selection highlights.
3.  Adjust the Tab Bar rendering to look like discrete chips.
4.  Verify that all original panel gaps and resize handles are untouched.

***

### 💡 Why This Prompt Works
*   **Self-Contained Definition:** It doesn't say "make it like Lumina." It says "make it Deep Slate, Sans-Serif UI, Rounded Selections." The agent can execute this without seeing a screenshot.
*   **Explicit Guardrails:** The "CRITICAL CONSTRAINT" section uses bold caps and negative commands ("DO NOT") to prevent the agent from "fixing" the gaps it might perceive as inefficiencies.
*   **Technical Translation:** It translates abstract design goals ("premium feel") into concrete Egui API calls (`rect_filled`, `shrink`, `WindowFrame::shadow`).



Here is the comprehensive, consolidated prompt for your agent. It combines the previous "Lumina/Porcelain" styling instructions with your new specific fixes for the settings icon, hover states, and list spacing.

***

### 🎯 Prompt for AI Agent: MindForge UI Refactor to "Porcelain" Workspace Aesthetic

**Context:**
We are refactoring the **MindForge** Egui application to match a specific "Modern Knowledge Workspace" visual style (codenamed "Porcelain"). The goal is to transform the UI from a "rigid IDE" look to a "soft, premium workspace" feel.

**️ CRITICAL CONSTRAINT: PRESERVE LAYOUT GEOMETRY**
*   **DO NOT** remove, shrink, or merge the spacing/gaps between UI components (sidebar, editor, title bar).
*   **DO NOT** change the panel docking logic or resize handles.
*   **DO NOT** change the height of the window or main panels.
*   We are only changing *surface styling* (colors, fonts, shapes, padding), not *structural geometry*.

---

### 🛠️ Specific Fixes Required (Priority)

1.  **Settings Icon Theme Mismatch:**
    *   The gear/settings icon in the bottom-left sidebar is currently stark black/white.
    *   **Fix:** Recolor it to match the secondary text color (`#64748b` or similar slate gray). It should feel integrated into the light theme, not like a high-contrast sticker.

2.  **Sidebar Note Hover State:**
    *   Currently, hovering over a note in the sidebar turns the background black. This is too harsh for the light "Porcelain" theme.
    *   **Fix:** Change the hover background to a soft, transparent gray (e.g., `rgba(0, 0, 0, 0.05)` or `#F1F5F9`). Text color should remain dark gray, not white.

3.  **Note List Item Height:**
    *   The vertical height of individual note items in the sidebar is too small/cramped.
    *   **Fix:** Increase the minimum height of each list item row by `4px` to `8px`. Ensure vertical centering of text is maintained. The list should feel breathable, not dense.

---

### 🎨 General "Porcelain" Design System Rules

**1. Color Palette (Light & Airy):**
*   **Backgrounds:** Use soft whites and light grays. Main canvas: `#FFFFFF`, Sidebar: `#FAFAFA` or `#F8FAFC`.
*   **Text:** Never pure black. Primary text: `#1E293B` (Slate 800). Secondary text: `#64748B` (Slate 500).
*   **Accents:** Use the specific Cyan/Blue seen in the screenshots (`#0EA5E9` or similar) for active states, but keep it soft.

**2. Typography Hierarchy:**
*   **UI Elements (Sidebar, Tabs, Headers):** Switch from Monospace to a clean **Sans-Serif** font (Inter, Roboto, or System UI).
*   **Editor Content:** Keep Monospace for code only.
*   **Line Numbers:** Reduce opacity to 40% so they recede visually.

**3. Component Styling Rules:**
*   **Sidebar Selection ("Floating Pill"):**
    *   Do NOT use a full-width block highlight.
    *   Use a rounded rectangle (`corner_radius = 6px`) that is inset by `4px` on left/right margins.
    *   Background should be a soft tint (e.g., `rgba(14, 165, 233, 0.1)`).
*   **Tabs:**
    *   Break the continuous top strip into discrete "chips".
    *   Add gaps between tabs.
    *   Active tab: White background, subtle shadow. Inactive tab: Light gray background.
*   **Settings Modal:**
    *   Remove zebra-striping from lists (like Shortcuts). Use uniform backgrounds with hover highlights only.
    *   Selected cards (like "Candle" caret) should have a subtle background tint + border, not just a border.

**4. Egui Implementation Specifics:**
*   Use `ui.painter().rect_filled()` with `shrink()` for selection highlights.
*   Apply `WindowFrame::shadow` to floating elements (modals, menus).
*   Ensure all interactive elements have a distinct hover state that doesn't rely on high-contrast inversion (no black-on-white flipping).

**Execution Plan:**
1.  Fix the Settings Icon color immediately.
2.  Adjust Sidebar row height and hover colors.
3.  Update global Style object with Porcelain palette and Sans-Serif UI font.
4.  Refactor Sidebar list rendering for "floating pill" selection.
5.  Refactor Tab Bar for discrete chips.
6.  Verify all original panel gaps and resize handles are untouched.
