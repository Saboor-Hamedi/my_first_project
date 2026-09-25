//! Fuzzy search utilities and Command Palette actions for MindForge.

use crate::theme::ThemeKind;
use crate::sound::SoundProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    OpenNote(i64),
    OpenThemePicker,
    ApplyTheme(ThemeKind),
    ShowSoundPicker,
    ApplySoundProfile(SoundProfile),
    OpenSetting(crate::settings::SettingTab),
    ToggleSidebar,
    TogglePreview,
    ToggleAi,
    ToggleTerminal,
    ToggleZen,
    ToggleTitlebar,
    ToggleTabs,
    ToggleChecklist,
    CloseTab,
    NewNote,
    QuickSave,
    RenameNote,
    DeleteNote,
    ImportWorkspace,
    RunScan,
    ScanHistory,
    OpenHelp,
    SetLunaStyle(crate::lunaline::LunaStyle),
    SetLunaColor(crate::lunaline::LunaColorMode),
}

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub id: i64,
    pub title: String,
    pub snippet: String,
    pub score: i64,
    pub badge: String,
    pub icon: &'static str,
    pub action: PaletteAction,
}

impl SearchItem {
    pub fn for_note(id: i64, title: String, snippet: String, score: i64) -> Self {
        Self {
            id,
            title,
            snippet,
            score,
            badge: String::new(),
            icon: "📄",
            action: PaletteAction::OpenNote(id),
        }
    }
}

pub struct BuiltinCommand {
    pub title: &'static str,
    pub snippet: &'static str,
    pub badge: &'static str,
    pub icon: &'static str,
    pub action: PaletteAction,
}

