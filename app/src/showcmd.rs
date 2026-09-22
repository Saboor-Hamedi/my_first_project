//! Standalone ShowCmd Keystroke & Command HUD Component.
//! Tracks pending/executed Vim commands, search queries, and command-line inputs,
//! rendering them as a clean, borderless floating capsule card.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Stroke};

/// Maximum characters to display — beyond this the text is truncated.
const MAX_DISPLAY_CHARS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowCmdKind {
    Keystroke,
    Command,
    Search,
    Visual,
}

#[derive(Clone, Debug)]
pub struct ShowCmdState {
    pub enabled: bool,
    pub text: String,
    pub kind: ShowCmdKind,
    pub last_change_time: f64,
    pub is_pending: bool,
}

impl Default for ShowCmdState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ShowCmdState {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            text: String::new(),
            kind: ShowCmdKind::Keystroke,
            last_change_time: 0.0,
            is_pending: false,
        }
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    /// Truncates text to MAX_DISPLAY_CHARS chars, appending "…" if needed.
    fn truncate(text: &str) -> String {
        let char_count = text.chars().count();
        if char_count <= MAX_DISPLAY_CHARS {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(MAX_DISPLAY_CHARS).collect();
            format!("{}…", truncated)
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Sets a pending partial command (e.g. "d", "40j", "ci", "\"a").
    /// Pass `is_visual = true` when called from Visual mode (sets VIS badge).
    pub fn set_pending(&mut self, text: &str, now: f64) {
        if !self.enabled {
            self.text.clear();
            return;
        }
        self.text = Self::truncate(text);
        // Infer kind from the text itself: visual pending starts with 'v'/'V'/'<'
        self.kind = infer_kind(text);
        self.last_change_time = now;
        self.is_pending = true;
    }

    /// Live-tracks command mode input (e.g. ":w", ":delete", ":set showcmd").
    pub fn set_command(&mut self, cmd: &str, now: f64) {
        if !self.enabled {
            self.text.clear();
            return;
        }
        // Truncate just the cmd part so the leading ':' is always visible
        let display_cmd = if cmd.chars().count() > MAX_DISPLAY_CHARS - 1 {
            let t: String = cmd.chars().take(MAX_DISPLAY_CHARS - 1).collect();
            format!("{}…", t)
        } else {
            cmd.to_string()
        };
        self.text = format!(":{}", display_cmd);
        self.kind = ShowCmdKind::Command;
        self.last_change_time = now;
        self.is_pending = true;
    }

    /// Live-tracks search query input (e.g. "/pattern", "?pattern").
    pub fn set_search(&mut self, symbol: &str, query: &str, now: f64) {
        if !self.enabled {
            self.text.clear();
            return;
        }
        let display_query = if query.chars().count() > MAX_DISPLAY_CHARS - 1 {
            let t: String = query.chars().take(MAX_DISPLAY_CHARS - 1).collect();
            format!("{}…", t)
        } else {
            query.to_string()
        };
        self.text = format!("{}{}", symbol, display_query);
        self.kind = ShowCmdKind::Search;
        self.last_change_time = now;
        self.is_pending = true;
    }

    /// Flashes a completed action (e.g. "ci\"", "vi\"", "40j", "dw", "dd", "x", "p").
    pub fn record_action(&mut self, text: &str, now: f64) {
        if !self.enabled {
            self.text.clear();
            return;
        }
        self.text = Self::truncate(text);
        self.kind = infer_kind(text);
        self.last_change_time = now;
        self.is_pending = false;
    }

    /// Clears the card display immediately (e.g. on Escape).
    pub fn clear(&mut self) {
        self.text.clear();
        self.is_pending = false;
    }

    /// Ticks timeouts: 2.0s for pending keys/search/command, 1.2s for completed actions.
    pub fn update(&mut self, now: f64) {
        if self.text.is_empty() {
            return;
        }
        let timeout = if self.is_pending { 2.0 } else { 1.2 };
        if (now - self.last_change_time) > timeout {
            self.text.clear();
            self.is_pending = false;
        }
    }

    /// Returns whether the card is currently visible within its timeout window.
    #[allow(dead_code)]
    pub fn is_visible(&self, now: f64) -> bool {
        if !self.enabled || self.text.is_empty() {
            return false;
        }
        let timeout = if self.is_pending { 2.0 } else { 1.2 };
        (now - self.last_change_time) <= timeout
    }

    /// Renders the card anchored at `anchor_bottom_right` (bottom-right corner of card).
    /// Borderless capsule pill — no visible stroke, no shimmer line, single soft shadow.
    pub fn render_card(
        &self,
        painter: &egui::Painter,
        anchor_bottom_right: Pos2,
        _accent: Color32,
        now: f64,
    ) -> Option<Rect> {
        if !self.enabled || self.text.is_empty() {
            return None;
        }

        let timeout = if self.is_pending { 2.0 } else { 1.2 };
        let elapsed = now - self.last_change_time;
        if elapsed > timeout {
            return None;
        }

        // Smooth fade-out over the last 0.4 s
        let alpha = {
            let fade = 0.4f64;
            if elapsed > timeout - fade {
                let frac = (timeout - elapsed) / fade;
                (frac.clamp(0.0, 1.0) * 255.0) as u8
            } else {
                255u8
            }
        };

        // ── Per-mode palette ─────────────────────────────────────────────────
        let (badge, main_color) = match self.kind {
            ShowCmdKind::Command   => ("CMD",  Color32::from_rgb(100, 200, 255)),
            ShowCmdKind::Search    => ("FIND", Color32::from_rgb(255, 215, 60)),
            ShowCmdKind::Visual    => ("VIS",  Color32::from_rgb(190, 130, 255)),
            ShowCmdKind::Keystroke => ("VIM",  Color32::WHITE),
        };

        let accent_a = Color32::from_rgba_unmultiplied(
            main_color.r(), main_color.g(), main_color.b(), alpha,
        );
        let muted_a  = Color32::from_rgba_unmultiplied(90, 100, 120, alpha);
        let white_a  = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

        // ── Text layout (badge · command) ────────────────────────────────────
        let mut job = egui::text::LayoutJob::default();

        // Badge label — small, accent-coloured
        job.append(
            badge,
            0.0,
            egui::text::TextFormat {
                font_id: FontId::monospace(10.0),
                color: accent_a,
                ..Default::default()
            },
        );

        // Separator " · "
        job.append(
            " · ",
            0.0,
            egui::text::TextFormat {
                font_id: FontId::monospace(11.0),
                color: muted_a,
                ..Default::default()
            },
        );

        // Command text — larger, bright
        job.append(
            &self.text,
            0.0,
            egui::text::TextFormat {
                font_id: FontId::monospace(15.0),
                color: white_a,
                ..Default::default()
            },
        );

        let galley = painter.layout_job(job);

        // ── Card geometry ────────────────────────────────────────────────────
        let card_h = 42.0f32;
        let h_pad  = 20.0f32;
        let card_w = (galley.size().x + h_pad * 2.0).max(120.0);

        let card_rect = Rect::from_min_max(
            Pos2::new(anchor_bottom_right.x - card_w, anchor_bottom_right.y - card_h),
            anchor_bottom_right,
        );

        // ── Single soft diffuse shadow — NOT a border ring ───────────────────
        // One large, very transparent rect gives a glow without looking like a stroke.
        painter.rect(
            card_rect.expand(6.0),
            card_h * 0.5 + 6.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, (alpha as f32 * 0.28) as u8),
            Stroke::NONE,
            egui::StrokeKind::Outside,
        );

        // ── Frosted obsidian glass body — absolutely no border ───────────────
        painter.rect(
            card_rect,
            card_h * 0.5,          // full capsule pill
            Color32::from_rgba_unmultiplied(14, 16, 22, (alpha as f32 * 0.97) as u8),
            Stroke::NONE,           // zero stroke
            egui::StrokeKind::Inside,
        );

        // ── Galley centered vertically & horizontally ────────────────────────
        let text_x = card_rect.min.x + (card_rect.width()  - galley.size().x) * 0.5;
        let text_y = card_rect.min.y + (card_rect.height() - galley.size().y) * 0.5;
        painter.galley(Pos2::new(text_x, text_y), galley, Color32::WHITE);

        Some(card_rect)
    }
}

