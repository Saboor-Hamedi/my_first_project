use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Green,
    Amber,
    TokyoNight,
    Dracula,
    Catppuccin,
    Nord,
    Cyberpunk,
    RosePine,
    Ice,
    White,
}

impl ThemeKind {
    pub const ALL: &'static [ThemeKind] = &[
        ThemeKind::Green,
        ThemeKind::TokyoNight,
        ThemeKind::Dracula,
        ThemeKind::Catppuccin,
        ThemeKind::Nord,
        ThemeKind::Cyberpunk,
        ThemeKind::RosePine,
        ThemeKind::Amber,
        ThemeKind::Ice,
        ThemeKind::White,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
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
            Self::Green => "green",
            Self::Amber => "amber",
            Self::TokyoNight => "tokyonight",
            Self::Dracula => "dracula",
            Self::Catppuccin => "catppuccin",
            Self::Nord => "nord",
            Self::Cyberpunk => "cyberpunk",
            Self::RosePine => "rosepine",
            Self::Ice => "ice",
            Self::White => "white",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Green => "Matrix Green",
            Self::Amber => "Amber CRT",
            Self::TokyoNight => "Tokyo Night",
            Self::Dracula => "Dracula",
            Self::Catppuccin => "Catppuccin",
            Self::Nord => "Nord Arctic",
            Self::Cyberpunk => "Cyberpunk",
            Self::RosePine => "Rosé Pine",
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
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Green => Self {
                kind,
                bg: Color32::from_rgb(11, 15, 12),
                text: Color32::from_rgb(190, 245, 205),
                accent: Color32::from_rgb(51, 255, 102),
                muted: Color32::from_rgb(80, 130, 95),
                highlight: Color32::from_rgb(110, 255, 160),
            },
            ThemeKind::Amber => Self {
                kind,
                bg: Color32::from_rgb(16, 13, 10),
                text: Color32::from_rgb(255, 230, 185),
                accent: Color32::from_rgb(255, 175, 45),
                muted: Color32::from_rgb(145, 105, 55),
                highlight: Color32::from_rgb(255, 215, 105),
            },
            ThemeKind::TokyoNight => Self {
                kind,
                bg: Color32::from_rgb(15, 17, 26),
                text: Color32::from_rgb(192, 202, 245),
                accent: Color32::from_rgb(122, 162, 247),
                muted: Color32::from_rgb(86, 95, 137),
                highlight: Color32::from_rgb(187, 154, 247),
            },
            ThemeKind::Dracula => Self {
                kind,
                bg: Color32::from_rgb(17, 16, 24),
                text: Color32::from_rgb(248, 248, 242),
                accent: Color32::from_rgb(255, 121, 198),
                muted: Color32::from_rgb(98, 114, 164),
                highlight: Color32::from_rgb(139, 233, 253),
            },
            ThemeKind::Catppuccin => Self {
                kind,
                bg: Color32::from_rgb(17, 17, 27),
                text: Color32::from_rgb(205, 214, 244),
                accent: Color32::from_rgb(203, 166, 247),
                muted: Color32::from_rgb(108, 112, 134),
                highlight: Color32::from_rgb(245, 194, 231),
            },
            ThemeKind::Nord => Self {
                kind,
                bg: Color32::from_rgb(15, 18, 23),
                text: Color32::from_rgb(236, 239, 244),
                accent: Color32::from_rgb(136, 192, 208),
                muted: Color32::from_rgb(94, 107, 133),
                highlight: Color32::from_rgb(143, 188, 187),
            },
            ThemeKind::Cyberpunk => Self {
                kind,
                bg: Color32::from_rgb(14, 11, 20),
                text: Color32::from_rgb(160, 245, 255),
                accent: Color32::from_rgb(255, 225, 53),
                muted: Color32::from_rgb(115, 85, 145),
                highlight: Color32::from_rgb(255, 42, 109),
            },
            ThemeKind::RosePine => Self {
                kind,
                bg: Color32::from_rgb(18, 15, 23),
                text: Color32::from_rgb(224, 222, 244),
                accent: Color32::from_rgb(235, 111, 146),
                muted: Color32::from_rgb(110, 106, 134),
                highlight: Color32::from_rgb(246, 193, 119),
            },
            ThemeKind::Ice => Self {
                kind,
                bg: Color32::from_rgb(10, 16, 24),
                text: Color32::from_rgb(210, 240, 255),
                accent: Color32::from_rgb(80, 210, 255),
                muted: Color32::from_rgb(70, 130, 160),
                highlight: Color32::from_rgb(140, 230, 255),
            },
            ThemeKind::White => Self {
                kind,
                bg: Color32::from_rgb(14, 14, 16),
                text: Color32::from_rgb(235, 235, 235),
                accent: Color32::from_rgb(255, 255, 255),
                muted: Color32::from_rgb(130, 130, 130),
                highlight: Color32::from_rgb(200, 220, 255),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_kind(ThemeKind::Green)
    }
}