pub const BUILTIN_COMMANDS: &[BuiltinCommand] = &[
    // --- Theme & Appearance Settings ---
    BuiltinCommand {
        title: "Settings: Color Theme",
        snippet: "Browse and live-preview all 19 visual themes",
        badge: ">theme",
        icon: "🎨",
        action: PaletteAction::OpenThemePicker,
    },
    BuiltinCommand {
        title: "Settings: Typing Sound Profile",
        snippet: "Preview and switch mechanical keyboard sound profiles live",
        badge: ">sound",
        icon: "🔊",
        action: PaletteAction::ShowSoundPicker,
    },
    BuiltinCommand {
        title: "Settings: Preferences & Appearance",
        snippet: "Customize active color palette, window opacity, and blur",
        badge: "Ctrl+,",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Theme),
    },
    BuiltinCommand {
        title: "Settings: Keyboard Shortcuts & Cheatsheet",
        snippet: "Review global shortcuts, editing commands, and markdown combos",
        badge: "F1",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Shortcuts),
    },
    BuiltinCommand {
        title: "Settings: Editor Mode (Vim / Hybrid)",
        snippet: "Switch between modal Vim motions and intuitive Hybrid writing",
        badge: "Ctrl+E",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::EditorMode),
    },
    BuiltinCommand {
        title: "Settings: Window Opacity & Transparency",
        snippet: "Adjust window opacity level from solid to translucent",
        badge: "Opacity",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Theme),
    },
    BuiltinCommand {
        title: "Settings: Window Blur Effect (Acrylic / Mica / Off)",
        snippet: "Configure Windows desktop acrylic or mica glass blur",
        badge: "Blur",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Theme),
    },

    // --- Carets & Typography Settings ---
    BuiltinCommand {
        title: "Settings: Carets & Cursor Styles",
        snippet: "Customize cursor animation, kind (Beam, Block, Neon), and width",
        badge: "Caret",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Carets),
    },
    BuiltinCommand {
        title: "Settings: Fonts & Monospace Typography",
        snippet: "Select custom font family, ligature rendering, and font size",
        badge: "Font",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Fonts),
    },

    // --- Audio, Keybindings & System Settings ---
    BuiltinCommand {
        title: "Settings: Mechanical Typing Audio & Switches",
        snippet: "Switch mechanical switch audio profiles (Thocky, Clacky, Silent)",
        badge: "Audio",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Sounds),
    },
    BuiltinCommand {
        title: "Settings: Custom Keybindings & Remapping",
        snippet: "Configure custom keybindings and inspect motion keymaps",
        badge: "Keymap",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Keybindings),
    },
    BuiltinCommand {
        title: "Settings: Vault Backup & Data Safety",
        snippet: "Configure automated SQLite snapshots and export paths",
        badge: "Backup",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Backup),
    },
    BuiltinCommand {
        title: "Settings: Check for App Updates",
        snippet: "Verify GitHub release packages and apply live updates",
        badge: "Updates",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Updates),
    },
    BuiltinCommand {
        title: "Settings: AI Engine & DeepSeek",
        snippet: "Configure DeepSeek API key and model parameters",
        badge: "Ctrl+Shift+I",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Ai),
    },

    // --- LunaLine Statusline Settings ---
    BuiltinCommand {
        title: "Settings: LunaLine Statusline",
        snippet: "Customize dock style (Pill, Powerline, Floating) & component toggles",
        badge: "LunaLine",
        icon: "⚙",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::LunaLine),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Style - Modern Pill Capsules",
        snippet: "Discrete capsules with subtle rounded pill background",
        badge: "Pill",
        icon: "🎨",
        action: PaletteAction::SetLunaStyle(crate::lunaline::LunaStyle::Pill),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Style - Neovim Powerline Chevrons",
        snippet: "Classic angled arrow chevrons connecting segments",
        badge: "Powerline",
        icon: "🎨",
        action: PaletteAction::SetLunaStyle(crate::lunaline::LunaStyle::Powerline),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Style - Floating Island Pill",
        snippet: "Detached glassmorphic statusline floating above edge",
        badge: "Floating",
        icon: "🎨",
        action: PaletteAction::SetLunaStyle(crate::lunaline::LunaStyle::Floating),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Style - Minimal Clean Typography",
        snippet: "Pure typographic statusline with subtle dot separators",
        badge: "Minimal",
        icon: "🎨",
        action: PaletteAction::SetLunaStyle(crate::lunaline::LunaStyle::Minimal),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Color Mode - Dynamic Accents",
        snippet: "Mode-reactive colors (Normal, Insert, Visual, Command)",
        badge: "Dynamic",
        icon: "🎨",
        action: PaletteAction::SetLunaColor(crate::lunaline::LunaColorMode::Dynamic),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Color Mode - Theme Accent",
        snippet: "Harmonizes directly with active Lumina theme accent",
        badge: "Accent",
        icon: "🎨",
        action: PaletteAction::SetLunaColor(crate::lunaline::LunaColorMode::ThemeAccent),
    },
    BuiltinCommand {
        title: "Settings: LunaLine Color Mode - Monochrome",
        snippet: "Stealth minimalist grayscale with maximum clarity",
        badge: "Mono",
        icon: "🎨",
        action: PaletteAction::SetLunaColor(crate::lunaline::LunaColorMode::Monochrome),
    },

    // --- View Settings & Navigation Shortcuts ---
    BuiltinCommand {
        title: "Settings: Toggle Sidebar Explorer",
        snippet: "Show or hide the document explorer drawer",
        badge: "Ctrl+B",
        icon: "👁",
        action: PaletteAction::ToggleSidebar,
    },
    BuiltinCommand {
        title: "Settings: Toggle Markdown Live Preview",
        snippet: "Split or close side-by-side formatted preview",
        badge: "Ctrl+\\",
        icon: "👁",
        action: PaletteAction::TogglePreview,
    },
    BuiltinCommand {
        title: "Settings: Toggle Interactive Terminal",
        snippet: "Open bottom docked PowerShell / cmd shell session",
        badge: "Ctrl+J",
        icon: "👁",
        action: PaletteAction::ToggleTerminal,
    },
    BuiltinCommand {
        title: "Settings: Toggle Zen Focus Mode",
        snippet: "Distraction-free pure canvas writing environment",
        badge: "Ctrl+.",
        icon: "👁",
        action: PaletteAction::ToggleZen,
    },
    BuiltinCommand {
        title: "Settings: Toggle AI Writing Assistant",
        snippet: "Open DeepSeek Pro intelligent writing assistant",
        badge: "Ctrl+Shift+I",
        icon: "👁",
        action: PaletteAction::ToggleAi,
    },
    BuiltinCommand {
        title: "Settings: Toggle Window Titlebar",
        snippet: "Show or hide the top window titlebar",
        badge: "UI",
        icon: "👁",
        action: PaletteAction::ToggleTitlebar,
    },
    BuiltinCommand {
        title: "Settings: Toggle Document Tabs",
        snippet: "Show or hide editor document tab strip",
        badge: "UI",
        icon: "👁",
        action: PaletteAction::ToggleTabs,
    },

    // --- Document & Action Settings Shortcuts ---
    BuiltinCommand {
        title: "Settings: New Note Document",
        snippet: "Create a fresh empty markdown note",
        badge: "Ctrl+N",
        icon: "📄",
        action: PaletteAction::NewNote,
    },
    BuiltinCommand {
        title: "Settings: Quick Save Active Note",
        snippet: "Commit active note buffer immediately to SQLite",
        badge: "Ctrl+S",
        icon: "📄",
        action: PaletteAction::QuickSave,
    },
    BuiltinCommand {
        title: "Settings: Rename Active Note",
        snippet: "Change topic title of the active document",
        badge: "Ctrl+R",
        icon: "📄",
        action: PaletteAction::RenameNote,
    },
    BuiltinCommand {
        title: "Settings: Delete Active Note",
        snippet: "Permanently delete current note from local vault",
        badge: "Ctrl+Shift+D",
        icon: "📄",
        action: PaletteAction::DeleteNote,
    },
    BuiltinCommand {
        title: "Settings: Toggle Checklist Checkbox",
        snippet: "Toggle checklist checkbox between [ ] and [x]",
        badge: "Ctrl+Shift+X",
        icon: "📄",
        action: PaletteAction::ToggleChecklist,
    },
    BuiltinCommand {
        title: "Settings: Close Note Tab",
        snippet: "Close the currently active note or doc tab",
        badge: "Ctrl+W",
        icon: "📄",
        action: PaletteAction::CloseTab,
    },
    BuiltinCommand {
        title: "Settings: Command Palette & Settings",
        snippet: "Search settings, themes, preferences, and shortcuts",
        badge: "Ctrl+Shift+P",
        icon: "⚡",
        action: PaletteAction::OpenSetting(crate::settings::SettingTab::Shortcuts),
    },
    BuiltinCommand {
        title: "Settings: Import Obsidian Vault or Folder",
        snippet: "Bulk-import local markdown files and vaults",
        badge: "Import",
        icon: "📄",
        action: PaletteAction::ImportWorkspace,
    },
    BuiltinCommand {
        title: "Settings: Run Web Security Scan",
        snippet: "Run automated security vulnerability scan against URL",
        badge: ":scan",
        icon: "⚡",
        action: PaletteAction::RunScan,
    },
    BuiltinCommand {
        title: "Settings: Security Scan History",
        snippet: "Review previous security vulnerability scan results",
        badge: ":scans",
        icon: "⚡",
        action: PaletteAction::ScanHistory,
    },
    BuiltinCommand {
        title: "Settings: Open Documentation & Guides",
        snippet: "Browse built-in user guides, shortcuts, and tutorials",
        badge: "F1",
        icon: "💡",
        action: PaletteAction::OpenHelp,
    },
];

