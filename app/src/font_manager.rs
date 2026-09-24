//! Font management, dynamic font discovery, and real-time egui typography loading.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub preview_snippet: &'static str,
    pub is_embedded: bool,
}

pub const SUPPORTED_FONTS: &[FontMetadata] = &[
    FontMetadata {
        id: "jetbrains_mono",
        display_name: "JetBrains Mono",
        description: "Built-in • Crisp developer font with modern coding ligatures",
        preview_snippet: "fn main() -> Result<(), Error> { let x = 42; }",
        is_embedded: true,
    },
    FontMetadata {
        id: "caskaydia_cove",
        display_name: "Cascadia Code / Caskaydia",
        description: "Modern Microsoft monospace with terminal glyphs & icons",
        preview_snippet: "git status: 3 modified, 1 untracked [main ⚡]",
        is_embedded: false,
    },
    FontMetadata {
        id: "fira_code",
        display_name: "Fira Code",
        description: "Classic developer typeface with extensive coding ligatures",
        preview_snippet: "0 <= x && x <= 100 === true // rich ligatures",
        is_embedded: false,
    },
    FontMetadata {
        id: "victor_mono",
        display_name: "Victor Mono",
        description: "Elegant monospace with signature cursive italics",
        preview_snippet: "struct Matrix { rows: usize, cols: usize }",
        is_embedded: false,
    },
    FontMetadata {
        id: "iosevka",
        display_name: "Iosevka",
        description: "Ultra-condensed monospace with maximum line density",
        preview_snippet: "type Result<T> = std::result::Result<T, Error>;",
        is_embedded: false,
    },
    FontMetadata {
        id: "berkeley_mono",
        display_name: "Berkeley Mono",
        description: "Precision-engineered typeface for craft developers",
        preview_snippet: "SELECT * FROM notes WHERE topic MATCH 'rust*';",
        is_embedded: false,
    },
];

/// Validates raw font binary headers to prevent any epaint parser panics.
/// TrueType headers: 0x00010000 or 'true'
/// OpenType headers: 'OTTO'
/// TrueType Collection: 'ttcf'
/// PostScript Type 1: 'typ1'
pub fn is_valid_font_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 12 {
        return false;
    }
    bytes.starts_with(&[0x00, 0x01, 0x00, 0x00])
        || bytes.starts_with(b"OTTO")
        || bytes.starts_with(b"true")
        || bytes.starts_with(b"ttcf")
        || bytes.starts_with(b"typ1")
}

