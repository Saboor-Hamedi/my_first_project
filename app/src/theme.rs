use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Shell,
    TokyoNight,
    Dracula,
    Catppuccin,
    Nord,
    RosePine,
    Cyberpunk,
    Green,
    Amber,
    Ice,
    White,
}

impl ThemeKind {
    pub const ALL: &'static [ThemeKind] = &[
        ThemeKind::Shell,
        ThemeKind::TokyoNight,
        ThemeKind::Dracula,
        ThemeKind::Catppuccin,
        ThemeKind::Nord,
        ThemeKind::RosePine,
        ThemeKind::Cyberpunk,
        ThemeKind::Green,
        ThemeKind::Amber,
        ThemeKind::Ice,
        ThemeKind::White,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "shell" | "012b36" | "terminal" | "solarized" | "solarizeddark" => Some(Self::Shell),
            "green" | "matrix" => Some(Self::Green),
            "amber" => Some(Self::Amber),
            "tokyo" | "tokyonight" => Some(Self::TokyoNight),
            "dracula" => Some(Self::Dracula),
            "catppuccin" | "mocha" => Some(Self::Catppuccin),
            "nord" => Some(Self::Nord),
            "cyberpunk" | "cyber" => Some(Self::Cyberpunk),
            "rose" | "rosepine" => Some(Self::RosePine),
            "ice" => Some(Self::Ice),
            "white" | "mono" | "monochrome" => Some(Self::White),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Shell => "shell",
            Self::TokyoNight => "tokyonight",
            Self::Dracula => "dracula",
            Self::Catppuccin => "catppuccin",
            Self::Nord => "nord",
            Self::RosePine => "rosepine",
            Self::Cyberpunk => "cyberpunk",
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Ice => "ice",
            Self::White => "white",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Shell => "Shell (#012b36)",
            Self::TokyoNight => "Tokyo Night",
            Self::Dracula => "Dracula",
            Self::Catppuccin => "Catppuccin",
            Self::Nord => "Nord Arctic",
            Self::RosePine => "Rosé Pine",
            Self::Cyberpunk => "Cyberpunk",
            Self::Green => "Matrix Green",
            Self::Amber => "Amber CRT",
            Self::Ice => "Glacier Ice",
            Self::White => "Monochrome",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub kind: ThemeKind,
    pub bg: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub muted: Color32,
    pub highlight: Color32,
}

impl Theme {
    /// Slightly elevated surface color for cards and active tabs
    pub fn surface(&self) -> Color32 {
        let r = (self.bg.r() as u16 + 10).min(255) as u8;
        let g = (self.bg.g() as u16 + 12).min(255) as u8;
        let b = (self.bg.b() as u16 + 16).min(255) as u8;
        Color32::from_rgb(r, g, b)
    }

    /// Border color for containers and dividers
    pub fn border(&self) -> Color32 {
        let r = (self.bg.r() as u16 + 22).min(255) as u8;
        let g = (self.bg.g() as u16 + 26).min(255) as u8;
        let b = (self.bg.b() as u16 + 32).min(255) as u8;
        Color32::from_rgb(r, g, b)
    }

    /// Sidebar background color (subtly adjusted relative to theme.bg)
    pub fn sidebar_bg(&self) -> Color32 {
        let r = self.bg.r().saturating_sub(4);
        let g = self.bg.g().saturating_sub(4);
        let b = self.bg.b().saturating_sub(4);
        Color32::from_rgb(r, g, b)
    }

    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Shell => Self {
                kind,
                bg: Color32::from_rgb(1, 43, 54),       // #012b36 (Solarized deep teal shell)
                text: Color32::from_rgb(238, 232, 213), // #eee8d5 Solarized base3
                accent: Color32::from_rgb(42, 161, 152), // #2aa198 Solarized cyan
                muted: Color32::from_rgb(101, 123, 131), // #657b83 Solarized base00
                highlight: Color32::from_rgb(38, 139, 210), // #268bd2 Solarized blue
            },
            ThemeKind::TokyoNight => Self {
                kind,
                bg: Color32::from_rgb(26, 27, 38),      // #1a1b26
                text: Color32::from_rgb(192, 202, 245),
                accent: Color32::from_rgb(122, 162, 247),
                muted: Color32::from_rgb(115, 125, 165),
                highlight: Color32::from_rgb(187, 154, 247),
            },
            ThemeKind::Dracula => Self {
                kind,
                bg: Color32::from_rgb(40, 42, 54),      // #282a36
                text: Color32::from_rgb(248, 248, 242),
                accent: Color32::from_rgb(255, 121, 198),
                muted: Color32::from_rgb(139, 147, 184),
                highlight: Color32::from_rgb(139, 233, 253),
            },
            ThemeKind::Catppuccin => Self {
                kind,
                bg: Color32::from_rgb(30, 30, 46),      // #1e1e2e
                text: Color32::from_rgb(205, 214, 244),
                accent: Color32::from_rgb(203, 166, 247),
                muted: Color32::from_rgb(140, 146, 175),
                highlight: Color32::from_rgb(245, 194, 231),
            },
            ThemeKind::Nord => Self {
                kind,
                bg: Color32::from_rgb(46, 52, 64),      // #2e3440
                text: Color32::from_rgb(236, 239, 244),
                accent: Color32::from_rgb(136, 192, 208),
                muted: Color32::from_rgb(140, 150, 175),
                highlight: Color32::from_rgb(143, 188, 187),
            },
            ThemeKind::RosePine => Self {
                kind,
                bg: Color32::from_rgb(31, 29, 46),      // #1f1d2e
                text: Color32::from_rgb(224, 222, 244),
                accent: Color32::from_rgb(235, 111, 146),
                muted: Color32::from_rgb(140, 135, 168),
                highlight: Color32::from_rgb(246, 193, 119),
            },
            ThemeKind::Cyberpunk => Self {
                kind,
                bg: Color32::from_rgb(24, 18, 40),
                text: Color32::from_rgb(160, 245, 255),
                accent: Color32::from_rgb(255, 225, 53),
                muted: Color32::from_rgb(145, 110, 175),
                highlight: Color32::from_rgb(255, 42, 109),
            },
            ThemeKind::Green => Self {
                kind,
                bg: Color32::from_rgb(12, 24, 15),
                text: Color32::from_rgb(190, 245, 205),
                accent: Color32::from_rgb(51, 255, 102),
                muted: Color32::from_rgb(95, 155, 115),
                highlight: Color32::from_rgb(110, 255, 160),
            },
            ThemeKind::Amber => Self {
                kind,
                bg: Color32::from_rgb(26, 20, 12),
                text: Color32::from_rgb(255, 230, 185),
                accent: Color32::from_rgb(255, 175, 45),
                muted: Color32::from_rgb(165, 125, 75),
                highlight: Color32::from_rgb(255, 215, 105),
            },
            ThemeKind::Ice => Self {
                kind,
                bg: Color32::from_rgb(16, 28, 44),
                text: Color32::from_rgb(220, 245, 255),
                accent: Color32::from_rgb(80, 210, 255),
                muted: Color32::from_rgb(95, 155, 185),
                highlight: Color32::from_rgb(150, 235, 255),
            },
            ThemeKind::White => Self {
                kind,
                bg: Color32::from_rgb(26, 26, 30),
                text: Color32::from_rgb(240, 240, 240),
                accent: Color32::from_rgb(255, 255, 255),
                muted: Color32::from_rgb(150, 150, 150),
                highlight: Color32::from_rgb(210, 230, 255),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_kind(ThemeKind::Shell)
    }
}
