🎯 Prompt for Agent: Fix Multi-Line Selection Rendering Defect
Context:
There is a critical visual bug in how the MindForge editor renders text selections (specifically when using v + j/k motions). The current implementation creates a disjointed, "blocky" appearance that breaks the premium aesthetic of the application.
The Visual Defect (What to Fix):
"Stacked Blocks" Instead of Continuous Flow:
When selecting across multiple lines, the background highlight renders as separate, independent rectangles for each row.
There are visible horizontal seams or gaps between the rows where the background color resets or misaligns. It looks like individual line highlights stacked on top of each other, rather than one unified selection shape.
Right-Edge Bleed (Whitespace Highlighting):
For intermediate lines in a multi-line selection, the highlight extends all the way to the right edge of the text container (or viewport), even if the actual text content ends much earlier.
This creates a heavy "solid bar" effect that obscures the true length of the line and makes the selection look bloated. The highlight should stop exactly at the last character of the line.
Sub-Pixel Seam Artifacts:
Due to the per-line rendering approach, there are often 1px vertical gaps or overlaps between rows caused by line-height calculations or anti-aliasing. These seams break the illusion of a single selection.
Cursor Disconnect:
The cursor sometimes appears visually separated from the selection block, or the selection doesn't extend fully under the cursor character.
Required Behavior (The Fix):
Seamless Vertical Fusion:
Eliminate all visible horizontal seams between selected rows. The selection must look like one continuous vertical shape.
If drawing per-line rects is necessary, implement a 0.5px–1px vertical overlap between rows to hide sub-pixel gaps. Alternatively, calculate a single merged path/mesh for the entire selection.
Text-Boundary Precision:
The selection background must stop exactly at the right edge of the last selected character on each line. Do not extend the highlight into empty whitespace beyond the text content.
Only the start line and end line should have partial-width highlights; intermediate lines should highlight only the actual text content (or up to the wrap point).
Cursor Integration:
Ensure the selection background extends fully under the cursor character. The cursor should always render on top of the selection layer with no visual separation.
Goal:
Transform the selection from "disjointed stacked blocks" to a "seamless, precise geometric shape." When a user selects text, it should look like a single, cohesive unit that respects the actual boundaries of the text content.