/// Searches for the font on the local operating system (Windows fonts directories, user fonts).
pub fn find_font_file(font_name: &str) -> Option<PathBuf> {
    let normalized = font_name.to_lowercase().replace([' ', '-', '_', '/'], "");

    let candidate_names: Vec<&str> = if normalized.contains("jetbrains") {
        vec![
            "JetBrainsMono-Regular.ttf",
            "JetBrainsMonoNerdFont-Regular.ttf",
            "JetBrainsMonoNLNerdFont-Regular.ttf",
            "JetBrainsMonoNLNerdFontMono-Regular.ttf",
            "JetBrainsMono.ttf",
        ]
    } else if normalized.contains("cascadia") || normalized.contains("caskaydia") {
        vec![
            "CascadiaCode-Regular.ttf",
            "CascadiaCode.ttf",
            "CascadiaMono.ttf",
            "CaskaydiaCoveNerdFont-Regular.ttf",
            "CaskaydiaCoveNerdFontMono-Regular.ttf",
            "cascadiacode.ttf",
        ]
    } else if normalized.contains("fira") {
        vec![
            "FiraCode-Regular.ttf",
            "FiraCode-Retina.ttf",
            "FiraCode-Medium.ttf",
            "FiraCode.ttf",
            "Fira Code Regular.ttf",
            "firacode-regular.ttf",
        ]
    } else if normalized.contains("victor") {
        vec![
            "VictorMono-Regular.ttf",
            "VictorMono-Medium.ttf",
            "VictorMono.ttf",
            "victormono-regular.ttf",
            "Victor Mono Regular.ttf",
        ]
    } else if normalized.contains("iosevka") {
        vec![
            "Iosevka-Regular.ttf",
            "iosevka-regular.ttf",
            "Iosevka.ttf",
            "iosevka.ttf",
            "IosevkaFixed-Regular.ttf",
            "IosevkaTerm-Regular.ttf",
        ]
    } else if normalized.contains("berkeley") {
        vec![
            "BerkeleyMono-Regular.ttf",
            "BerkeleyMono.ttf",
            "berkeleymono-regular.ttf",
            "BerkeleyMono-Regular.otf",
            "Berkeley Mono Regular.ttf",
        ]
    } else if normalized.contains("hack") {
        vec![
            "hack.regular.ttf",
            "Hack-Regular.ttf",
            "Hack.ttf",
        ]
    } else {
        return None;
    };

    let mut search_dirs = Vec::new();
    #[cfg(target_os = "windows")]
    {
        search_dirs.push(PathBuf::from(r"C:\Windows\Fonts"));
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            search_dirs.push(PathBuf::from(local_app_data).join(r"Microsoft\Windows\Fonts"));
        }
        if let Some(proj) = directories::ProjectDirs::from("com", "mindforge", "mindforge") {
            search_dirs.push(proj.data_local_dir().join("fonts"));
        }
    }
    search_dirs.push(PathBuf::from("fonts"));
    search_dirs.push(PathBuf::from("app/assets"));

    for dir in &search_dirs {
        for candidate in &candidate_names {
            let p = dir.join(candidate);
            if p.exists() {
                if let Ok(bytes) = std::fs::read(&p) {
                    if is_valid_font_bytes(&bytes) {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Checks if a font is available (either embedded or installed locally).
pub fn is_font_available(font_name: &str) -> bool {
    let lower = font_name.to_lowercase().replace([' ', '-', '_', '/'], "");
    if lower.contains("jetbrains") || lower == "default" {
        return true;
    }
    find_font_file(font_name).is_some()
}

/// Configures and applies the chosen font to egui context dynamically.
/// Completely crash-proof: verifies font validity before feeding into epaint.
pub fn apply_font(ctx: &egui::Context, font_name: &str) {
    let mut fonts = FontDefinitions::default();

    // 1. Embedded core developer font: JetBrains Mono (verified TTF asset)
    fonts.font_data.insert(
        "jetbrains_mono".into(),
        FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf")).into(),
    );

    // 2. Windows system emoji
    #[cfg(target_os = "windows")]
    {
        if let Ok(emoji_bytes) = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf") {
            if is_valid_font_bytes(&emoji_bytes) {
                fonts.font_data.insert(
                    "win_emoji".into(),
                    FontData::from_owned(emoji_bytes).into(),
                );
            }
        }
    }

    // 3. Resolve active font key safely
    let lower = font_name.to_lowercase().replace([' ', '-', '_', '/'], "");
    let active_font_key = if lower.contains("jetbrains") || lower == "default" {
        "jetbrains_mono".to_string()
    } else if let Some(font_path) = find_font_file(font_name) {
        if let Ok(bytes) = std::fs::read(&font_path) {
            if is_valid_font_bytes(&bytes) {
                fonts.font_data.insert(
                    "user_selected_font".into(),
                    FontData::from_owned(bytes).into(),
                );
                "user_selected_font".to_string()
            } else {
                "jetbrains_mono".to_string()
            }
        } else {
            "jetbrains_mono".to_string()
        }
    } else {
        "jetbrains_mono".to_string()
    };

    // Apply to monospace
    let mono = fonts.families.get_mut(&FontFamily::Monospace).unwrap();
    mono.clear();
    mono.push(active_font_key.clone());
    if active_font_key != "jetbrains_mono" {
        mono.push("jetbrains_mono".into());
    }
    #[cfg(target_os = "windows")]
    if fonts.font_data.contains_key("win_emoji") {
        mono.push("win_emoji".into());
    }

    // Apply to proportional
    let prop = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
    prop.clear();
    prop.push(active_font_key);
    prop.push("jetbrains_mono".into());
    #[cfg(target_os = "windows")]
    if fonts.font_data.contains_key("win_emoji") {
        prop.push("win_emoji".into());
    }

    ctx.set_fonts(fonts);
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_fonts_contain_requested_typography() {
        let names: Vec<&str> = SUPPORTED_FONTS.iter().map(|f| f.display_name).collect();
        assert!(names.iter().any(|n| n.contains("JetBrains Mono")));
        assert!(names.iter().any(|n| n.contains("Iosevka")));
        assert!(names.iter().any(|n| n.contains("Victor Mono")));
        assert!(names.iter().any(|n| n.contains("Fira Code")));
        assert!(names.iter().any(|n| n.contains("Cascadia") || n.contains("Caskaydia")));
        assert!(names.iter().any(|n| n.contains("Berkeley Mono")));
        assert_eq!(SUPPORTED_FONTS.len(), 6);
    }

    #[test]
    fn test_embedded_font_is_always_available() {
        assert!(is_font_available("JetBrains Mono"));
        assert!(is_font_available("jetbrains_mono"));
        assert!(is_font_available("default"));
    }

    #[test]
    fn test_font_validation_rejects_corrupted_data() {
        assert!(!is_valid_font_bytes(b""));
        assert!(!is_valid_font_bytes(b"<html>404 Not Found</html>"));
        assert!(!is_valid_font_bytes(&[0u8; 10]));
        assert!(is_valid_font_bytes(&[0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0]));
        assert!(is_valid_font_bytes(b"OTTO\0\0\0\0\0\0\0\0"));
    }
}