/// Computes a fuzzy match score between needle and haystack.
/// Returns Some(score) if needle is a subsequence of haystack, None otherwise.
pub fn fuzzy_match(needle: &str, haystack: &str) -> Option<i64> {
    if needle.is_empty() {
        return Some(0);
    }
    let needle_chars: Vec<char> = needle.to_lowercase().chars().collect();
    let haystack_chars: Vec<char> = haystack.to_lowercase().chars().collect();

    let mut n_idx = 0;
    let mut score = 0i64;
    let mut consecutive = 0i64;

    for (h_idx, &hc) in haystack_chars.iter().enumerate() {
        if hc == needle_chars[n_idx] {
            score += 10;
            if consecutive > 0 {
                score += consecutive * 5; // bonus for consecutive letters
            }
            if h_idx == 0 || haystack_chars[h_idx - 1].is_whitespace() || haystack_chars[h_idx - 1] == '_' || haystack_chars[h_idx - 1] == '-' || haystack_chars[h_idx - 1] == ':' {
                score += 15; // word boundary bonus
            }
            consecutive += 1;
            n_idx += 1;
            if n_idx == needle_chars.len() {
                if needle_chars.len() == haystack_chars.len() {
                    score += 50; // exact match bonus
                }
                score -= (haystack_chars.len() as i64).min(30);
                return Some(score);
            }
        } else {
            consecutive = 0;
        }
    }

    None
}