// ── Kind inference ────────────────────────────────────────────────────────────

/// Infers ShowCmdKind from the raw text string.
fn infer_kind(text: &str) -> ShowCmdKind {
    if text.starts_with(':') {
        ShowCmdKind::Command
    } else if text.starts_with('/') || text.starts_with('?') {
        ShowCmdKind::Search
    } else if text.starts_with('v') || text.starts_with('V') || text.starts_with('<') {
        ShowCmdKind::Visual
    } else {
        ShowCmdKind::Keystroke
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcmd_state_transitions() {
        let mut hud = ShowCmdState::new(true);
        assert!(hud.enabled);
        assert_eq!(hud.text, "");

        // Pending multi-key text object
        hud.set_pending("ci\"", 10.0);
        assert_eq!(hud.text, "ci\"");
        assert!(hud.is_pending);
        assert!(hud.is_visible(10.0));
        assert!(hud.is_visible(11.5)); // still within 2.0s

        // Still visible before timeout
        hud.update(11.0);
        assert_eq!(hud.text, "ci\"");

        // Expires after 2.0s
        hud.update(12.1);
        assert_eq!(hud.text, "");
        assert!(!hud.is_visible(12.1));

        // Completed action — 1.2s timeout
        hud.record_action("40j", 20.0);
        assert_eq!(hud.text, "40j");
        assert!(!hud.is_pending);
        assert!(hud.is_visible(20.5));
        hud.update(21.0);
        assert_eq!(hud.text, "40j");
        hud.update(21.3);
        assert_eq!(hud.text, "");

        // Command + search
        hud.set_command("w", 30.0);
        assert_eq!(hud.text, ":w");
        assert_eq!(hud.kind, ShowCmdKind::Command);

        hud.set_search("/", "needle", 35.0);
        assert_eq!(hud.text, "/needle");
        assert_eq!(hud.kind, ShowCmdKind::Search);

        // Visual from record_action
        hud.record_action("vi\"", 40.0);
        assert_eq!(hud.kind, ShowCmdKind::Visual);

        // Clear works immediately
        hud.set_pending("dw", 50.0);
        hud.clear();
        assert_eq!(hud.text, "");
        assert!(!hud.is_visible(50.0));

        // Disabled state never stores text
        let mut off = ShowCmdState::new(false);
        off.set_pending("ci\"", 1.0);
        assert_eq!(off.text, "");
        off.set_command("d", 1.0);
        assert_eq!(off.text, "");
        off.set_search("/", "pat", 1.0);
        assert_eq!(off.text, "");
        off.record_action("dw", 1.0);
        assert_eq!(off.text, "");
    }

    #[test]
    fn test_showcmd_truncation() {
        let mut hud = ShowCmdState::new(true);

        // Exactly at limit — no truncation
        let exact: String = "x".repeat(MAX_DISPLAY_CHARS);
        hud.record_action(&exact, 1.0);
        assert_eq!(hud.text.chars().count(), MAX_DISPLAY_CHARS);
        assert!(!hud.text.ends_with('…'));

        // One over limit — gets truncated
        let over: String = "x".repeat(MAX_DISPLAY_CHARS + 5);
        hud.record_action(&over, 2.0);
        assert!(hud.text.ends_with('…'));
        assert!(hud.text.chars().count() <= MAX_DISPLAY_CHARS + 1); // +1 for the ellipsis char

        // Command truncation keeps leading ':'
        let long_cmd: String = "w".repeat(MAX_DISPLAY_CHARS + 10);
        hud.set_command(&long_cmd, 3.0);
        assert!(hud.text.starts_with(':'));
        assert!(hud.text.ends_with('…'));

        // Search truncation keeps leading symbol
        let long_query: String = "a".repeat(MAX_DISPLAY_CHARS + 10);
        hud.set_search("/", &long_query, 4.0);
        assert!(hud.text.starts_with('/'));
        assert!(hud.text.ends_with('…'));
    }

    #[test]
    fn test_showcmd_kind_classification() {
        let mut hud = ShowCmdState::new(true);

        hud.record_action(":delete", 1.0);
        assert_eq!(hud.kind, ShowCmdKind::Command);

        hud.record_action("/pattern", 2.0);
        assert_eq!(hud.kind, ShowCmdKind::Search);

        hud.record_action("?back", 3.0);
        assert_eq!(hud.kind, ShowCmdKind::Search);

        hud.record_action("vi\"", 4.0);
        assert_eq!(hud.kind, ShowCmdKind::Visual);

        hud.record_action("VA", 5.0);
        assert_eq!(hud.kind, ShowCmdKind::Visual);

        hud.record_action("40j", 6.0);
        assert_eq!(hud.kind, ShowCmdKind::Keystroke);

        hud.record_action("ci\"", 7.0);
        assert_eq!(hud.kind, ShowCmdKind::Keystroke);

        hud.record_action("dw", 8.0);
        assert_eq!(hud.kind, ShowCmdKind::Keystroke);

        // set_pending also infers kind correctly
        hud.set_pending("vi", 9.0);
        assert_eq!(hud.kind, ShowCmdKind::Visual);

        hud.set_pending("d", 10.0);
        assert_eq!(hud.kind, ShowCmdKind::Keystroke);
    }

    #[test]
    fn test_showcmd_timeout_values() {
        let mut hud = ShowCmdState::new(true);

        // Pending: 2.0s timeout
        hud.set_pending("d", 100.0);
        assert!(hud.is_visible(101.9));
        assert!(!hud.is_visible(102.1));

        // Completed action: 1.2s timeout
        hud.record_action("dd", 200.0);
        assert!(hud.is_visible(201.1));
        assert!(!hud.is_visible(201.3));
    }
}
