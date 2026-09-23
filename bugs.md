🎯 Prompt for AI Agent: Refactor ShowCmd Card (Vim HUD)
Context:
The ShowCmd card (the floating indicator showing Vim modes like "VIM h") is currently breaking the light theme immersion. It looks like a dark-mode artifact pasted onto a light canvas. We need to restyle it to feel like a native, premium part of the "Porcelain" interface.
🛠️ Specific Fixes Required:
Theme Awareness (Critical):
The card must dynamically adapt to the active theme.
Light Mode: Background should be white (#FFFFFF) or very light gray (#F8FAFC) with a subtle border (#E2E8F0). Text should be dark slate (#1E293B).
Dark Mode: Keep the current dark aesthetic but refine the contrast.
Implementation: Bind the card's background/border colors to the global theme variables, do not hardcode dark hex values.
Visual Polish ("More Beautiful"):
Shape: Increase corner_radius to 8px or 10px. The current boxy look is too rigid.
Depth: Add a soft drop shadow (blur: 8px, offset: y=2, color: rgba(0,0,0,0.08) for light mode). This makes it float elegantly above the editor rather than sitting flat.
Padding: Increase internal padding. Current layout feels cramped. Aim for 6px vertical and 10px horizontal.
Internal Layout & Typography:
Mode Badge ("VIM"):
Give it a distinct background pill (e.g., rgba(14, 165, 233, 0.1) in light mode, text #0EA5E9).
Font: Uppercase, bold, tracking 0.05em, size 10px or 11px.
Command Text ("h"):
Font: Monospace (to align with code), size 14px or 16px (slightly larger than badge).
Color: Primary text color (#1E293B light / #E2E8F0 dark).
Alignment: Vertically center the badge and text perfectly. Add a 6px gap between them.
Animation & Interaction:
Add a subtle entrance animation (fade-in + slide-up 100ms) when the command appears.
Ensure the card auto-sizes to fit content but has a min_width so it doesn't jitter when typing short commands like "h" vs long ones like "dw".
Execution Plan:
Locate the ShowCmd widget rendering logic.
Replace hardcoded dark colors with theme-aware variables.
Update the Rect drawing to use rounded corners and shadow.
Adjust font sizes and padding for better breathing room.
Test in both Light and Dark modes to ensure readability.