/// Unified Search & Command Palette router.
/// Handles:
/// 1. `>theme [query]` -> Live interactive theme selector across all 19 themes
/// 2. `>sound [query]` -> Live interactive sound profile selector with Enter-to-preview
/// 3. `>[query]`       -> VS Code-style Command Palette across all settings, views, and actions
/// 4. `[query]`        -> Fast fuzzy search across notes and content
pub fn search_palette(
    query_str: &str,
    notes: &[core::Note],
    active_theme: ThemeKind,
    active_sound: SoundProfile,
) -> Vec<SearchItem> {
    let raw = query_str.trim();

    // ── 1. Theme Sub-Picker Mode ─────────────────────────────────────────────
    if raw.starts_with(">theme") || raw.starts_with("> theme") {
        let needle = if let Some(stripped) = raw.strip_prefix(">theme") {
            stripped.trim()
        } else if let Some(stripped) = raw.strip_prefix("> theme") {
            stripped.trim()
        } else {
            ""
        };

        let mut results = Vec::new();
        for &theme_kind in ThemeKind::ALL {
            let name = theme_kind.display_name();
            let key_name = theme_kind.name();
            let score = if needle.is_empty() {
                Some(100)
            } else {
                fuzzy_match(needle, name).or_else(|| fuzzy_match(needle, key_name))
            };

            if let Some(mut s) = score {
                let is_active = theme_kind == active_theme;
                if is_active { s += 200; }
                let snippet = if theme_kind.is_light() {
                    "Crisp Paper • Light".to_string()
                } else {
                    "Midnight Canvas • Dark".to_string()
                };
                let badge = if is_active { "✓ Active".to_string() } else { String::new() };
                results.push(SearchItem {
                    id: 0,
                    title: name.to_string(),
                    snippet,
                    score: s,
                    badge,
                    icon: "🎨",
                    action: PaletteAction::ApplyTheme(theme_kind),
                });
            }
        }
        results.sort_by(|a, b| b.score.cmp(&a.score));
        return results;
    }

    // ── 1b. Sound Sub-Picker Mode (`>sound ...`) ─────────────────────────────
    if raw.starts_with(">sound") || raw.starts_with("> sound") {
        let needle = if let Some(stripped) = raw.strip_prefix(">sound") {
            stripped.trim()
        } else if let Some(stripped) = raw.strip_prefix("> sound") {
            stripped.trim()
        } else {
            ""
        };

        let mut results = Vec::new();
        for &profile in SoundProfile::ALL.iter() {
            let name = profile.name();
            let desc = profile.description();
            let score = if needle.is_empty() {
                Some(100)
            } else {
                fuzzy_match(needle, name).or_else(|| fuzzy_match(needle, desc))
            };

            if let Some(mut s) = score {
                let is_active = profile == active_sound;
                if is_active { s += 200; }
                let badge = if is_active { "✓ Active".to_string() } else { String::new() };
                let icon = if profile == SoundProfile::Off { "🔇" } else { "🔊" };
                results.push(SearchItem {
                    id: 0,
                    title: name.to_string(),
                    snippet: desc.to_string(),
                    score: s,
                    badge,
                    icon,
                    action: PaletteAction::ApplySoundProfile(profile),
                });
            }
        }
        results.sort_by(|a, b| b.score.cmp(&a.score));
        return results;
    }

    // ── 2. VS Code Command Palette Mode (`>...`) ─────────────────────────────
    if raw.starts_with('>') {
        let needle = raw.strip_prefix('>').unwrap_or("").trim();
        let mut results = Vec::new();

        for cmd in BUILTIN_COMMANDS {
            let score = if needle.is_empty() {
                Some(100)
            } else {
                let s_title = fuzzy_match(needle, cmd.title);
                let s_snip = fuzzy_match(needle, cmd.snippet);
                let s_badge = fuzzy_match(needle, cmd.badge);
                s_title.or(s_snip).or(s_badge)
            };

            if let Some(s) = score {
                results.push(SearchItem {
                    id: 0,
                    title: cmd.title.to_string(),
                    snippet: cmd.snippet.to_string(),
                    score: s,
                    badge: cmd.badge.to_string(),
                    icon: cmd.icon,
                    action: cmd.action.clone(),
                });
            }
        }
        results.sort_by(|a, b| b.score.cmp(&a.score));
        return results;
    }

    // ── 3. Notes Fuzzy Search Mode ───────────────────────────────────────────
    let mut results = Vec::new();
    for note in notes {
        let score_topic = fuzzy_match(raw, &note.topic);
        let score_body = fuzzy_match(raw, &note.body);
        if let Some(score) = score_topic.or(score_body) {
            let snippet = if raw.is_empty() {
                if note.body.len() > 60 {
                    format!("{}...", &note.body[..60].replace('\n', " "))
                } else {
                    note.body.replace('\n', " ")
                }
            } else if let Some(pos) = note.body.to_lowercase().find(&raw.to_lowercase()) {
                let start = pos.saturating_sub(20);
                let end = (pos + raw.len() + 40).min(note.body.len());
                format!("...{}...", note.body[start..end].replace('\n', " "))
            } else if note.body.len() > 60 {
                format!("{}...", &note.body[..60].replace('\n', " "))
            } else {
                note.body.replace('\n', " ")
            };

            results.push(SearchItem::for_note(
                note.id,
                note.topic.clone(),
                snippet,
                score,
            ));
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("rust", "The Rust Programming Language").is_some());
        assert!(fuzzy_match("rpl", "Rust Programming Language").is_some());
        assert!(fuzzy_match("xyz", "Rust Language").is_none());

        let score_prefix = fuzzy_match("rust", "Rust").unwrap();
        let score_sub = fuzzy_match("rust", "A long text about Rust").unwrap();
        assert!(score_prefix > score_sub);
    }

    #[test]
    fn test_command_palette_matching() {
        let items = search_palette(">", &[], ThemeKind::TokyoNight, crate::sound::SoundProfile::Off);
        assert!(!items.is_empty());
        assert!(items.iter().all(|i| i.title.starts_with("Settings: ")));
        assert!(items.iter().any(|i| i.title.contains("Color Theme")));
        assert!(items.iter().any(|i| i.title.contains("Keyboard Shortcuts")));

        let theme_filter = search_palette(">theme", &[], ThemeKind::TokyoNight, crate::sound::SoundProfile::Off);
        assert_eq!(theme_filter.len(), ThemeKind::ALL.len());
        let active = theme_filter.iter().find(|i| i.badge.contains("Active"));
        assert!(active.is_some());
        // Inactive themes must have empty badges
        let inactive_with_badge = theme_filter.iter().filter(|i| !i.badge.is_empty() && !i.badge.contains("Active")).count();
        assert_eq!(inactive_with_badge, 0);

        // Sound picker: >sound shows all profiles
        let sound_filter = search_palette(">sound", &[], ThemeKind::TokyoNight, crate::sound::SoundProfile::Thocky);
        assert_eq!(sound_filter.len(), crate::sound::SoundProfile::ALL.len());
        let active_sound = sound_filter.iter().find(|i| i.badge.contains("Active"));
        assert!(active_sound.is_some());
        assert_eq!(active_sound.unwrap().title, "Thocky");
        // Inactive sound profiles must have empty badges
        let inactive_sound_with_badge = sound_filter.iter().filter(|i| !i.badge.is_empty() && !i.badge.contains("Active")).count();
        assert_eq!(inactive_sound_with_badge, 0);

        // Note search items must have empty badges
        let sample_notes = vec![core::Note {
            id: 1,
            topic: "Architecture".to_string(),
            body: "MindForge system design".to_string(),
            struggled_with: None,
            created_at: chrono::NaiveDateTime::default(),
        }];
        let note_results = search_palette("Arch", &sample_notes, ThemeKind::TokyoNight, crate::sound::SoundProfile::Off);
        assert!(!note_results.is_empty());
        assert_eq!(note_results[0].badge, "");
    }
}
